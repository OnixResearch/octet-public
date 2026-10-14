use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects `unwrap_or` / `unwrap_or_else` calls on `Option` and `Result`
    /// that replace missing or failing state with an integer extreme sentinel
    /// such as `u64::MAX` or `i64::MIN`. Tiger Style prefers making invalid
    /// state explicit instead of smuggling it through a fake-but-valid number.
    pub tigerstyle::SENTINEL_FALLBACK,
    Warn,
    "extreme sentinel fallback masks absence or failure"
}

declare_lint_pass!(SentinelFallback => [SENTINEL_FALLBACK]);

fn is_option_or_result(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty::Adt(adt, _) = ty.peel_refs().kind() else {
        return false;
    };

    let did = adt.did();
    cx.tcx.is_diagnostic_item(sym::Option, did) || cx.tcx.is_diagnostic_item(sym::Result, did)
}

fn fallback_expr_for_method_call<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
) -> Option<&'tcx hir::Expr<'tcx>> {
    let ExprKind::MethodCall(segment, _receiver, args, _) = &expr.kind else {
        return None;
    };

    match (segment.ident.name.as_str(), args) {
        ("unwrap_or", [fallback]) => Some(fallback),
        ("unwrap_or_else", [closure_expr]) => closure_tail_expr(cx, closure_expr),
        _ => None,
    }
}

fn closure_tail_expr<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let ExprKind::Closure(closure) = &expr.kind else {
        return None;
    };

    let body = cx.tcx.hir_body(closure.body);
    let value = body.value;
    match &value.kind {
        ExprKind::Block(block, _) => block.expr,
        _ => Some(value),
    }
}

fn direct_integer_extreme_const<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    result_ty: ty::Ty<'tcx>,
) -> bool {
    let ExprKind::Path(qpath) = &expr.kind else {
        return false;
    };

    let hir::QPath::TypeRelative(base_ty, segment) = qpath else {
        return false;
    };
    if segment.ident.name != Symbol::intern("MAX") && segment.ident.name != Symbol::intern("MIN") {
        return false;
    }

    let Res::Def(_, did) = cx.typeck_results().qpath_res(qpath, expr.hir_id) else {
        return false;
    };

    let assoc = cx.tcx.associated_item(did);
    if assoc.container != ty::AssocContainer::InherentImpl {
        return false;
    }

    let Some(impl_did) = assoc.impl_container(cx.tcx) else {
        return false;
    };
    let impl_self_ty = cx.tcx.type_of(impl_did).instantiate_identity();
    let fallback_ty = cx.typeck_results().expr_ty(expr);

    matches!(base_ty.kind, hir::TyKind::Path(hir::QPath::Resolved(_, path)) if matches!(path.res, Res::PrimTy(hir::PrimTy::Int(_) | hir::PrimTy::Uint(_))))
        && matches!(impl_self_ty.kind(), ty::Int(_) | ty::Uint(_))
        && fallback_ty == result_ty
        && impl_self_ty == fallback_ty
}

impl<'tcx> LateLintPass<'tcx> for SentinelFallback {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let ExprKind::MethodCall(segment, receiver, _args, _) = &expr.kind else {
            return;
        };
        let method_name = segment.ident.name.as_str();
        if method_name != "unwrap_or" && method_name != "unwrap_or_else" {
            return;
        }

        if !is_option_or_result(cx, cx.typeck_results().expr_ty(receiver)) {
            return;
        }

        let Some(fallback) = fallback_expr_for_method_call(cx, expr) else {
            return;
        };

        if !direct_integer_extreme_const(cx, fallback, cx.typeck_results().expr_ty(expr)) {
            return;
        }

        let message = "extreme sentinel fallback masks absence or failure";
        if let Some(suggestion) = sentinel_suggestion(cx, expr, receiver) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                SENTINEL_FALLBACK,
                expr.span,
                message,
                "sentinel_fallback",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, SENTINEL_FALLBACK, expr.span, message, "sentinel_fallback");
        }
    }
}

fn sentinel_suggestion(cx: &LateContext<'_>, expr: &hir::Expr<'_>, receiver: &hir::Expr<'_>) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let receiver_snippet = sm.span_to_snippet(receiver.span).ok()?;
    let replacement = format!("{receiver_snippet}?");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "propagate the error instead of masking with sentinel".to_owned(),
        replacement,
        applicability: Applicability::MaybeIncorrect,
    })
}
