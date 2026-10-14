use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::BinOpKind;
use rustc_hir::ExprKind;
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
    /// Detects unprotected `+`, `-`, `*` on unsigned integer types in
    /// production code where overflow is not checked. Verified-logic uses
    /// `checked_add`, `saturating_sub`, etc. for every arithmetic operation.
    pub tigerstyle::RAW_ARITHMETIC_OVERFLOW,
    Allow,
    "unprotected arithmetic on unsigned integers"
}

declare_lint_pass!(RawArithmeticOverflow => [RAW_ARITHMETIC_OVERFLOW]);

fn is_unsigned_int(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Uint(_))
}

fn is_constant_expr(expr: &hir::Expr<'_>) -> bool {
    matches!(expr.kind, ExprKind::Lit(_))
}

fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

fn saturating_replacement(
    cx: &LateContext<'_>,
    lhs: &hir::Expr<'_>,
    rhs: &hir::Expr<'_>,
    op: BinOpKind,
) -> Option<String> {
    let lhs = expr_snippet(cx, lhs)?;
    let rhs = expr_snippet(cx, rhs)?;
    let method = match op {
        BinOpKind::Add => "saturating_add",
        BinOpKind::Sub => "saturating_sub",
        BinOpKind::Mul => "saturating_mul",
        _ => return None,
    };

    Some(format!("({lhs}).{method}({rhs})"))
}

impl<'tcx> LateLintPass<'tcx> for RawArithmeticOverflow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Binary(op, lhs, rhs) = &expr.kind else {
            return;
        };

        if !matches!(op.node, BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul) {
            return;
        }

        let ty = cx.typeck_results().expr_ty(expr);
        if !is_unsigned_int(ty) {
            return;
        }

        // Skip if both operands are constants (compiler checks these)
        if is_constant_expr(lhs) && is_constant_expr(rhs) {
            return;
        }

        let op_name = match op.node {
            BinOpKind::Add => "addition",
            BinOpKind::Sub => "subtraction",
            BinOpKind::Mul => "multiplication",
            _ => "arithmetic",
        };

        let message = format!("unprotected unsigned {op_name} may overflow");
        if let Some(replacement) = saturating_replacement(cx, lhs, rhs, op.node) {
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
