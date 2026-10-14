use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_hir::StmtKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const JOIN_HANDLE_TYPE: &str = "JoinHandle";
const SUPPORTED_JOIN_HANDLE_CRATES: &[&str] = &["std", "tokio"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IgnoredJoinHandleFacts {
    returns_supported_join_handle: bool,
    is_explicit_discard: bool,
    is_in_test_context: bool,
}

fn classify_ignored_join_handle(facts: IgnoredJoinHandleFacts) -> bool {
    facts.returns_supported_join_handle && facts.is_explicit_discard && !facts.is_in_test_context
}

declare_tool_lint! {
    /// Detects spawned task or thread join handles discarded immediately.
    /// Detached work needs an explicit lifecycle decision: await, join, abort,
    /// store, or document the fire-and-forget boundary.
    pub tigerstyle::IGNORED_JOIN_HANDLE,
    Warn,
    "spawned task or thread join handle is discarded"
}

declare_lint_pass!(IgnoredJoinHandle => [IGNORED_JOIN_HANDLE]);

fn supported_join_handle_type(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty::Adt(adt, _) = ty.peel_refs().kind() else {
        return false;
    };

    let def_path = cx.get_def_path(adt.did());
    let Some(crate_name) = def_path.first().map(rustc_span::Symbol::as_str) else {
        return false;
    };
    let Some(type_name) = def_path.last().map(rustc_span::Symbol::as_str) else {
        return false;
    };

    SUPPORTED_JOIN_HANDLE_CRATES.contains(&crate_name) && type_name == JOIN_HANDLE_TYPE
}

fn expr_returns_supported_join_handle(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    supported_join_handle_type(cx, cx.typeck_results().expr_ty(expr))
}

fn ignored_join_handle_facts(
    cx: &LateContext<'_>,
    hir_id: hir::HirId,
    expr: &hir::Expr<'_>,
    is_explicit_discard: bool,
) -> IgnoredJoinHandleFacts {
    IgnoredJoinHandleFacts {
        returns_supported_join_handle: expr_returns_supported_join_handle(cx, expr),
        is_explicit_discard,
        is_in_test_context: test_context::late_hir_id_is_in_test_context(cx, hir_id, expr.span),
    }
}

fn emit_ignored_join_handle(cx: &LateContext<'_>, span: rustc_span::Span) {
    emit_tiger_style_lint(
        cx,
        IGNORED_JOIN_HANDLE,
        span,
        "spawned task or thread join handle is discarded",
        "ignored_join_handle",
    );
}

impl<'tcx> LateLintPass<'tcx> for IgnoredJoinHandle {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        if !matches!(local.pat.kind, PatKind::Wild) {
            return;
        }
        let Some(init) = local.init else {
            return;
        };

        let facts = ignored_join_handle_facts(cx, local.hir_id, init, true);
        if classify_ignored_join_handle(facts) {
            emit_ignored_join_handle(cx, init.span);
        }
    }

    fn check_stmt(&mut self, cx: &LateContext<'tcx>, stmt: &'tcx hir::Stmt<'tcx>) {
        let StmtKind::Semi(expr) = stmt.kind else {
            return;
        };

        let facts = ignored_join_handle_facts(cx, stmt.hir_id, expr, true);
        if classify_ignored_join_handle(facts) {
            emit_ignored_join_handle(cx, expr.span);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IgnoredJoinHandleFacts;
    use super::classify_ignored_join_handle;

    fn discarded_join_handle() -> IgnoredJoinHandleFacts {
        IgnoredJoinHandleFacts {
            returns_supported_join_handle: true,
            is_explicit_discard: true,
            is_in_test_context: false,
        }
    }

    #[test]
    fn classifies_discarded_join_handle() {
        assert!(classify_ignored_join_handle(discarded_join_handle()));
    }

    #[test]
    fn skips_non_join_handle_or_named_lifecycle() {
        let not_join_handle = IgnoredJoinHandleFacts {
            returns_supported_join_handle: false,
            ..discarded_join_handle()
        };
        let named_lifecycle = IgnoredJoinHandleFacts {
            is_explicit_discard: false,
            ..discarded_join_handle()
        };

        assert!(!classify_ignored_join_handle(not_join_handle));
        assert!(!classify_ignored_join_handle(named_lifecycle));
    }

    #[test]
    fn skips_test_contexts() {
        let test_context = IgnoredJoinHandleFacts {
            is_in_test_context: true,
            ..discarded_join_handle()
        };

        assert!(!classify_ignored_join_handle(test_context));
    }
}
