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
    is_in_test_context: bool,
}

fn classify_ignored_join_handle(facts: IgnoredJoinHandleFacts) -> bool {
    facts.returns_supported_join_handle && !facts.is_in_test_context
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

/// Whether `ty` is an owned `JoinHandle` from a supported crate.
fn owned_join_handle_type(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    let ty::Adt(adt, _) = ty.kind() else {
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

/// Whether `ty` is an owned `JoinHandle`, possibly inside `Result`/`Option`
/// wrappers (`std::thread::Builder::spawn` returns `io::Result<JoinHandle<T>>`).
/// References are not peeled: dropping a borrowed handle detaches nothing.
fn wrapped_join_handle_type<'tcx>(cx: &LateContext<'tcx>, mut ty: ty::Ty<'tcx>) -> bool {
    loop {
        if owned_join_handle_type(cx, ty) {
            return true;
        }
        let ty::Adt(adt, args) = ty.kind() else {
            return false;
        };
        let is_wrapper = [rustc_span::sym::Result, rustc_span::sym::Option]
            .into_iter()
            .any(|wrapper| cx.tcx.is_diagnostic_item(wrapper, adt.did()));
        let Some(inner) = args.types().next().filter(|_| is_wrapper) else {
            return false;
        };
        ty = inner;
    }
}

fn peel_trivial_blocks<'a, 'tcx>(mut expr: &'a hir::Expr<'tcx>) -> &'a hir::Expr<'tcx> {
    loop {
        match expr.kind {
            hir::ExprKind::DropTemps(inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    return expr;
                };
                expr = inner;
            }
            _ => return expr,
        }
    }
}

/// Whether `expr` calls a function whose declared return type is a
/// (possibly wrapped) `JoinHandle`, such as `std::thread::spawn` or
/// `std::thread::Builder::spawn`. A generic return such as `HashMap::insert`'s
/// `Option<V>` does not count: that handle is not the freshly spawned one.
fn calls_handle_producer(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let typeck_results = cx.typeck_results();
    let expr = peel_trivial_blocks(expr);
    let def_id = match expr.kind {
        hir::ExprKind::Call(callee, _) => match typeck_results.expr_ty(callee).kind() {
            ty::FnDef(def_id, _) => Some(*def_id),
            _ => None,
        },
        hir::ExprKind::MethodCall(..) => typeck_results.type_dependent_def_id(expr.hir_id),
        _ => None,
    };
    def_id.is_some_and(|def_id| {
        let declared_output = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder().output();
        wrapped_join_handle_type(cx, declared_output)
    })
}

/// Whether `expr` yields a freshly spawned handle inside `Result`/`Option`:
/// a call to a handle producer, or a combinator such as `bool::then` whose
/// closure argument calls one.
fn produces_fresh_wrapped_handle(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    if calls_handle_producer(cx, expr) {
        return true;
    }
    let (hir::ExprKind::Call(_, args) | hir::ExprKind::MethodCall(_, _, args, _)) = peel_trivial_blocks(expr).kind
    else {
        return false;
    };
    args.iter().any(|arg| match arg.kind {
        hir::ExprKind::Closure(closure) => calls_handle_producer(cx, cx.tcx.hir_body(closure.body).value),
        _ => false,
    })
}

fn expr_returns_supported_join_handle(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let resolved = cx.typeck_results().expr_ty(expr);
    owned_join_handle_type(cx, resolved.peel_refs())
        || (wrapped_join_handle_type(cx, resolved) && produces_fresh_wrapped_handle(cx, expr))
}

fn ignored_join_handle_facts(cx: &LateContext<'_>, hir_id: hir::HirId, expr: &hir::Expr<'_>) -> IgnoredJoinHandleFacts {
    IgnoredJoinHandleFacts {
        returns_supported_join_handle: expr_returns_supported_join_handle(cx, expr),
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
        // `let _ = ...` and `_ = ...` (lowered as `AssignDesugar`) discard a
        // handle; the `let _ = __arg` that rustc emits for a `_` async fn
        // parameter does not.
        if !matches!(local.pat.kind, PatKind::Wild)
            || !matches!(local.source, hir::LocalSource::Normal | hir::LocalSource::AssignDesugar)
        {
            return;
        }
        let Some(init) = local.init else {
            return;
        };

        let facts = ignored_join_handle_facts(cx, local.hir_id, init);
        if classify_ignored_join_handle(facts) {
            emit_ignored_join_handle(cx, init.span);
        }
    }

    fn check_stmt(&mut self, cx: &LateContext<'tcx>, stmt: &'tcx hir::Stmt<'tcx>) {
        let StmtKind::Semi(expr) = stmt.kind else {
            return;
        };

        let facts = ignored_join_handle_facts(cx, stmt.hir_id, expr);
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
            is_in_test_context: false,
        }
    }

    #[test]
    fn classifies_discarded_join_handle() {
        assert!(classify_ignored_join_handle(discarded_join_handle()));
    }

    #[test]
    fn skips_non_join_handle() {
        let not_join_handle = IgnoredJoinHandleFacts {
            returns_supported_join_handle: false,
            ..discarded_join_handle()
        };

        assert!(!classify_ignored_join_handle(not_join_handle));
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
