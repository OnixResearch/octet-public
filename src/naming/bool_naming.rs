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
    /// Detects `let` bindings of type `bool` whose name does not start with
    /// a predicate prefix (`is_`, `has_`, `should_`, `can_`, `was_`, `will_`,
    /// `needs_`). Boolean variables without such prefixes read ambiguously —
    /// `active` could be a flag, a count, an enum variant.
    pub tigerstyle::BOOL_NAMING,
    Warn,
    "boolean binding should have a predicate prefix"
}

declare_lint_pass!(BoolNaming => [BOOL_NAMING]);

const BOOL_PREFIXES: &[&str] = &["is_", "has_", "should_", "can_", "was_", "will_", "needs_"];

fn suggested_bool_name(name: &str) -> String {
    format!("is_{name}")
}

impl<'tcx> LateLintPass<'tcx> for BoolNaming {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        // Only check simple ident bindings, not destructuring
        let PatKind::Binding(_, _, ident, _) = &local.pat.kind else {
            return;
        };

        let name = ident.name.as_str();

        // Skip underscore-prefixed bindings
        if name.starts_with('_') {
            return;
        }

        // Skip if already has a predicate prefix
        if BOOL_PREFIXES.iter().any(|p| name.starts_with(p)) {
            return;
        }

        // Check if the binding type is bool
        let ty = cx.typeck_results().pat_ty(local.pat);
        if !ty.is_bool() {
            return;
        }

        let suggested_name = suggested_bool_name(name);
        let PatKind::Binding(_, binding_hir_id, _, _) = local.pat.kind else {
            return;
        };

        let suggestion = if ident.span.from_expansion() {
            LintSuggestion::Single {
                span: ident.span,
                message: format!("rename binding to `{suggested_name}`"),
                replacement: suggested_name.clone(),
                applicability: Applicability::MaybeIncorrect,
            }
        } else {
            let body = use_site_collector::enclosing_body(cx, local.hir_id);
            if let Some(use_sites) = use_site_collector::collect_local_use_sites(cx, body, binding_hir_id) {
                let mut parts = vec![(ident.span, suggested_name.clone())];
                parts.extend(use_sites.into_iter().map(|span| (span, suggested_name.clone())));
                LintSuggestion::Multipart {
                    message: format!("rename binding to `{suggested_name}`"),
                    parts,
                    applicability: Applicability::MachineApplicable,
                }
            } else {
                LintSuggestion::Single {
                    span: ident.span,
                    message: format!("rename binding to `{suggested_name}`"),
                    replacement: suggested_name,
                    applicability: Applicability::MaybeIncorrect,
                }
            }
        };

        emit_tiger_style_lint_with_suggestion(
            cx,
            BOOL_NAMING,
            ident.span,
            format!("boolean binding `{name}` should have a predicate prefix"),
            "bool_naming",
            suggestion,
        );
    }
}
