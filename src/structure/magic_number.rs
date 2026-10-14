use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects inline numeric literals that should be extracted into named
    /// constants, configuration fields, or domain wrappers.
    pub tigerstyle::MAGIC_NUMBER,
    Warn,
    "inline numeric literal should be named or configured"
}

declare_lint_pass!(MagicNumber => [MAGIC_NUMBER]);

const UNIT_SUFFIXES: &[&str] = &[
    "_ms", "_millis", "_secs", "_sec", "_us", "_micros", "_ns", "_nanos", "_bytes", "_kb", "_mb", "_gb",
];

const DOMAIN_WORDS: &[&str] = &[
    "timeout",
    "deadline",
    "delay",
    "retry",
    "reconnect",
    "capacity",
    "limit",
    "threshold",
    "window",
    "batch",
    "port",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LiteralKind {
    Numeric,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LiteralConvention {
    Conventional,
    DomainSpecific,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceOrigin {
    Authored,
    Expansion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MagicNumberFacts {
    literal_kind: LiteralKind,
    convention: LiteralConvention,
    is_documented_by_parent: bool,
    is_policy_threshold_comparison: bool,
    is_in_test_context: bool,
    source_origin: SourceOrigin,
}

fn classify_magic_number(facts: MagicNumberFacts) -> bool {
    facts.literal_kind == LiteralKind::Numeric
        && facts.convention == LiteralConvention::DomainSpecific
        && !facts.is_documented_by_parent
        && facts.is_policy_threshold_comparison
        && !facts.is_in_test_context
        && facts.source_origin == SourceOrigin::Authored
}

fn has_unit_suffix(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    UNIT_SUFFIXES.iter().any(|suffix| lower.ends_with(suffix))
}

fn has_domain_word(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    DOMAIN_WORDS.iter().any(|word| lower.contains(word))
}

fn is_conventional_literal(lit: &hir::Lit) -> bool {
    match lit.node {
        LitKind::Int(value, _) => matches!(value.get(), 0 | 1),
        LitKind::Float(symbol, _) => matches!(symbol.as_str(), "0.0" | "1.0"),
        _ => true,
    }
}

fn literal_is_documented_by_parent(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::LetStmt(local) => {
                if let hir::PatKind::Binding(_, _, ident, _) = local.pat.kind {
                    let name = ident.name.as_str();
                    if has_unit_suffix(name) || has_domain_word(name) {
                        return true;
                    }
                }
            }
            hir::Node::Expr(parent) => match parent.kind {
                ExprKind::Call(_, args) if args.iter().any(|arg| arg.hir_id == expr.hir_id) => return true,
                ExprKind::MethodCall(_, receiver, args, _)
                    if receiver.hir_id == expr.hir_id || args.iter().any(|arg| arg.hir_id == expr.hir_id) =>
                {
                    return true;
                }
                _ => {}
            },
            hir::Node::Item(item) => {
                if matches!(item.kind, hir::ItemKind::Const(..) | hir::ItemKind::Static(..)) {
                    return true;
                }
                break;
            }
            hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    false
}

fn other_expr_has_domain_name(expr: &hir::Expr<'_>) -> bool {
    match expr.kind {
        ExprKind::Path(hir::QPath::Resolved(_, path)) => path.segments.last().is_some_and(|segment| {
            let name = segment.ident.name.as_str();
            has_unit_suffix(name) || has_domain_word(name)
        }),
        _ => false,
    }
}

fn literal_is_policy_threshold_comparison(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    const COMPARISON_PARENT_SCAN_LIMIT: usize = 3;

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id).take(COMPARISON_PARENT_SCAN_LIMIT) {
        let hir::Node::Expr(parent) = node else {
            continue;
        };
        let ExprKind::Binary(op, lhs, rhs) = parent.kind else {
            continue;
        };
        if !matches!(
            op.node,
            hir::BinOpKind::Lt
                | hir::BinOpKind::Le
                | hir::BinOpKind::Gt
                | hir::BinOpKind::Ge
                | hir::BinOpKind::Eq
                | hir::BinOpKind::Ne
        ) {
            return false;
        }
        if lhs.hir_id == expr.hir_id {
            return other_expr_has_domain_name(rhs);
        }
        if rhs.hir_id == expr.hir_id {
            return other_expr_has_domain_name(lhs);
        }
    }
    false
}

impl LateLintPass<'_> for MagicNumber {
    fn check_expr(&mut self, cx: &LateContext<'_>, expr: &hir::Expr<'_>) {
        let ExprKind::Lit(lit) = expr.kind else {
            return;
        };

        let facts = MagicNumberFacts {
            literal_kind: if matches!(lit.node, LitKind::Int(..) | LitKind::Float(..)) {
                LiteralKind::Numeric
            } else {
                LiteralKind::Other
            },
            convention: if is_conventional_literal(&lit) {
                LiteralConvention::Conventional
            } else {
                LiteralConvention::DomainSpecific
            },
            is_documented_by_parent: literal_is_documented_by_parent(cx, expr),
            is_policy_threshold_comparison: literal_is_policy_threshold_comparison(cx, expr),
            is_in_test_context: test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span),
            source_origin: if expr.span.from_expansion() {
                SourceOrigin::Expansion
            } else {
                SourceOrigin::Authored
            },
        };
        if !classify_magic_number(facts) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            MAGIC_NUMBER,
            expr.span,
            "inline numeric literal should be extracted into a named constant or configuration field",
            "magic_number",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::LiteralConvention;
    use super::LiteralKind;
    use super::MagicNumberFacts;
    use super::SourceOrigin;
    use super::classify_magic_number;
    use super::has_domain_word;
    use super::has_unit_suffix;

    fn policy_threshold_literal() -> MagicNumberFacts {
        MagicNumberFacts {
            literal_kind: LiteralKind::Numeric,
            convention: LiteralConvention::DomainSpecific,
            is_documented_by_parent: false,
            is_policy_threshold_comparison: true,
            is_in_test_context: false,
            source_origin: SourceOrigin::Authored,
        }
    }

    #[test]
    fn classifies_inline_policy_threshold() {
        assert!(classify_magic_number(policy_threshold_literal()));
    }

    #[test]
    fn skips_documented_or_conventional_literals() {
        let documented = MagicNumberFacts {
            is_documented_by_parent: true,
            ..policy_threshold_literal()
        };
        let conventional = MagicNumberFacts {
            convention: LiteralConvention::Conventional,
            ..policy_threshold_literal()
        };

        assert!(!classify_magic_number(documented));
        assert!(!classify_magic_number(conventional));
    }

    #[test]
    fn skips_non_threshold_literals_and_test_contexts() {
        let non_threshold = MagicNumberFacts {
            is_policy_threshold_comparison: false,
            ..policy_threshold_literal()
        };
        let test_context = MagicNumberFacts {
            is_in_test_context: true,
            ..policy_threshold_literal()
        };

        assert!(!classify_magic_number(non_threshold));
        assert!(!classify_magic_number(test_context));
    }

    #[test]
    fn recognizes_policy_names_and_unit_suffixes() {
        assert!(has_unit_suffix("retry_delay_ms"));
        assert!(has_domain_word("connection_threshold"));
        assert!(!has_unit_suffix("attempt"));
        assert!(!has_domain_word("value"));
    }
}
