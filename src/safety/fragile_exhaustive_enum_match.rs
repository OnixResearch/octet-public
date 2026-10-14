use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::PatKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "fragile_exhaustive_enum_match";

declare_tool_lint! {
    /// Detects exhaustive matches over same-crate enums without a wildcard arm.
    /// Adding a new variant forces every exhaustive site to change; use this as
    /// an advisory OCP review signal unless the enum is deliberately sealed.
    pub tigerstyle::FRAGILE_EXHAUSTIVE_ENUM_MATCH,
    Allow,
    "same-crate enum matched exhaustively without a fallback"
}

pub struct FragileExhaustiveEnumMatch {
    sealed_enum_marker: String,
    state_machine_enum_marker: String,
}

impl FragileExhaustiveEnumMatch {
    pub fn new(sealed_enum_marker: String, state_machine_enum_marker: String) -> Self {
        Self {
            sealed_enum_marker,
            state_machine_enum_marker,
        }
    }
}

impl_lint_pass!(FragileExhaustiveEnumMatch => [FRAGILE_EXHAUSTIVE_ENUM_MATCH]);

fn enum_type<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<rustc_middle::ty::AdtDef<'tcx>> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).kind() else {
        return None;
    };
    adt.is_enum().then_some(*adt)
}

fn enum_has_marker(cx: &LateContext<'_>, adt: rustc_middle::ty::AdtDef<'_>, marker: &str) -> bool {
    let Some(local_def_id) = adt.did().as_local() else {
        return false;
    };
    let hir_id = cx.tcx.local_def_id_to_hir_id(local_def_id);
    let marker_tail = marker.rsplit("::").next().unwrap_or(marker);
    cx.tcx.hir_attrs(hir_id).iter().any(|attr| {
        let rendered = format!("{attr:?}");
        rendered.contains(marker) || rendered.contains(marker_tail)
    })
}

fn arm_mentions_variant(cx: &LateContext<'_>, arm: &hir::Arm<'_>, variant_name: &str) -> bool {
    cx.sess()
        .source_map()
        .span_to_snippet(arm.pat.span)
        .is_ok_and(|snippet| snippet.contains(variant_name))
}

fn arms_cover_all_variants(cx: &LateContext<'_>, arms: &[hir::Arm<'_>], adt: rustc_middle::ty::AdtDef<'_>) -> bool {
    adt.variants()
        .iter()
        .all(|variant| arms.iter().any(|arm| arm_mentions_variant(cx, arm, variant.name.as_str())))
}

impl<'tcx> LateLintPass<'tcx> for FragileExhaustiveEnumMatch {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Match(scrutinee, arms, _) = &expr.kind else {
            return;
        };
        if expr.span.from_expansion()
            || scrutinee.span.from_expansion()
            || arms.iter().any(|arm| matches!(arm.pat.kind, PatKind::Wild))
        {
            return;
        }
        let Some(adt) = enum_type(cx, scrutinee) else {
            return;
        };
        if !adt.did().is_local()
            || enum_has_marker(cx, adt, &self.sealed_enum_marker)
            || enum_has_marker(cx, adt, &self.state_machine_enum_marker)
        {
            return;
        }
        if !arms_cover_all_variants(cx, arms, adt) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            FRAGILE_EXHAUSTIVE_ENUM_MATCH,
            expr.span,
            format!(
                "same-crate enum `{}` is matched exhaustively without a fallback; adding a variant will force this site to change",
                cx.tcx.def_path_str(adt.did())
            ),
            LINT_NAME,
        );
    }
}
