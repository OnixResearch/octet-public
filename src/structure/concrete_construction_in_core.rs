use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "concrete_construction_in_core";
const NEW_CALLEE_SUFFIX: &str = "::new";
const DEFAULT_CALLEE_SUFFIX: &str = "::default";
const STANDARD_VALUE_CRATES: &[&str] = &["alloc", "core", "std"];

declare_tool_lint! {
    /// Detects concrete infrastructure construction inside declared core scopes.
    /// Core code should accept injected ports/capabilities instead of constructing
    /// external concrete dependencies directly.
    pub tigerstyle::CONCRETE_CONSTRUCTION_IN_CORE,
    Warn,
    "concrete dependency constructed in core scope"
}

pub struct ConcreteConstructionInCore {
    core_source_scopes: Vec<String>,
}

impl ConcreteConstructionInCore {
    pub fn new(core_source_scopes: Vec<String>) -> Self {
        Self { core_source_scopes }
    }
}

impl_lint_pass!(ConcreteConstructionInCore => [CONCRETE_CONSTRUCTION_IN_CORE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructedTypeOrigin {
    LocalDomain,
    StandardValue,
    External,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructionKind {
    Direct,
    Indirect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ConstructionFacts {
    in_core_scope: bool,
    from_expansion: bool,
    in_test_context: bool,
    kind: ConstructionKind,
    type_origin: ConstructedTypeOrigin,
}

fn should_emit_construction(facts: ConstructionFacts) -> bool {
    facts.in_core_scope
        && !facts.from_expansion
        && !facts.in_test_context
        && facts.kind == ConstructionKind::Direct
        && facts.type_origin == ConstructedTypeOrigin::External
}

fn path_is_in_core_scope(cx: &LateContext<'_>, span: rustc_span::Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let filename = cx.sess().source_map().span_to_filename(span);
    let path = filename.prefer_local_unconditionally().to_string();
    scopes.iter().any(|scope| path.contains(scope))
}

fn direct_constructor_name(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    let hir::ExprKind::Call(callee, _) = expr.kind else {
        return None;
    };
    let snippet = cx.sess().source_map().span_to_snippet(callee.span).ok()?;
    let type_name = snippet
        .strip_suffix(NEW_CALLEE_SUFFIX)
        .or_else(|| snippet.strip_suffix(DEFAULT_CALLEE_SUFFIX))?
        .trim();
    if type_name.is_empty() {
        return None;
    }
    Some(type_name.to_owned())
}

fn constructed_type_origin(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> ConstructedTypeOrigin {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return ConstructedTypeOrigin::Unsupported;
    };
    if adt.did().is_local() {
        return ConstructedTypeOrigin::LocalDomain;
    }
    let path = cx.get_def_path(adt.did());
    let Some(crate_name) = path.first().map(rustc_span::Symbol::as_str) else {
        return ConstructedTypeOrigin::Unsupported;
    };
    if STANDARD_VALUE_CRATES.contains(&crate_name) {
        ConstructedTypeOrigin::StandardValue
    } else {
        ConstructedTypeOrigin::External
    }
}

impl<'tcx> LateLintPass<'tcx> for ConcreteConstructionInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let constructor_name = direct_constructor_name(cx, expr);
        let facts = ConstructionFacts {
            in_core_scope: path_is_in_core_scope(cx, expr.span, &self.core_source_scopes),
            from_expansion: expr.span.from_expansion(),
            in_test_context: test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span),
            kind: if constructor_name.is_some() {
                ConstructionKind::Direct
            } else {
                ConstructionKind::Indirect
            },
            type_origin: constructed_type_origin(cx, expr),
        };
        if !should_emit_construction(facts) {
            return;
        }
        let Some(type_name) = constructor_name else {
            return;
        };

        emit_tiger_style_lint(
            cx,
            CONCRETE_CONSTRUCTION_IN_CORE,
            expr.span,
            format!(
                "core scope constructs concrete type `{type_name}`; inject the dependency through a port or capability"
            ),
            LINT_NAME,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(type_origin: ConstructedTypeOrigin) -> ConstructionFacts {
        ConstructionFacts {
            in_core_scope: true,
            from_expansion: false,
            in_test_context: false,
            kind: ConstructionKind::Direct,
            type_origin,
        }
    }

    #[test]
    fn direct_external_constructor_is_reportable() {
        assert!(should_emit_construction(facts(ConstructedTypeOrigin::External)));
    }

    #[test]
    fn local_and_standard_value_constructors_are_not_reportable() {
        assert!(!should_emit_construction(facts(ConstructedTypeOrigin::LocalDomain)));
        assert!(!should_emit_construction(facts(ConstructedTypeOrigin::StandardValue)));
    }

    #[test]
    fn indirect_parent_and_out_of_scope_shapes_are_not_reportable() {
        let mut indirect = facts(ConstructedTypeOrigin::External);
        indirect.kind = ConstructionKind::Indirect;
        let mut out_of_scope = facts(ConstructedTypeOrigin::External);
        out_of_scope.in_core_scope = false;

        assert!(!should_emit_construction(indirect));
        assert!(!should_emit_construction(out_of_scope));
    }
}
