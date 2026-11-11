use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty::Ty;
use rustc_middle::ty::{self};
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `as u32`, `as u16`, `as u8`, `as i32`, `as i16`, `as i8`
    /// casts from a wider integer type, including `usize` and `isize`,
    /// without a prior bounds check.
    /// Unchecked narrowing casts silently truncate — use `TryFrom` or
    /// a checked conversion function.
    pub tigerstyle::UNCHECKED_NARROWING,
    Warn,
    "narrowing integer cast without bounds check"
}

declare_lint_pass!(UncheckedNarrowing => [UNCHECKED_NARROWING]);

/// Width assumed for `usize`/`isize` sources. Findings must not depend on the
/// compilation target, so pointer-sized sources use the widest pointer width
/// Rust supports (64 bits): `len() as u32` truncates on 64-bit targets.
const POINTER_SIZED_SOURCE_BITS: u64 = 64;

/// Width of a fixed-width integer type; `None` for `usize`/`isize`, whose
/// width depends on the target.
fn int_width(ty: Ty<'_>) -> Option<u64> {
    match ty.kind() {
        ty::Int(int_ty) => int_ty.bit_width(),
        ty::Uint(uint_ty) => uint_ty.bit_width(),
        _ => None,
    }
}

/// Width of a cast source: fixed-width integers use their own width, and
/// `usize`/`isize` use [`POINTER_SIZED_SOURCE_BITS`].
fn source_int_width(ty: Ty<'_>) -> Option<u64> {
    match ty.kind() {
        ty::Int(ty::IntTy::Isize) | ty::Uint(ty::UintTy::Usize) => Some(POINTER_SIZED_SOURCE_BITS),
        _ => int_width(ty),
    }
}

fn ty_name(ty: Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

/// Check whether `expr` spells `<target>::<const_name>`, e.g. `u32::MAX`.
fn is_target_const_expr<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &hir::Expr<'tcx>,
    target_ty: Ty<'tcx>,
    const_name: &str,
) -> bool {
    let ExprKind::Path(_) = &expr.kind else {
        return false;
    };

    let Ok(snippet) = cx.sess().source_map().span_to_snippet(expr.span) else {
        return false;
    };

    let normalized = snippet.chars().filter(|ch| !ch.is_whitespace()).collect::<String>();
    let target_name = ty_name(target_ty);

    normalized == format!("{target_name}::{const_name}") || normalized == format!("<{target_name}>::{const_name}")
}

/// Check whether two expressions name the same local/path (by resolution) or
/// the same field projection of one.
pub(crate) fn same_checked_expr(lhs: &hir::Expr<'_>, rhs: &hir::Expr<'_>) -> bool {
    match (&lhs.kind, &rhs.kind) {
        (ExprKind::Path(hir::QPath::Resolved(_, lhs_path)), ExprKind::Path(hir::QPath::Resolved(_, rhs_path))) => {
            lhs_path.res == rhs_path.res
        }
        (ExprKind::Field(lhs_base, lhs_ident), ExprKind::Field(rhs_base, rhs_ident)) => {
            lhs_ident.name == rhs_ident.name && same_checked_expr(lhs_base, rhs_base)
        }
        _ => false,
    }
}

fn span_contains(outer: Span, inner: Span) -> bool {
    let outer = outer.source_callsite();
    let inner = inner.source_callsite();
    outer.lo() <= inner.lo() && inner.hi() <= outer.hi()
}

/// Check whether `expr` is `<target>::<const_name> as <source>`.
fn is_target_const_cast_to_source<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
    const_name: &str,
) -> bool {
    matches!(&expr.kind, ExprKind::Cast(const_expr, _) if cx.typeck_results().expr_ty(expr) == source_ty
        && cx.typeck_results().expr_ty(const_expr) == target_ty
        && is_target_const_expr(cx, const_expr, target_ty, const_name))
}

fn is_zero_literal(expr: &hir::Expr<'_>) -> bool {
    matches!(&expr.kind, ExprKind::Lit(lit) if matches!(lit.node, rustc_ast::LitKind::Int(value, _) if value.get() == 0))
}

/// Check whether `expr` is a lower bound that keeps the cast in range: `0` or
/// `<target>::MIN as <source>`.
fn is_target_lower_bound<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> bool {
    is_zero_literal(expr) || is_target_const_cast_to_source(cx, expr, source_ty, target_ty, "MIN")
}

/// Range facts proven about the casted expression.
#[derive(Clone, Copy)]
struct BoundFacts {
    /// The value is at most `<target>::MAX`.
    upper: bool,
    /// The value is at least `<target>::MIN`.
    lower: bool,
}

impl BoundFacts {
    const BOTH: Self = Self {
        upper: true,
        lower: true,
    };
    const NONE: Self = Self {
        upper: false,
        lower: false,
    };

    fn union(self, other: Self) -> Self {
        Self {
            upper: self.upper || other.upper,
            lower: self.lower || other.lower,
        }
    }

    fn intersection(self, other: Self) -> Self {
        Self {
            upper: self.upper && other.upper,
            lower: self.lower && other.lower,
        }
    }

    /// Unsigned sources are never below `<target>::MIN`; signed sources also
    /// need a proven lower bound or negative values wrap.
    fn proves_safe_narrowing(self, source_ty: Ty<'_>) -> bool {
        self.upper && (self.lower || !source_ty.is_signed())
    }
}

/// `a op b` rewritten as `b op' a`.
fn mirrored_comparison(op: hir::BinOpKind) -> hir::BinOpKind {
    match op {
        hir::BinOpKind::Lt => hir::BinOpKind::Gt,
        hir::BinOpKind::Le => hir::BinOpKind::Ge,
        hir::BinOpKind::Gt => hir::BinOpKind::Lt,
        hir::BinOpKind::Ge => hir::BinOpKind::Le,
        other => other,
    }
}

/// The comparison that holds when `a op b` is false.
fn negated_comparison(op: hir::BinOpKind) -> hir::BinOpKind {
    match op {
        hir::BinOpKind::Lt => hir::BinOpKind::Ge,
        hir::BinOpKind::Le => hir::BinOpKind::Gt,
        hir::BinOpKind::Gt => hir::BinOpKind::Le,
        hir::BinOpKind::Ge => hir::BinOpKind::Lt,
        hir::BinOpKind::Eq => hir::BinOpKind::Ne,
        hir::BinOpKind::Ne => hir::BinOpKind::Eq,
        other => other,
    }
}

/// Facts about `inner` when the comparison `lhs op rhs` holds.
fn comparison_facts<'tcx>(
    cx: &LateContext<'tcx>,
    op: hir::BinOpKind,
    lhs: &'tcx hir::Expr<'tcx>,
    rhs: &'tcx hir::Expr<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> BoundFacts {
    // Normalize to `inner op bound`.
    let (op, bound) = if same_checked_expr(lhs, inner) {
        (op, rhs)
    } else if same_checked_expr(rhs, inner) {
        (mirrored_comparison(op), lhs)
    } else {
        return BoundFacts::NONE;
    };

    BoundFacts {
        upper: matches!(op, hir::BinOpKind::Le | hir::BinOpKind::Lt)
            && is_target_const_cast_to_source(cx, bound, source_ty, target_ty, "MAX"),
        lower: matches!(op, hir::BinOpKind::Ge | hir::BinOpKind::Gt)
            && is_target_lower_bound(cx, bound, source_ty, target_ty),
    }
}

fn is_result_with_ok_type<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>, ok_ty: Ty<'tcx>) -> bool {
    let ty::Adt(adt, args) = cx.typeck_results().expr_ty(expr).kind() else {
        return false;
    };
    cx.tcx.is_diagnostic_item(rustc_span::sym::Result, adt.did()) && args[0].expect_ty() == ok_ty
}

fn path_last_segment_name_is(expr: &hir::Expr<'_>, expected: &str) -> bool {
    let ExprKind::Path(path) = &expr.kind else {
        return false;
    };

    match path {
        hir::QPath::Resolved(_, path) => {
            path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == expected)
        }
        hir::QPath::TypeRelative(_, segment) => segment.ident.name.as_str() == expected,
    }
}

fn exact_try_conversion_guard<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> bool {
    if !is_result_with_ok_type(cx, expr, target_ty) {
        return false;
    }

    match &expr.kind {
        ExprKind::Call(callee, [arg]) => {
            path_last_segment_name_is(callee, "try_from")
                && cx.typeck_results().expr_ty(arg) == source_ty
                && same_checked_expr(arg, inner)
        }
        ExprKind::MethodCall(segment, receiver, args, _) => {
            segment.ident.name.as_str() == "try_into"
                && args.is_empty()
                && cx.typeck_results().expr_ty(receiver) == source_ty
                && same_checked_expr(receiver, inner)
        }
        _ => false,
    }
}

/// Facts about `inner` when `cond` evaluates to `holds`.
fn condition_facts<'tcx>(
    cx: &LateContext<'tcx>,
    cond: &'tcx hir::Expr<'tcx>,
    holds: bool,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> BoundFacts {
    match &cond.kind {
        ExprKind::Binary(op, lhs, rhs) => match (op.node, holds) {
            // `a && b` holding (or `a || b` failing) means both sides hold (fail).
            (hir::BinOpKind::And, true) | (hir::BinOpKind::Or, false) => {
                condition_facts(cx, lhs, holds, inner, source_ty, target_ty)
                    .union(condition_facts(cx, rhs, holds, inner, source_ty, target_ty))
            }
            // `a || b` holding (or `a && b` failing) only proves what both sides prove.
            (hir::BinOpKind::Or, true) | (hir::BinOpKind::And, false) => {
                condition_facts(cx, lhs, holds, inner, source_ty, target_ty)
                    .intersection(condition_facts(cx, rhs, holds, inner, source_ty, target_ty))
            }
            (op, true) => comparison_facts(cx, op, lhs, rhs, inner, source_ty, target_ty),
            (op, false) => comparison_facts(cx, negated_comparison(op), lhs, rhs, inner, source_ty, target_ty),
        },
        ExprKind::MethodCall(segment, receiver, args, _) => {
            let expected = if holds { "is_ok" } else { "is_err" };
            if args.is_empty()
                && segment.ident.name.as_str() == expected
                && exact_try_conversion_guard(cx, receiver, inner, source_ty, target_ty)
            {
                BoundFacts::BOTH
            } else {
                BoundFacts::NONE
            }
        }
        ExprKind::Unary(hir::UnOp::Not, nested) => condition_facts(cx, nested, !holds, inner, source_ty, target_ty),
        ExprKind::Block(block, _) => block
            .expr
            .map_or(BoundFacts::NONE, |tail| condition_facts(cx, tail, holds, inner, source_ty, target_ty)),
        _ => BoundFacts::NONE,
    }
}

/// Facts proven by clamping the casted expression with `.min(<target>::MAX as
/// <source>)` and `.max(0 | <target>::MIN as <source>)` before the cast.
fn clamp_facts<'tcx>(
    cx: &LateContext<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> BoundFacts {
    let ExprKind::MethodCall(segment, receiver, [arg], _) = &inner.kind else {
        return BoundFacts::NONE;
    };

    if cx.typeck_results().expr_ty(receiver) != source_ty || cx.typeck_results().expr_ty(arg) != source_ty {
        return BoundFacts::NONE;
    }

    let clamped = match segment.ident.name.as_str() {
        "min" if is_target_const_cast_to_source(cx, arg, source_ty, target_ty, "MAX") => BoundFacts {
            upper: true,
            lower: false,
        },
        "max" if is_target_lower_bound(cx, arg, source_ty, target_ty) => BoundFacts {
            upper: false,
            lower: true,
        },
        _ => return BoundFacts::NONE,
    };

    clamped.union(clamp_facts(cx, receiver, source_ty, target_ty))
}

fn bounds_guard_facts<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> BoundFacts {
    let mut facts = BoundFacts::NONE;
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Expr(parent) => {
                let ExprKind::If(cond, then_expr, else_expr) = &parent.kind else {
                    continue;
                };

                if span_contains(then_expr.span, expr.span) {
                    facts = facts.union(condition_facts(cx, cond, true, inner, source_ty, target_ty));
                }

                if else_expr.is_some_and(|branch| span_contains(branch.span, expr.span)) {
                    facts = facts.union(condition_facts(cx, cond, false, inner, source_ty, target_ty));
                }
            }
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }
    facts
}

/// Facts proven by earlier `assert!` statements in the innermost enclosing block.
fn prior_same_block_assert_facts<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    inner: &'tcx hir::Expr<'tcx>,
    source_ty: Ty<'tcx>,
    target_ty: Ty<'tcx>,
) -> BoundFacts {
    let mut facts = BoundFacts::NONE;
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Block(block) => {
                let expr_callsite = expr.span.source_callsite();

                for stmt in block.stmts {
                    let stmt_callsite = stmt.span.source_callsite();
                    if stmt_callsite.hi() > expr_callsite.lo() {
                        break;
                    }

                    let (hir::StmtKind::Semi(assert_expr) | hir::StmtKind::Expr(assert_expr)) = stmt.kind else {
                        continue;
                    };

                    let ExprKind::If(cond, _, _) = &assert_expr.kind else {
                        continue;
                    };

                    if !assert_expr.span.from_expansion() {
                        continue;
                    }

                    let Ok(callsite) = cx.sess().source_map().span_to_snippet(assert_expr.span.source_callsite())
                    else {
                        continue;
                    };
                    if !callsite.trim_start().starts_with("assert!") {
                        continue;
                    }

                    // `assert!(cond)` lowers to `if !cond { panic }`, so code after it sees `!cond` false.
                    facts = facts.union(condition_facts(cx, cond, false, inner, source_ty, target_ty));
                }

                return facts;
            }
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    facts
}

/// Check whether `attr` is `#[allow(clippy::cast_possible_truncation)]` or
/// `#[expect(clippy::cast_possible_truncation)]`.
fn is_truncation_allow_attr(attr: &hir::Attribute) -> bool {
    if !(attr.has_name(rustc_span::sym::allow) || attr.has_name(rustc_span::sym::expect)) {
        return false;
    }

    attr.meta_item_list().is_some_and(|items| {
        items.iter().any(|item| {
            item.meta_item().is_some_and(|meta| {
                matches!(
                    meta.path.segments.as_slice(),
                    [tool, lint] if tool.ident.name.as_str() == "clippy"
                        && lint.ident.name.as_str() == "cast_possible_truncation"
                )
            })
        })
    })
}

/// Check if the cast is inside an item, method, statement, or expression marked
/// `#[allow(clippy::cast_possible_truncation)]` (or `#[expect(...)]`).
fn has_truncation_allow(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    for (hir_id, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        if cx.tcx.hir_attrs(hir_id).iter().any(is_truncation_allow_attr) {
            return true;
        }
        if matches!(node, hir::Node::Item(_)) {
            break;
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for UncheckedNarrowing {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Cast(inner, _) = &expr.kind else {
            return;
        };

        let source_ty = cx.typeck_results().expr_ty(inner);
        let target_ty = cx.typeck_results().expr_ty(expr);

        let Some(source_width) = source_int_width(source_ty) else {
            return;
        };
        // Casts to `usize`/`isize` belong to `platform_dependent_cast`.
        let Some(target_width) = int_width(target_ty) else {
            return;
        };

        if source_width <= target_width {
            return;
        }

        // Skip if clamped before the cast or inside bounds-checking guards
        let facts = clamp_facts(cx, inner, source_ty, target_ty)
            .union(bounds_guard_facts(cx, expr, inner, source_ty, target_ty))
            .union(prior_same_block_assert_facts(cx, expr, inner, source_ty, target_ty));
        if facts.proves_safe_narrowing(source_ty) {
            return;
        }

        // Skip if inside #[allow(clippy::cast_possible_truncation)]
        if has_truncation_allow(cx, expr) {
            return;
        }

        let source_name = ty_name(source_ty);
        let target_name = ty_name(target_ty);

        let message = format!("narrowing cast from `{source_name}` to `{target_name}` without bounds check");
        if let Some(suggestion) =
            crate::cast_suggestion::cast_conversion_suggestion(cx, expr, inner, source_ty, target_ty, false)
        {
            emit_tiger_style_lint_with_suggestion(
                cx,
                UNCHECKED_NARROWING,
                expr.span,
                message,
                "unchecked_narrowing",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, UNCHECKED_NARROWING, expr.span, message, "unchecked_narrowing");
        }
    }
}
