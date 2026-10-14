use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

const LINT_NAME: &str = "clone_then_borrow";

declare_tool_lint! {
    /// Detects cloning a value and immediately taking a shared borrow of the clone.
    /// Tiger Style treats clone-to-satisfy-borrow-checker patterns as performance
    /// and ownership-clarity smells when no owned value crosses a boundary.
    pub tigerstyle::CLONE_THEN_BORROW,
    Warn,
    "clone immediately borrowed instead of borrowing the original value"
}

declare_lint_pass!(CloneThenBorrow => [CLONE_THEN_BORROW]);

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            hir::ExprKind::DropTemps(inner) | hir::ExprKind::AddrOf(_, _, inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    break expr;
                };
                expr = inner;
            }
            _ => break expr,
        }
    }
}

fn def_path_looks_like_clone(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) else {
        return false;
    };
    let def_path = cx.get_def_path(def_id);
    def_path.last().is_some_and(|segment| *segment == Symbol::intern("clone"))
        && def_path.iter().any(|segment| *segment == Symbol::intern("Clone"))
}

fn borrowed_clone_receiver<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    if expr.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::AddrOf(_, hir::Mutability::Not, borrowed) = &expr.kind else {
        return None;
    };
    let borrowed = peel_transparent_expr(borrowed);
    if borrowed.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::MethodCall(segment, receiver, args, _) = &borrowed.kind else {
        return None;
    };
    if segment.ident.name.as_str() != "clone" || !args.is_empty() || !def_path_looks_like_clone(cx, borrowed) {
        return None;
    }

    Some(*receiver)
}

fn borrow_original_replacement(cx: &LateContext<'_>, receiver: &hir::Expr<'_>, receiver_snippet: &str) -> String {
    if matches!(cx.typeck_results().expr_ty(receiver).kind(), ty::Ref(..)) {
        receiver_snippet.to_owned()
    } else {
        format!("&{receiver_snippet}")
    }
}

impl<'tcx> LateLintPass<'tcx> for CloneThenBorrow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some(receiver) = borrowed_clone_receiver(cx, expr) else {
            return;
        };
        let Ok(receiver_snippet) = cx.sess().source_map().span_to_snippet(receiver.span) else {
            return;
        };
        let replacement = borrow_original_replacement(cx, receiver, &receiver_snippet);
        let message = "clone result is immediately borrowed; borrow the original value or use an explicit borrowed adapter instead";

        emit_tiger_style_lint_with_suggestion(
            cx,
            CLONE_THEN_BORROW,
            expr.span,
            message,
            LINT_NAME,
            LintSuggestion::Single {
                span: expr.span,
                message: "borrow the original value instead of cloning then borrowing".to_owned(),
                replacement,
                applicability: Applicability::MaybeIncorrect,
            },
        );
    }
}
