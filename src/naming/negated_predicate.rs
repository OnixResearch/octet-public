use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use super::use_site_collector;
use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects negative predicate names on boolean-returning functions and
    /// bool-typed local bindings. Tiger Style prefers positive predicate names
    /// such as `is_ready` over names like `is_not_ready` or `no_capacity`.
    pub tigerstyle::NEGATED_PREDICATE,
    Warn,
    "negative predicate name"
}

declare_lint_pass!(NegatedPredicate => [NEGATED_PREDICATE]);

const NEGATIVE_PREFIXES: &[&str] = &[
    "is_not_",
    "has_no_",
    "has_not_",
    "lacks_",
    "cannot_",
    "cant_",
    "does_not_",
    "doesnt_",
    "isnt_",
    "wasnt_",
    "wont_",
    "no_",
];

fn has_negative_prefix(name: &str) -> bool {
    NEGATIVE_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
}

fn positive_name(name: &str) -> String {
    const TRANSFORMS: &[(&str, &str)] = &[
        ("is_not_", "is_"),
        ("has_no_", "has_"),
        ("has_not_", "has_"),
        ("lacks_", "has_"),
        ("cannot_", "can_"),
        ("cant_", "can_"),
        ("does_not_", "does_"),
        ("doesnt_", "does_"),
        ("isnt_", "is_"),
        ("wasnt_", "was_"),
        ("wont_", "will_"),
        ("no_", "has_"),
    ];

    for &(neg, pos) in TRANSFORMS {
        if let Some(suffix) = name.strip_prefix(neg) {
            return format!("{pos}{suffix}");
        }
    }

    name.to_owned()
}

fn lint_name(cx: &LateContext<'_>, span: rustc_span::Span, name: &str) {
    let suggested = positive_name(name);
    emit_tiger_style_lint_with_suggestion(
        cx,
        NEGATED_PREDICATE,
        span,
        format!("negated predicate name `{name}` — prefer a positive form like `{suggested}`"),
        "negated_predicate",
        LintSuggestion::Single {
            span,
            message: format!("rename to `{suggested}`"),
            replacement: suggested,
            applicability: Applicability::HasPlaceholders,
        },
    );
}

impl<'tcx> LateLintPass<'tcx> for NegatedPredicate {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { ident, .. } = &item.kind else {
            return;
        };

        let name = ident.name.as_str();
        if name == "not" && !cx.tcx.visibility(item.owner_id.def_id).is_public() {
            return;
        }
        if !has_negative_prefix(name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, ident.span, name);
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(_, _) = impl_item.kind else {
            return;
        };
        let name = impl_item.ident.name.as_str();
        if !has_negative_prefix(name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(impl_item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, impl_item.ident.span, name);
        }
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(_, _) = trait_item.kind else {
            return;
        };
        let name = trait_item.ident.name.as_str();
        if !has_negative_prefix(name) {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(trait_item.owner_id.def_id).instantiate_identity();
        if fn_sig.skip_binder().output().is_bool() {
            lint_name(cx, trait_item.ident.span, name);
        }
    }

    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        let PatKind::Binding(_, binding_hir_id, ident, _) = local.pat.kind else {
            return;
        };

        let name = ident.name.as_str();
        if !has_negative_prefix(name) || !cx.typeck_results().pat_ty(local.pat).is_bool() {
            return;
        }

        let suggested = positive_name(name);
        let suggestion = if ident.span.from_expansion() {
            LintSuggestion::Single {
                span: ident.span,
                message: format!("rename to `{suggested}`"),
                replacement: suggested.clone(),
                applicability: Applicability::MaybeIncorrect,
            }
        } else {
            let body = use_site_collector::enclosing_body(cx, local.hir_id);
            if let Some(use_sites) = use_site_collector::collect_local_use_sites(cx, body, binding_hir_id) {
                let mut parts = vec![(ident.span, suggested.clone())];
                parts.extend(use_sites.into_iter().map(|span| (span, suggested.clone())));
                LintSuggestion::Multipart {
                    message: format!("rename to `{suggested}`"),
                    parts,
                    applicability: Applicability::MachineApplicable,
                }
            } else {
                LintSuggestion::Single {
                    span: ident.span,
                    message: format!("rename to `{suggested}`"),
                    replacement: suggested.clone(),
                    applicability: Applicability::MaybeIncorrect,
                }
            }
        };

        emit_tiger_style_lint_with_suggestion(
            cx,
            NEGATED_PREDICATE,
            ident.span,
            format!("negated predicate name `{name}` — prefer a positive form like `{suggested}`"),
            "negated_predicate",
            suggestion,
        );
    }
}
