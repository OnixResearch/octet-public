use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::walk_expr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `as usize` and `as isize` casts from fixed-width integer
    /// types. On a 32-bit platform, `u64 as usize` silently truncates.
    /// Tiger Style mandates explicitly sized types for cross-platform
    /// consistency.
    pub tigerstyle::PLATFORM_DEPENDENT_CAST,
    Warn,
    "`as usize`/`as isize` from fixed-width integer is platform-dependent"
}

declare_lint_pass!(PlatformDependentCast => [PLATFORM_DEPENDENT_CAST]);

fn is_fixed_width_int(ty: ty::Ty<'_>) -> bool {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width().is_some(),
        ty::Uint(uint_ty) => uint_ty.bit_width().is_some(),
        _ => false,
    }
}

fn is_platform_sized(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Int(rustc_middle::ty::IntTy::Isize) | ty::Uint(rustc_middle::ty::UintTy::Usize))
}

fn ty_name(ty: ty::Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

/// `From` impls that std provides on every platform, so the conversion is
/// lossless and needs no failure handling.
fn has_lossless_from(source_ty: ty::Ty<'_>, target_ty: ty::Ty<'_>) -> bool {
    match target_ty.kind() {
        ty::Uint(ty::UintTy::Usize) => matches!(source_ty.kind(), ty::Uint(ty::UintTy::U8 | ty::UintTy::U16)),
        ty::Int(ty::IntTy::Isize) => {
            matches!(source_ty.kind(), ty::Int(ty::IntTy::I8 | ty::IntTy::I16) | ty::Uint(ty::UintTy::U8))
        }
        _ => false,
    }
}

fn platform_cast_suggestion<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &hir::Expr<'_>,
    inner: &hir::Expr<'_>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> Option<LintSuggestion> {
    crate::cast_suggestion::cast_conversion_suggestion(
        cx,
        expr,
        inner,
        source_ty,
        target_ty,
        has_lossless_from(source_ty, target_ty),
    )
}

/// `a` and `b` name the same local place (`x`, `x.field`, `*x`).
fn same_place(a: &hir::Expr<'_>, b: &hir::Expr<'_>) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Path(hir::QPath::Resolved(None, a_path)), ExprKind::Path(hir::QPath::Resolved(None, b_path))) => {
            matches!(a_path.res, Res::Local(_)) && a_path.res == b_path.res
        }
        (ExprKind::Field(a_base, a_field), ExprKind::Field(b_base, b_field)) => {
            a_field.name == b_field.name && same_place(a_base, b_base)
        }
        (ExprKind::Unary(hir::UnOp::Deref, a_inner), ExprKind::Unary(hir::UnOp::Deref, b_inner)) => {
            same_place(a_inner, b_inner)
        }
        _ => false,
    }
}

/// `a` and `b` are the same cast of the same place to the same type.
fn same_cast(cx: &LateContext<'_>, a: &hir::Expr<'_>, b: &hir::Expr<'_>) -> bool {
    let (ExprKind::Cast(a_inner, _), ExprKind::Cast(b_inner, _)) = (&a.kind, &b.kind) else {
        return false;
    };
    let typeck = cx.typeck_results();
    same_place(a_inner, b_inner)
        && typeck.expr_ty(a) == typeck.expr_ty(b)
        && typeck.expr_ty(a_inner) == typeck.expr_ty(b_inner)
}

fn is_len_of(expr: &hir::Expr<'_>, base: &hir::Expr<'_>) -> bool {
    matches!(&expr.kind, ExprKind::MethodCall(segment, receiver, [], _)
        if segment.ident.name == sym::len && same_place(receiver, base))
}

fn negated_comparison(op: hir::BinOpKind) -> Option<hir::BinOpKind> {
    match op {
        hir::BinOpKind::Lt => Some(hir::BinOpKind::Ge),
        hir::BinOpKind::Le => Some(hir::BinOpKind::Gt),
        hir::BinOpKind::Gt => Some(hir::BinOpKind::Le),
        hir::BinOpKind::Ge => Some(hir::BinOpKind::Lt),
        _ => None,
    }
}

/// Whether `cond` evaluating to `holds` proves `cast < base.len()` for a cast
/// accepted by `is_guard_cast`.
fn condition_bounds_index(
    cond: &hir::Expr<'_>,
    holds: bool,
    base: &hir::Expr<'_>,
    is_guard_cast: &dyn Fn(&hir::Expr<'_>) -> bool,
) -> bool {
    match &cond.kind {
        ExprKind::DropTemps(inner) => condition_bounds_index(inner, holds, base, is_guard_cast),
        ExprKind::Unary(hir::UnOp::Not, inner) => condition_bounds_index(inner, !holds, base, is_guard_cast),
        ExprKind::Block(block, _) if block.stmts.is_empty() => {
            block.expr.is_some_and(|tail| condition_bounds_index(tail, holds, base, is_guard_cast))
        }
        ExprKind::Binary(op, lhs, rhs) => match (op.node, holds) {
            // `a && b` holding (or `a || b` failing) means both sides hold (fail).
            (hir::BinOpKind::And, true) | (hir::BinOpKind::Or, false) => {
                condition_bounds_index(lhs, holds, base, is_guard_cast)
                    || condition_bounds_index(rhs, holds, base, is_guard_cast)
            }
            (op, holds) => {
                let op = if holds { Some(op) } else { negated_comparison(op) };
                match op {
                    Some(hir::BinOpKind::Lt) => is_guard_cast(lhs) && is_len_of(rhs, base),
                    Some(hir::BinOpKind::Gt) => is_guard_cast(rhs) && is_len_of(lhs, base),
                    _ => false,
                }
            }
        },
        _ => false,
    }
}

/// Whether an `assert!` statement in `block` before `child` proves the bound.
fn prior_assert_bounds_index(
    cx: &LateContext<'_>,
    block: &hir::Block<'_>,
    child: hir::HirId,
    base: &hir::Expr<'_>,
    is_guard_cast: &dyn Fn(&hir::Expr<'_>) -> bool,
) -> bool {
    for stmt in block.stmts {
        if stmt.hir_id == child {
            return false;
        }
        let (hir::StmtKind::Semi(assert_expr) | hir::StmtKind::Expr(assert_expr)) = stmt.kind else {
            continue;
        };
        let ExprKind::If(cond, _, _) = &assert_expr.kind else {
            continue;
        };
        let is_assert = assert_expr
            .span
            .ctxt()
            .outer_expn_data()
            .macro_def_id
            .is_some_and(|macro_id| cx.tcx.is_diagnostic_item(sym::assert_macro, macro_id));
        // `assert!(cond)` lowers to `if !cond { panic }`; later code sees the `if` condition fail.
        if is_assert && condition_bounds_index(cond, false, base, is_guard_cast) {
            return true;
        }
    }
    false
}

/// Whether `index` (`base[cast]`) only runs after a check proving
/// `cast < base.len()`: an enclosing `if`, the left side of an enclosing `&&`,
/// or a prior `assert!` in an enclosing block.
fn index_is_len_bounded(
    cx: &LateContext<'_>,
    index: &hir::Expr<'_>,
    base: &hir::Expr<'_>,
    is_guard_cast: &dyn Fn(&hir::Expr<'_>) -> bool,
) -> bool {
    let mut child = index.hir_id;
    for (parent_id, node) in cx.tcx.hir_parent_iter(index.hir_id) {
        let bounded = match node {
            hir::Node::Expr(parent) => match &parent.kind {
                ExprKind::If(cond, then_branch, else_branch) => {
                    (then_branch.hir_id == child && condition_bounds_index(cond, true, base, is_guard_cast))
                        || (else_branch.is_some_and(|branch| branch.hir_id == child)
                            && condition_bounds_index(cond, false, base, is_guard_cast))
                }
                ExprKind::Binary(op, lhs, rhs) if op.node == hir::BinOpKind::And && rhs.hir_id == child => {
                    condition_bounds_index(lhs, true, base, is_guard_cast)
                }
                _ => false,
            },
            hir::Node::Block(block) => prior_assert_bounds_index(cx, block, child, base, is_guard_cast),
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => return false,
            _ => false,
        };
        if bounded {
            return true;
        }
        child = parent_id;
    }
    false
}

/// Finds an index `len_receiver[cast']` with `cast'` matching `guard_cast`
/// that is bounded by the comparison containing `guard_cast` itself.
struct GuardedIndexFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    guard_cast: &'tcx hir::Expr<'tcx>,
    len_receiver: &'tcx hir::Expr<'tcx>,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for GuardedIndexFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }
        if let ExprKind::Index(base, index, _) = &expr.kind
            && same_place(base, self.len_receiver)
            && same_cast(self.cx, index, self.guard_cast)
            && index_is_len_bounded(self.cx, expr, base, &|candidate| candidate.hir_id == self.guard_cast.hir_id)
        {
            self.found = true;
            return;
        }
        walk_expr(self, expr);
    }
}

/// The spec exempts len-bounded indexing: `data[x as usize]` after a check
/// such as `assert!((x as usize) < data.len())`, and the cast inside that
/// check when it guards such an index.
fn is_len_bounded_index_cast<'tcx>(cx: &LateContext<'tcx>, cast: &'tcx hir::Expr<'tcx>) -> bool {
    let hir::Node::Expr(parent) = cx.tcx.parent_hir_node(cast.hir_id) else {
        return false;
    };
    match &parent.kind {
        ExprKind::Index(base, index, _) if index.hir_id == cast.hir_id => {
            index_is_len_bounded(cx, parent, base, &|candidate| same_cast(cx, candidate, cast))
        }
        ExprKind::Binary(op, lhs, rhs) if negated_comparison(op.node).is_some() => {
            let len_call = if lhs.hir_id == cast.hir_id { rhs } else { lhs };
            let ExprKind::MethodCall(segment, len_receiver, [], _) = &len_call.kind else {
                return false;
            };
            let Some(body) = cx.enclosing_body else {
                return false;
            };
            if segment.ident.name != sym::len {
                return false;
            }
            let mut finder = GuardedIndexFinder {
                cx,
                guard_cast: cast,
                len_receiver,
                found: false,
            };
            finder.visit_expr(cx.tcx.hir_body(body).value);
            finder.found
        }
        _ => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for PlatformDependentCast {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        // Fire when casting from fixed-width to platform-sized
        if is_fixed_width_int(source_ty) && is_platform_sized(target_ty) && !is_len_bounded_index_cast(cx, expr) {
            let source_name = ty_name(source_ty);
            let target_name = ty_name(target_ty);

            let message = format!("`as {target_name}` from `{source_name}` is platform-dependent");
            if let Some(suggestion) = platform_cast_suggestion(cx, expr, inner, source_ty, target_ty) {
                emit_tiger_style_lint_with_suggestion(
                    cx,
                    PLATFORM_DEPENDENT_CAST,
                    expr.span,
                    message,
                    "platform_dependent_cast",
                    suggestion,
                );
            } else {
                emit_tiger_style_lint(cx, PLATFORM_DEPENDENT_CAST, expr.span, message, "platform_dependent_cast");
            }
        }
    }
}
