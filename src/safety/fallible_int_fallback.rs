use rustc_ast::LitKind;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects `.unwrap_or(0)`, `.unwrap_or(T::MAX)`, `.unwrap_or(T::MIN)`,
    /// and `.unwrap_or_default()` on `Result<integer, TryFromIntError>`.
    /// These patterns hide failed integer conversions behind sentinel values.
    pub tigerstyle::FALLIBLE_INT_FALLBACK,
    Warn,
    "fallible integer conversion with sentinel/default fallback"
}

declare_lint_pass!(FallibleIntFallback => [FALLIBLE_INT_FALLBACK]);

fn result_ok_err_tys<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> Option<(ty::Ty<'tcx>, ty::Ty<'tcx>)> {
    let ty::Adt(adt, args) = ty.kind() else {
        return None;
    };
    if !cx.tcx.is_diagnostic_item(sym::Result, adt.did()) || args.len() < 2 {
        return None;
    }
    Some((args.type_at(0), args.type_at(1)))
}

fn is_integer_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

fn is_try_from_int_error(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty::Adt(adt, _) = ty.kind() else {
        return false;
    };
    matches!(
        cx.tcx.def_path_str(adt.did()).as_str(),
        "core::num::error::TryFromIntError" | "std::num::TryFromIntError"
    )
}

fn is_zero_literal(expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, ExprKind::Lit(lit) if matches!(lit.node, LitKind::Int(value, _) if value == 0))
}

fn is_integer_extreme(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let ExprKind::Path(_) = expr.kind else {
        return false;
    };
    let Ok(snippet) = cx.sess().source_map().span_to_snippet(expr.span) else {
        return false;
    };
    let normalized = snippet.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    normalized.ends_with("::MIN") || normalized.ends_with("::MAX")
}

fn fallback_is_sentinel(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    is_zero_literal(expr) || is_integer_extreme(cx, expr)
}

impl<'tcx> LateLintPass<'tcx> for FallibleIntFallback {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if expr.span.in_external_macro(cx.sess().source_map())
            || test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
        {
            return;
        }

        let ExprKind::MethodCall(method, recv, args, _) = expr.kind else {
            return;
        };
        let method_name = method.ident.name.as_str();
        if method_name != "unwrap_or" && method_name != "unwrap_or_default" {
            return;
        }

        let Some((ok_ty, err_ty)) = result_ok_err_tys(cx, cx.typeck_results().expr_ty(recv)) else {
            return;
        };
        if !is_integer_ty(ok_ty) || !is_try_from_int_error(cx, err_ty) {
            return;
        }

        let fallback_kind = if method_name == "unwrap_or_default" && args.is_empty() {
            "default"
        } else if method_name == "unwrap_or" && args.len() == 1 && fallback_is_sentinel(cx, &args[0]) {
            "sentinel"
        } else {
            return;
        };

        emit_tiger_style_lint(
            cx,
            FALLIBLE_INT_FALLBACK,
            expr.span,
            format!("fallible integer conversion hides an out-of-range error behind a {fallback_kind} fallback"),
            "fallible_int_fallback",
        );
    }
}
