use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects direct recursion (a function calling itself). Tiger Style
    /// avoids recursion because it makes execution depth unpredictable
    /// and can overflow the stack. Use iteration with explicit bounds
    /// instead.
    pub tigerstyle::NO_RECURSION,
    Warn,
    "function calls itself recursively"
}

declare_lint_pass!(NoRecursion => [NO_RECURSION]);

fn resolved_call_is_recursive(target_def_id: hir::def_id::DefId, candidate_def_id: Option<hir::def_id::DefId>) -> bool {
    candidate_def_id == Some(target_def_id)
}

struct RecursionFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    target_def_id: hir::def_id::DefId,
    found: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for RecursionFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let ExprKind::Call(func, _) = &expr.kind
            && let ExprKind::Path(hir::QPath::Resolved(_, path)) = &func.kind
            && let hir::def::Res::Def(_, def_id) = path.res
            && resolved_call_is_recursive(self.target_def_id, Some(def_id))
        {
            self.found = Some(expr.span);
            return;
        }

        if let ExprKind::MethodCall(_, _, _, _) = &expr.kind
            && resolved_call_is_recursive(
                self.target_def_id,
                self.cx.typeck_results().type_dependent_def_id(expr.hir_id),
            )
        {
            self.found = Some(expr.span);
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for NoRecursion {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        _span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        let mut finder = RecursionFinder {
            cx,
            target_def_id: def_id.to_def_id(),
            found: None,
        };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
            emit_tiger_style_lint(cx, NO_RECURSION, call_span, "function calls itself recursively", "no_recursion");
        }
    }
}

#[cfg(test)]
mod tests {
    use rustc_hir::def_id::DefId;
    use rustc_hir::def_id::LOCAL_CRATE;

    use super::resolved_call_is_recursive;

    fn local_def(index: u32) -> DefId {
        DefId {
            krate: LOCAL_CRATE,
            index: rustc_hir::def_id::DefIndex::from_u32(index),
        }
    }

    #[test]
    fn classifies_same_resolved_call_as_recursive() {
        let target = local_def(1);

        assert!(resolved_call_is_recursive(target, Some(target)));
    }

    #[test]
    fn skips_different_or_unresolved_calls() {
        let target = local_def(1);
        let other = local_def(2);

        assert!(!resolved_call_is_recursive(target, Some(other)));
        assert!(!resolved_call_is_recursive(target, None));
    }
}
