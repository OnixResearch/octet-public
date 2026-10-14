use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const CONSTRUCTOR_NEW: &str = "new";
pub(crate) const SUPPORTED_ALLOCATING_CRATES: &[&str] = &["alloc", "std"];
pub(crate) const SUPPORTED_ALLOCATING_TYPES: &[&str] = &["Vec", "String", "Box", "HashMap", "BTreeMap"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct AllocationInLoopFacts {
    is_supported_allocation_constructor: bool,
    is_inside_loop: bool,
    is_in_test_context: bool,
    is_from_expansion: bool,
}

fn classify_allocation_in_loop(facts: AllocationInLoopFacts) -> bool {
    facts.is_supported_allocation_constructor
        && facts.is_inside_loop
        && !facts.is_in_test_context
        && !facts.is_from_expansion
}

declare_tool_lint! {
    /// Detects obvious allocation constructors inside loop bodies.
    /// Hot paths should allocate outside the loop or reserve capacity before
    /// iteration unless the allocation is the intentionally bounded work unit.
    pub tigerstyle::ALLOCATION_IN_LOOP,
    Allow,
    "allocation constructor runs inside a loop"
}

declare_lint_pass!(AllocationInLoop => [ALLOCATION_IN_LOOP]);

pub(crate) fn supported_allocating_type(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> Option<String> {
    let ty::Adt(adt, _) = ty.peel_refs().kind() else {
        return None;
    };

    let def_path = cx.get_def_path(adt.did());
    let crate_name = def_path.first()?.as_str();
    let type_name = def_path.last()?.as_str();

    if SUPPORTED_ALLOCATING_CRATES.contains(&crate_name) && SUPPORTED_ALLOCATING_TYPES.contains(&type_name) {
        Some(type_name.to_owned())
    } else {
        None
    }
}

pub(crate) fn expr_supported_allocation_type(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    supported_allocating_type(cx, cx.typeck_results().expr_ty(expr))
}

pub(crate) fn qpath_names_constructor_new(qpath: &hir::QPath<'_>) -> bool {
    match qpath {
        hir::QPath::Resolved(_, path) => {
            path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == CONSTRUCTOR_NEW)
        }
        hir::QPath::TypeRelative(_, segment) => segment.ident.name.as_str() == CONSTRUCTOR_NEW,
    }
}

pub(crate) fn expr_is_zero_arg_new_call(expr: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Call(callee, args) = &expr.kind else {
        return false;
    };
    if !args.is_empty() {
        return false;
    }
    let hir::ExprKind::Path(qpath) = &callee.kind else {
        return false;
    };

    qpath_names_constructor_new(qpath)
}

/// Returns the allocating type name if `expr` is an obvious allocation call
/// (constructor call, method call, or macro expansion that produces a known
/// heap-allocating type such as `Vec`, `String`, `Box`, `HashMap`, `BTreeMap`).
pub(crate) fn expr_is_allocation(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<String> {
    match &expr.kind {
        hir::ExprKind::Call(callee, _) => {
            let ty = cx.typeck_results().expr_ty(expr);
            if let Some(type_name) = supported_allocating_type(cx, ty)
                && let hir::ExprKind::Path(qpath) = &callee.kind
            {
                if qpath_names_constructor_new(qpath) {
                    return Some(type_name);
                }
                if let Some(ref method) = qpath_names_method_name(qpath)
                    && (method.as_str() == "from" || method.as_str() == "from_str")
                {
                    return Some(type_name);
                }
            }
        }
        hir::ExprKind::MethodCall(method, ..) => {
            let method_name = method.ident.name.as_str();
            if method_name == "to_string" || method_name == "to_owned" {
                let ty = cx.typeck_results().expr_ty(expr);
                if let Some(type_name) = supported_allocating_type(cx, ty) {
                    return Some(type_name);
                }
            }
        }
        _ => {}
    }
    None
}

fn qpath_names_method_name(qpath: &hir::QPath<'_>) -> Option<String> {
    let segment_name = match qpath {
        hir::QPath::Resolved(_, path) => path.segments.last()?.ident.name.as_str(),
        hir::QPath::TypeRelative(_, segment) => segment.ident.name.as_str(),
    };
    Some(segment_name.to_owned())
}

fn hir_id_has_loop_parent(cx: &LateContext<'_>, hir_id: hir::HirId) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        match node {
            hir::Node::Expr(parent) if matches!(parent.kind, hir::ExprKind::Loop(..)) => return true,
            hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_) => return false,
            _ => {}
        }
    }
    false
}

fn allocation_in_loop_facts(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> AllocationInLoopFacts {
    AllocationInLoopFacts {
        is_supported_allocation_constructor: expr_is_zero_arg_new_call(expr)
            && expr_supported_allocation_type(cx, expr).is_some(),
        is_inside_loop: hir_id_has_loop_parent(cx, expr.hir_id),
        is_in_test_context: test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span),
        is_from_expansion: expr.span.from_expansion(),
    }
}

impl<'tcx> LateLintPass<'tcx> for AllocationInLoop {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let facts = allocation_in_loop_facts(cx, expr);
        if !classify_allocation_in_loop(facts) {
            return;
        }

        let type_name = expr_supported_allocation_type(cx, expr).unwrap_or_else(|| "collection".to_owned());
        let message = format!("{type_name} allocation constructor runs inside a loop");
        emit_tiger_style_lint(cx, ALLOCATION_IN_LOOP, expr.span, message, "allocation_in_loop");
    }
}

#[cfg(test)]
mod tests {
    use super::AllocationInLoopFacts;
    use super::classify_allocation_in_loop;

    fn loop_allocation() -> AllocationInLoopFacts {
        AllocationInLoopFacts {
            is_supported_allocation_constructor: true,
            is_inside_loop: true,
            is_in_test_context: false,
            is_from_expansion: false,
        }
    }

    #[test]
    fn classifies_allocation_inside_loop() {
        assert!(classify_allocation_in_loop(loop_allocation()));
    }

    #[test]
    fn skips_non_allocation_or_outside_loop() {
        let not_allocation = AllocationInLoopFacts {
            is_supported_allocation_constructor: false,
            ..loop_allocation()
        };
        let outside_loop = AllocationInLoopFacts {
            is_inside_loop: false,
            ..loop_allocation()
        };

        assert!(!classify_allocation_in_loop(not_allocation));
        assert!(!classify_allocation_in_loop(outside_loop));
    }

    #[test]
    fn skips_test_contexts_and_expansions() {
        let test_context = AllocationInLoopFacts {
            is_in_test_context: true,
            ..loop_allocation()
        };
        let expansion = AllocationInLoopFacts {
            is_from_expansion: true,
            ..loop_allocation()
        };

        assert!(!classify_allocation_in_loop(test_context));
        assert!(!classify_allocation_in_loop(expansion));
    }
}
