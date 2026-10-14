use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects `Default::default()` / `T::default()` calls that rely on
    /// defaults defined in a different crate. Tiger Style prefers explicit
    /// construction at review boundaries instead of silently inheriting
    /// foreign defaults that can change across versions.
    pub tigerstyle::EXPLICIT_DEFAULTS,
    Warn,
    "implicit default on external type"
}

declare_lint_pass!(ExplicitDefaults => [EXPLICIT_DEFAULTS]);

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn resolved_callee_def_id(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> Option<rustc_hir::def_id::DefId> {
    let ExprKind::Path(qpath) = &callee.kind else {
        return None;
    };

    match cx.typeck_results().qpath_res(qpath, callee.hir_id) {
        Res::Def(_, did) => Some(did),
        _ => None,
    }
}

fn is_default_trait_function(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> bool {
    match_def_path(cx, did, &["core", "default", "Default", "default"])
        || match_def_path(cx, did, &["std", "default", "Default", "default"])
}

fn foreign_adt_type(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return false;
    };

    !adt.did().is_local()
}

impl<'tcx> LateLintPass<'tcx> for ExplicitDefaults {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let ExprKind::Call(callee, args) = &expr.kind else {
            return;
        };
        if !args.is_empty() {
            return;
        }

        let Some(did) = resolved_callee_def_id(cx, callee) else {
            return;
        };
        if !is_default_trait_function(cx, did) || !foreign_adt_type(cx, expr) {
            return;
        }

        let message = "implicit default on external type — construct with explicit values";
        if let Some(suggestion) = explicit_default_suggestion(cx, expr) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                EXPLICIT_DEFAULTS,
                expr.span,
                message,
                "explicit_defaults",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, EXPLICIT_DEFAULTS, expr.span, message, "explicit_defaults");
        }
    }
}

fn explicit_default_suggestion(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<LintSuggestion> {
    let ty = cx.typeck_results().expr_ty(expr);
    let ty::Adt(adt, _) = ty.kind() else {
        return None;
    };

    let type_name = cx.tcx.def_path_str(adt.did());
    let replacement = format!("{type_name}::new(/* fields */)");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "use explicit constructor instead of `Default::default()`".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}
