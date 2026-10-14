use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "foreign_enum_catch_all";

declare_tool_lint! {
    /// Detects catch-all match arms over enums defined in another crate.
    /// Foreign enums can grow without this crate seeing the new variant at review time.
    pub tigerstyle::FOREIGN_ENUM_CATCH_ALL,
    Allow,
    "catch-all arm on foreign enum match"
}

pub struct ForeignEnumCatchAll;

impl_lint_pass!(ForeignEnumCatchAll => [FOREIGN_ENUM_CATCH_ALL]);

fn enum_type<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<rustc_middle::ty::AdtDef<'tcx>> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return None;
    };
    adt.is_enum().then_some(*adt)
}

impl<'tcx> LateLintPass<'tcx> for ForeignEnumCatchAll {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Match(scrutinee, arms, _) = &expr.kind else {
            return;
        };
        if expr.span.from_expansion() || scrutinee.span.from_expansion() {
            return;
        }
        let Some(adt) = enum_type(cx, scrutinee) else {
            return;
        };
        if adt.did().is_local() {
            return;
        }
        let Some(catch_all) = arms.iter().find(|arm| matches!(arm.pat.kind, PatKind::Wild)) else {
            return;
        };

        emit_tiger_style_lint(
            cx,
            FOREIGN_ENUM_CATCH_ALL,
            catch_all.span,
            format!(
                "foreign enum `{}` uses a `_` catch-all arm; handle new variants explicitly or document the fallback",
                cx.tcx.def_path_str(adt.did())
            ),
            LINT_NAME,
        );
    }
}
