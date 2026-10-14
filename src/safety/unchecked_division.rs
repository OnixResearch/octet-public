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
    /// Detects `a / b` or `a % b` where `b` is a function parameter or
    /// local variable (not a constant) and there is no prior zero-guard
    /// in the function. Every division in verified-logic has an explicit
    /// `requires b > 0` precondition.
    pub tigerstyle::UNCHECKED_DIVISION,
    Warn,
    "division by variable without zero check"
}

declare_lint_pass!(UncheckedDivision => [UNCHECKED_DIVISION]);

fn is_integer_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(_) | ty::Uint(_))
}

fn is_literal_or_const(expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Lit(_) => true,
        ExprKind::Path(hir::QPath::Resolved(_, path)) => {
            matches!(
                path.res,
                hir::def::Res::Def(hir::def::DefKind::Const { .. } | hir::def::DefKind::AssocConst { .. }, _)
            )
        }
        _ => false,
    }
}

/// Get the name of a simple path expression (variable or parameter name).
fn expr_name(expr: &hir::Expr<'_>) -> Option<String> {
    if let ExprKind::Path(hir::QPath::Resolved(_, path)) = &expr.kind {
        path.segments.last().map(|s| s.ident.name.as_str().to_string())
    } else {
        None
    }
}

/// Check if there's a prior zero-guard for the divisor in the parent chain.
/// Looks for patterns like:
///   - `assert!(b > 0)` / `assert!(b != 0)` / `assert_ne!(b, 0)`
///   - `if b == 0 { return ... }`
///   - `if b > 0 { ... division ... }`
fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

fn guarded_division_replacement(
    cx: &LateContext<'_>,
    lhs: &hir::Expr<'_>,
    rhs: &hir::Expr<'_>,
    op: BinOpKind,
) -> Option<String> {
    let lhs = expr_snippet(cx, lhs)?;
    let rhs = expr_snippet(cx, rhs)?;
    let op = match op {
        BinOpKind::Div => "/",
        BinOpKind::Rem => "%",
        _ => return None,
    };

    Some(format!(
        "{{ let __tigerstyle_divisor = {rhs}; assert!(__tigerstyle_divisor != 0); {lhs} {op} __tigerstyle_divisor }}"
    ))
}

fn has_zero_guard(cx: &LateContext<'_>, expr: &hir::Expr<'_>, divisor_name: &str) -> bool {
    // Check parent chain for an enclosing if-guard
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Expr(parent) => {
                if let ExprKind::If(cond, _, _) = &parent.kind {
                    let source_map = cx.sess().source_map();
                    if let Ok(snippet) = source_map.span_to_snippet(cond.span) {
                        // Check if condition references the divisor with a zero/nonzero check
                        if snippet.contains(divisor_name)
                            && (snippet.contains("> 0")
                                || snippet.contains("!= 0")
                                || snippet.contains("== 0")
                                || snippet.contains(">= 1")
                                || snippet.contains(".is_some()"))
                        {
                            return true;
                        }
                    }
                }
                // Check enclosing block for preceding assert statements
                if let ExprKind::Block(block, _) = &parent.kind {
                    let source_map = cx.sess().source_map();
                    if let Ok(snippet) = source_map.span_to_snippet(block.span)
                        && snippet.contains(divisor_name)
                        && (snippet.contains("assert!")
                            || snippet.contains("assert_ne!")
                            || snippet.contains("requires"))
                        && (snippet.contains("> 0") || snippet.contains("!= 0") || snippet.contains(">= 1"))
                    {
                        return true;
                    }
                }
            }
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for UncheckedDivision {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Binary(op, lhs, rhs) = &expr.kind else {
            return;
        };

        if !matches!(op.node, BinOpKind::Div | BinOpKind::Rem) {
            return;
        }

        let rhs_ty = cx.typeck_results().expr_ty(rhs);
        if !is_integer_ty(rhs_ty) {
            return;
        }

        if is_literal_or_const(rhs) {
            return;
        }

        let name = expr_name(rhs).unwrap_or_else(|| "expression".into());

        // Skip if there's a zero-guard for this divisor
        if has_zero_guard(cx, expr, &name) {
            return;
        }

        let message = format!("division by `{name}` without zero check");
        if let Some(replacement) = guarded_division_replacement(cx, lhs, rhs, op.node) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                UNCHECKED_DIVISION,
                expr.span,
                message,
                "unchecked_division",
                LintSuggestion::Single {
                    span: expr.span,
                    message: format!("guard `{name}` before division"),
                    replacement,
                    applicability: Applicability::MaybeIncorrect,
                },
            );
        } else {
            emit_tiger_style_lint(cx, UNCHECKED_DIVISION, expr.span, message, "unchecked_division");
        }
    }
}
