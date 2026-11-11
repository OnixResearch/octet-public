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
    /// Detects `a / b`, `a % b`, `a /= b`, or `a %= b` where `b` is an
    /// integer (or a reference to one) that is a function parameter or local
    /// variable (not a constant) and no guard proves `b != 0` at the
    /// division: an enclosing `if b != 0` / `b > 0` / `b >= 1` then-branch,
    /// an enclosing `if b == 0` else-branch, a `match` arm whose guard implies
    /// `b != 0` or that follows an unguarded `0` arm over `b`, or an earlier
    /// statement in an enclosing block that diverges when `b` is zero (for
    /// example `assert!(b > 0)` or `if b == 0 { return ...; }`).
    pub tigerstyle::UNCHECKED_DIVISION,
    Warn,
    "division by variable without zero check"
}

declare_lint_pass!(UncheckedDivision => [UNCHECKED_DIVISION]);

fn is_integer_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.peel_refs().kind(), ty::Int(_) | ty::Uint(_))
}

fn reference_depth(mut ty: ty::Ty<'_>) -> usize {
    let mut depth = 0;
    while let ty::Ref(_, inner, _) = ty.kind() {
        ty = *inner;
        depth += 1;
    }
    depth
}

/// Strip `*` and `&` so `count`, `*count`, and `&count` name the same divisor.
fn peel_deref_and_borrow<'a, 'hir>(mut expr: &'a hir::Expr<'hir>) -> &'a hir::Expr<'hir> {
    while let ExprKind::Unary(hir::UnOp::Deref, inner) | ExprKind::AddrOf(_, _, inner) = expr.kind {
        expr = inner;
    }
    expr
}

fn is_literal_or_const(expr: &hir::Expr<'_>) -> bool {
    match &peel_deref_and_borrow(expr).kind {
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
    if let ExprKind::Path(hir::QPath::Resolved(_, path)) = &peel_deref_and_borrow(expr).kind {
        path.segments.last().map(|s| s.ident.name.as_str().to_string())
    } else {
        None
    }
}

fn is_int_literal(expr: &hir::Expr<'_>, value: u128) -> bool {
    matches!(&expr.kind, ExprKind::Lit(lit) if matches!(lit.node, rustc_ast::LitKind::Int(v, _) if v.get() == value))
}

fn mirrored_comparison(op: BinOpKind) -> BinOpKind {
    match op {
        BinOpKind::Lt => BinOpKind::Gt,
        BinOpKind::Le => BinOpKind::Ge,
        BinOpKind::Gt => BinOpKind::Lt,
        BinOpKind::Ge => BinOpKind::Le,
        other => other,
    }
}

/// Normalize `lhs op rhs` to `divisor op bound`, if either side is the divisor.
fn divisor_comparison<'hir>(
    op: BinOpKind,
    lhs: &'hir hir::Expr<'hir>,
    rhs: &'hir hir::Expr<'hir>,
    divisor: &hir::Expr<'_>,
) -> Option<(BinOpKind, &'hir hir::Expr<'hir>)> {
    let divisor = peel_deref_and_borrow(divisor);
    if crate::safety::unchecked_narrowing::same_checked_expr(peel_deref_and_borrow(lhs), divisor) {
        Some((op, rhs))
    } else if crate::safety::unchecked_narrowing::same_checked_expr(peel_deref_and_borrow(rhs), divisor) {
        Some((mirrored_comparison(op), lhs))
    } else {
        None
    }
}

fn condition_true_implies_nonzero(cond: &hir::Expr<'_>, divisor: &hir::Expr<'_>) -> bool {
    match &cond.kind {
        ExprKind::Binary(op, lhs, rhs) => match op.node {
            BinOpKind::And => {
                condition_true_implies_nonzero(lhs, divisor) || condition_true_implies_nonzero(rhs, divisor)
            }
            BinOpKind::Or => {
                condition_true_implies_nonzero(lhs, divisor) && condition_true_implies_nonzero(rhs, divisor)
            }
            _ => divisor_comparison(op.node, lhs, rhs, divisor).is_some_and(|(op, bound)| match op {
                BinOpKind::Ne | BinOpKind::Gt => is_int_literal(bound, 0),
                BinOpKind::Ge => is_int_literal(bound, 1),
                _ => false,
            }),
        },
        ExprKind::Unary(hir::UnOp::Not, nested) => condition_false_implies_nonzero(nested, divisor),
        ExprKind::DropTemps(nested) => condition_true_implies_nonzero(nested, divisor),
        ExprKind::Block(block, _) => {
            block.stmts.is_empty() && block.expr.is_some_and(|tail| condition_true_implies_nonzero(tail, divisor))
        }
        _ => false,
    }
}

fn condition_false_implies_nonzero(cond: &hir::Expr<'_>, divisor: &hir::Expr<'_>) -> bool {
    match &cond.kind {
        ExprKind::Binary(op, lhs, rhs) => match op.node {
            BinOpKind::And => {
                condition_false_implies_nonzero(lhs, divisor) && condition_false_implies_nonzero(rhs, divisor)
            }
            BinOpKind::Or => {
                condition_false_implies_nonzero(lhs, divisor) || condition_false_implies_nonzero(rhs, divisor)
            }
            _ => divisor_comparison(op.node, lhs, rhs, divisor).is_some_and(|(op, bound)| match op {
                BinOpKind::Eq | BinOpKind::Le => is_int_literal(bound, 0),
                BinOpKind::Lt => is_int_literal(bound, 1),
                _ => false,
            }),
        },
        ExprKind::Unary(hir::UnOp::Not, nested) => condition_true_implies_nonzero(nested, divisor),
        ExprKind::DropTemps(nested) => condition_false_implies_nonzero(nested, divisor),
        ExprKind::Block(block, _) => {
            block.stmts.is_empty() && block.expr.is_some_and(|tail| condition_false_implies_nonzero(tail, divisor))
        }
        _ => false,
    }
}

/// Check whether evaluating `expr` never completes normally. A block whose tail
/// diverges (such as `{ panic(...) }`) is typed by its coercion target, so look
/// through block statements and tails as well as the expression type.
fn always_diverges(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    if cx.typeck_results().node_type_opt(expr.hir_id).is_some_and(ty::Ty::is_never) {
        return true;
    }

    let ExprKind::Block(block, _) = &expr.kind else {
        return false;
    };
    let stmt_diverges = |stmt: &hir::Stmt<'_>| match stmt.kind {
        hir::StmtKind::Semi(inner) | hir::StmtKind::Expr(inner) => always_diverges(cx, inner),
        hir::StmtKind::Let(_) | hir::StmtKind::Item(_) => false,
    };
    block.stmts.iter().any(stmt_diverges) || block.expr.is_some_and(|tail| always_diverges(cx, tail))
}

/// Check whether an earlier statement of `block` diverges unless the divisor is
/// nonzero, such as `if b == 0 { return ...; }` or `assert!(b > 0)` (which
/// lowers to `if !(b > 0) { panic(...) }`).
fn has_prior_zero_exit(
    cx: &LateContext<'_>,
    block: &hir::Block<'_>,
    division: &hir::Expr<'_>,
    divisor: &hir::Expr<'_>,
) -> bool {
    let division_lo = division.span.source_callsite().lo();
    for stmt in block.stmts {
        if stmt.span.source_callsite().hi() > division_lo {
            break;
        }

        let (hir::StmtKind::Semi(guard) | hir::StmtKind::Expr(guard)) = stmt.kind else {
            continue;
        };
        let ExprKind::If(cond, then_expr, _) = &guard.kind else {
            continue;
        };

        if always_diverges(cx, then_expr) && condition_false_implies_nonzero(cond, divisor) {
            return true;
        }
    }
    false
}

/// Whether `pat` matches the integer literal `0`, alone or as one alternative of
/// an or-pattern.
fn pattern_matches_zero(pat: &hir::Pat<'_>) -> bool {
    match &pat.kind {
        hir::PatKind::Expr(hir::PatExpr {
            kind: hir::PatExprKind::Lit { lit, negated: false },
            ..
        }) => matches!(lit.node, rustc_ast::LitKind::Int(v, _) if v.get() == 0),
        hir::PatKind::Or(alternatives) => alternatives.iter().any(pattern_matches_zero),
        _ => false,
    }
}

/// Whether the divisor names the whole scrutinee value inside `arm`: the
/// scrutinee itself, or a plain binding (`d =>`) of the scrutinee.
fn divisor_is_whole_scrutinee(arm: &hir::Arm<'_>, scrutinee: &hir::Expr<'_>, divisor: &hir::Expr<'_>) -> bool {
    let divisor = peel_deref_and_borrow(divisor);
    if crate::safety::unchecked_narrowing::same_checked_expr(divisor, peel_deref_and_borrow(scrutinee)) {
        return true;
    }
    let hir::PatKind::Binding(_, binding_id, _, None) = arm.pat.kind else {
        return false;
    };
    matches!(
        &divisor.kind,
        ExprKind::Path(hir::QPath::Resolved(_, path)) if path.res == hir::def::Res::Local(binding_id)
    )
}

/// Check whether the match arm whose body holds the division proves the divisor
/// nonzero: its guard implies `divisor != 0`, or an earlier unguarded arm took
/// the scrutinee value `0` and the divisor names the whole scrutinee.
fn arm_proves_nonzero(cx: &LateContext<'_>, arm: &hir::Arm<'_>, divisor: &hir::Expr<'_>) -> bool {
    if arm.guard.is_some_and(|guard| condition_true_implies_nonzero(guard, divisor)) {
        return true;
    }
    let hir::Node::Expr(hir::Expr {
        kind: ExprKind::Match(scrutinee, arms, _),
        ..
    }) = cx.tcx.parent_hir_node(arm.hir_id)
    else {
        return false;
    };
    let earlier_arms = arms.iter().take_while(|earlier| earlier.hir_id != arm.hir_id);
    let zero_excluded = earlier_arms
        .filter(|earlier| earlier.guard.is_none())
        .any(|earlier| pattern_matches_zero(earlier.pat));
    zero_excluded && divisor_is_whole_scrutinee(arm, scrutinee, divisor)
}

/// Check whether a guard proves the divisor nonzero at `division`: an enclosing
/// if-branch whose condition implies `divisor != 0`, a match arm that cannot see
/// a zero divisor, or an earlier statement in an enclosing block that exits when
/// the divisor is zero.
fn has_zero_guard(cx: &LateContext<'_>, division: &hir::Expr<'_>, divisor: &hir::Expr<'_>) -> bool {
    let mut child_id = division.hir_id;
    for (parent_id, node) in cx.tcx.hir_parent_iter(division.hir_id) {
        match node {
            hir::Node::Expr(parent) => {
                if let ExprKind::If(cond, then_expr, else_expr) = &parent.kind {
                    if then_expr.hir_id == child_id && condition_true_implies_nonzero(cond, divisor) {
                        return true;
                    }
                    if else_expr.is_some_and(|branch| branch.hir_id == child_id)
                        && condition_false_implies_nonzero(cond, divisor)
                    {
                        return true;
                    }
                }
            }
            hir::Node::Arm(arm) if arm.body.hir_id == child_id && arm_proves_nonzero(cx, arm, divisor) => {
                return true;
            }
            hir::Node::Block(block) if has_prior_zero_exit(cx, block, division, divisor) => return true,
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
        child_id = parent_id;
    }
    false
}

fn expr_snippet(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    cx.sess().source_map().span_to_snippet(expr.span).ok()
}

fn guarded_division_replacement(
    cx: &LateContext<'_>,
    lhs: &hir::Expr<'_>,
    rhs: &hir::Expr<'_>,
    op: &str,
    divisor_ref_depth: usize,
) -> Option<String> {
    let lhs = expr_snippet(cx, lhs)?;
    let rhs = expr_snippet(cx, rhs)?;
    let deref = "*".repeat(divisor_ref_depth);

    Some(format!(
        "{{ let __tigerstyle_divisor = {rhs}; assert!({deref}__tigerstyle_divisor != 0); {lhs} {op} __tigerstyle_divisor }}"
    ))
}

/// Split `a / b`, `a % b`, `a /= b`, and `a %= b` into dividend, divisor, and operator token.
fn integer_division_operands<'tcx>(
    expr: &'tcx hir::Expr<'tcx>,
) -> Option<(&'tcx hir::Expr<'tcx>, &'tcx hir::Expr<'tcx>, &'static str)> {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, BinOpKind::Div | BinOpKind::Rem) => {
            Some((lhs, rhs, op.node.as_str()))
        }
        ExprKind::AssignOp(op, lhs, rhs)
            if matches!(op.node, hir::AssignOpKind::DivAssign | hir::AssignOpKind::RemAssign) =>
        {
            Some((lhs, rhs, op.node.as_str()))
        }
        _ => None,
    }
}

impl<'tcx> LateLintPass<'tcx> for UncheckedDivision {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some((lhs, rhs, op)) = integer_division_operands(expr) else {
            return;
        };

        let rhs_ty = cx.typeck_results().expr_ty(rhs);
        if !is_integer_ty(rhs_ty) {
            return;
        }

        if is_literal_or_const(rhs) {
            return;
        }

        // Skip if there's a zero-guard for this divisor
        if has_zero_guard(cx, expr, rhs) {
            return;
        }

        let name = expr_name(rhs).unwrap_or_else(|| "expression".into());

        let message = format!("division by `{name}` without zero check");
        if let Some(replacement) = guarded_division_replacement(cx, lhs, rhs, op, reference_depth(rhs_ty)) {
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
