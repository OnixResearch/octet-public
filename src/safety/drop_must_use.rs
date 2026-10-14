use rustc_hir as hir;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::find_attr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects explicit `drop(...)` calls that consume `#[must_use]` values.
    /// Calling `drop` on a value whose type or producer is marked `#[must_use]`
    /// acknowledges the value only syntactically while still discarding the
    /// success/error or obligation it represents.
    pub tigerstyle::DROP_MUST_USE,
    Warn,
    "explicitly dropping a must-use value hides the required handling"
}

declare_lint_pass!(DropMustUse => [DROP_MUST_USE]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DropMustUseFacts {
    is_std_mem_drop: bool,
    type_is_must_use: bool,
    producer_is_must_use: bool,
    is_in_test_context: bool,
}

fn classify_drop_must_use(facts: DropMustUseFacts) -> bool {
    facts.is_std_mem_drop && (facts.type_is_must_use || facts.producer_is_must_use) && !facts.is_in_test_context
}

#[allow(deprecated)]
fn has_must_use_attr(cx: &LateContext<'_>, def_id: rustc_hir::def_id::DefId) -> bool {
    find_attr!(cx.tcx.get_all_attrs(def_id), AttributeKind::MustUse { .. })
}

fn type_is_must_use(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty = ty.peel_refs();
    let ty::Adt(adt, _) = ty.kind() else {
        return false;
    };

    cx.tcx.is_diagnostic_item(sym::Result, adt.did())
        || cx.tcx.is_diagnostic_item(sym::Option, adt.did())
        || has_must_use_attr(cx, adt.did())
}

fn expr_is_must_use_call(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        hir::ExprKind::Call(callee, _) => {
            let hir::ExprKind::Path(qpath) = &callee.kind else {
                return false;
            };
            cx.qpath_res(qpath, callee.hir_id).opt_def_id().is_some_and(|def_id| has_must_use_attr(cx, def_id))
        }
        hir::ExprKind::MethodCall(_, _, _, _) => cx
            .typeck_results()
            .type_dependent_def_id(expr.hir_id)
            .is_some_and(|def_id| has_must_use_attr(cx, def_id)),
        _ => false,
    }
}

fn is_std_mem_drop(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Path(qpath) = &callee.kind else {
        return false;
    };

    cx.qpath_res(qpath, callee.hir_id)
        .opt_def_id()
        .and_then(|def_id| cx.tcx.get_diagnostic_name(def_id))
        .is_some_and(|name| name == sym::mem_drop)
}

impl<'tcx> LateLintPass<'tcx> for DropMustUse {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let hir::ExprKind::Call(callee, [arg]) = &expr.kind else {
            return;
        };

        let facts = DropMustUseFacts {
            is_std_mem_drop: is_std_mem_drop(cx, callee),
            type_is_must_use: type_is_must_use(cx, cx.typeck_results().expr_ty(arg)),
            producer_is_must_use: expr_is_must_use_call(cx, arg),
            is_in_test_context: test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span),
        };
        if !classify_drop_must_use(facts) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            DROP_MUST_USE,
            expr.span,
            "explicitly dropping a must-use value hides the required handling",
            "drop_must_use",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::DropMustUseFacts;
    use super::classify_drop_must_use;

    fn explicit_drop_of_must_use_type() -> DropMustUseFacts {
        DropMustUseFacts {
            is_std_mem_drop: true,
            type_is_must_use: true,
            producer_is_must_use: false,
            is_in_test_context: false,
        }
    }

    #[test]
    fn classifies_explicit_drop_of_must_use_type_or_producer() {
        let producer = DropMustUseFacts {
            type_is_must_use: false,
            producer_is_must_use: true,
            ..explicit_drop_of_must_use_type()
        };

        assert!(classify_drop_must_use(explicit_drop_of_must_use_type()));
        assert!(classify_drop_must_use(producer));
    }

    #[test]
    fn skips_plain_drop_and_test_contexts() {
        let plain_value = DropMustUseFacts {
            type_is_must_use: false,
            producer_is_must_use: false,
            ..explicit_drop_of_must_use_type()
        };
        let test_context = DropMustUseFacts {
            is_in_test_context: true,
            ..explicit_drop_of_must_use_type()
        };

        assert!(!classify_drop_must_use(plain_value));
        assert!(!classify_drop_must_use(test_context));
    }
}
