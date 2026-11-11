use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::BinOpKind;
use rustc_hir::ExprKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects unprotected `+`, `-`, `*` and `+=`, `-=`, `*=` on unsigned
    /// integer types in production code where overflow is not checked.
    /// Verified-logic uses `checked_add`, `saturating_sub`, etc. for every
    /// arithmetic operation.
    pub tigerstyle::RAW_ARITHMETIC_OVERFLOW,
    Allow,
    "unprotected arithmetic on unsigned integers"
}

declare_lint_pass!(RawArithmeticOverflow => [RAW_ARITHMETIC_OVERFLOW]);

fn is_unsigned_int(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Uint(_))
}

/// Only literals, named const items, and primitive expressions composed from
/// them are known here. Associated consts and inline const blocks can depend
/// on generic parameters; `let` bindings can be runtime even if initialized
/// from a const. Keep the lint for all of those opaque operands.
fn is_constant_expr(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Lit(_) => true,
        ExprKind::Path(qpath) => {
            matches!(cx.typeck_results().qpath_res(qpath, expr.hir_id), Res::Def(DefKind::Const { .. }, _))
        }
        ExprKind::Binary(op, lhs, rhs)
            if matches!(
                op.node,
                BinOpKind::Add
                    | BinOpKind::Sub
                    | BinOpKind::Mul
                    | BinOpKind::Div
                    | BinOpKind::Rem
                    | BinOpKind::BitAnd
                    | BinOpKind::BitOr
                    | BinOpKind::BitXor
                    | BinOpKind::Shl
                    | BinOpKind::Shr
            ) && is_integer_ty(cx.typeck_results().expr_ty(expr))
                && is_integer_ty(cx.typeck_results().expr_ty(lhs))
                && is_integer_ty(cx.typeck_results().expr_ty(rhs)) =>
        {
            is_constant_expr(cx, lhs) && is_constant_expr(cx, rhs)
        }
        ExprKind::Unary(hir::UnOp::Neg | hir::UnOp::Not, inner) if is_integer_ty(cx.typeck_results().expr_ty(expr)) => {
            is_constant_expr(cx, inner)
        }
        ExprKind::Cast(inner, _)
            if is_integer_ty(cx.typeck_results().expr_ty(expr))
                && is_integer_ty(cx.typeck_results().expr_ty(inner)) =>
        {
            is_constant_expr(cx, inner)
        }
        ExprKind::DropTemps(inner) => is_constant_expr(cx, inner),
        _ => false,
    }
}

fn is_integer_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

fn saturating_method(op: BinOpKind) -> Option<&'static str> {
    match op {
        BinOpKind::Add => Some("saturating_add"),
        BinOpKind::Sub => Some("saturating_sub"),
        BinOpKind::Mul => Some("saturating_mul"),
        _ => None,
    }
}

/// `a op b` as a saturating call. A constant lhs such as an unsuffixed literal
/// may have no type of its own yet (`(2).saturating_mul(n)` is E0689), so it is
/// spelled through the result type: `u64::saturating_mul(2, n)`.
fn saturating_replacement<'tcx>(
    cx: &LateContext<'tcx>,
    lhs: &hir::Expr<'tcx>,
    rhs: &hir::Expr<'tcx>,
    op: BinOpKind,
    result_ty: ty::Ty<'tcx>,
) -> Option<String> {
    let method = saturating_method(op)?;
    let lhs_snippet = expr_snippet(cx, lhs)?;
    let rhs_snippet = expr_snippet(cx, rhs)?;

    if is_constant_expr(cx, lhs) && is_integer_ty(cx.typeck_results().expr_ty(lhs)) {
        return Some(format!("{result_ty}::{method}({lhs_snippet}, {rhs_snippet})"));
    }
    Some(format!("({lhs_snippet}).{method}({rhs_snippet})"))
}

/// Places that can be spelled twice without repeating side effects: locals,
/// statics, field projections, and dereferences of those.
fn is_side_effect_free_place(expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Path(_) => true,
        ExprKind::Field(base, _) | ExprKind::Unary(hir::UnOp::Deref, base) => is_side_effect_free_place(base),
        _ => false,
    }
}

/// `place op= value` as `place = (place).saturating_op(value)`. Places such as
/// `v[next()]` get no suggestion because the rewrite would evaluate them twice.
fn compound_saturating_replacement(
    cx: &LateContext<'_>,
    place: &hir::Expr<'_>,
    value: &hir::Expr<'_>,
    op: BinOpKind,
) -> Option<String> {
    if !is_side_effect_free_place(place) {
        return None;
    }
    let method = saturating_method(op)?;
    let place = expr_snippet(cx, place)?;
    let value = expr_snippet(cx, value)?;
    Some(format!("{place} = ({place}).{method}({value})"))
}

/// An unsigned `+`, `-` or `*`, either `lhs op rhs` or the compound `lhs op= rhs`.
struct UnsignedArithmetic<'tcx> {
    op: BinOpKind,
    lhs: &'tcx hir::Expr<'tcx>,
    rhs: &'tcx hir::Expr<'tcx>,
    result_ty: ty::Ty<'tcx>,
    compound: bool,
}

fn unsigned_arithmetic<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<UnsignedArithmetic<'tcx>> {
    let (op, lhs, rhs, result_ty, compound) = match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) => (op.node, *lhs, *rhs, cx.typeck_results().expr_ty(expr), false),
        ExprKind::AssignOp(op, lhs, rhs) => {
            let op = match op.node {
                hir::AssignOpKind::AddAssign => BinOpKind::Add,
                hir::AssignOpKind::SubAssign => BinOpKind::Sub,
                hir::AssignOpKind::MulAssign => BinOpKind::Mul,
                _ => return None,
            };
            (op, *lhs, *rhs, cx.typeck_results().expr_ty(lhs), true)
        }
        _ => return None,
    };

    (matches!(op, BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul) && is_unsigned_int(result_ty)).then_some(
        UnsignedArithmetic {
            op,
            lhs,
            rhs,
            result_ty,
            compound,
        },
    )
}

impl<'tcx> LateLintPass<'tcx> for RawArithmeticOverflow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some(UnsignedArithmetic {
            op,
            lhs,
            rhs,
            result_ty,
            compound,
        }) = unsigned_arithmetic(cx, expr)
        else {
            return;
        };

        // Retain checks on runtime locals, even in const fn bodies.
        if is_constant_expr(cx, lhs) && is_constant_expr(cx, rhs) {
            return;
        }

        if crate::test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let op_name = match op {
            BinOpKind::Add => "addition",
            BinOpKind::Sub => "subtraction",
            BinOpKind::Mul => "multiplication",
            _ => "arithmetic",
        };

        let message = format!("unprotected unsigned {op_name} may overflow");
        let replacement = if compound {
            compound_saturating_replacement(cx, lhs, rhs, op)
        } else {
            saturating_replacement(cx, lhs, rhs, op, result_ty)
        };
        if let Some(replacement) = replacement {
            emit_tiger_style_lint_with_suggestion(
                cx,
                RAW_ARITHMETIC_OVERFLOW,
                expr.span,
                message,
                "raw_arithmetic_overflow",
                LintSuggestion::Single {
                    span: expr.span,
                    message: format!("replace with saturating {op_name}"),
                    replacement,
                    applicability: Applicability::MaybeIncorrect,
                },
            );
        } else {
            emit_tiger_style_lint(cx, RAW_ARITHMETIC_OVERFLOW, expr.span, message, "raw_arithmetic_overflow");
        }
    }
}
