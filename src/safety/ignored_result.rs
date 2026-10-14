use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_hir::PatKind;
use rustc_hir::StmtKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct IgnoredResultFacts {
    is_result_type: bool,
    is_drop_impl_context: bool,
    is_fire_and_forget_shutdown_send: bool,
}

declare_tool_lint! {
    /// Detects `let _ = expr` where `expr` returns `Result<_, _>`. Silently
    /// discarding a Result hides errors that could propagate or be logged.
    /// A study of catastrophic failures in distributed systems found that
    /// 92% had an error path that was explicitly ignored.
    pub tigerstyle::IGNORED_RESULT,
    Warn,
    "Result value silently discarded with `let _ = ...`"
}

declare_lint_pass!(IgnoredResult => [IGNORED_RESULT]);

fn classify_ignored_result(facts: IgnoredResultFacts) -> bool {
    facts.is_result_type && !facts.is_drop_impl_context && !facts.is_fire_and_forget_shutdown_send
}

fn is_result_type(cx: &LateContext<'_>, t: ty::Ty<'_>) -> bool {
    if let ty::Adt(adt, _) = t.kind() {
        cx.tcx.is_diagnostic_item(sym::Result, adt.did())
    } else {
        false
    }
}

fn expr_is_result_type(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    is_result_type(cx, cx.typeck_results().expr_ty(expr))
}

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn is_drop_impl_context(cx: &LateContext<'_>, hir_id: hir::HirId) -> bool {
    let mut saw_drop_method = false;

    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        match node {
            hir::Node::ImplItem(impl_item) => {
                saw_drop_method = impl_item.ident.name == sym::drop;
            }
            hir::Node::Item(item) => {
                let ItemKind::Impl(impl_block) = &item.kind else {
                    break;
                };

                if !saw_drop_method {
                    break;
                }

                let Some(of_trait) = impl_block.of_trait else {
                    break;
                };

                let Some(drop_trait) = cx.tcx.lang_items().drop_trait() else {
                    break;
                };

                return of_trait.trait_ref.trait_def_id() == Some(drop_trait);
            }
            hir::Node::TraitItem(_) => break,
            _ => {}
        }
    }

    false
}

fn supported_sender_type(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).peel_refs().kind() else {
        return false;
    };

    match_def_path(cx, adt.did(), &["std", "sync", "mpsc", "Sender"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "Sender"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "bounded", "Sender"])
}

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            hir::ExprKind::DropTemps(inner) => expr = inner,
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

fn send_receiver_from_expr<'tcx>(expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let expr = peel_transparent_expr(expr);

    match &expr.kind {
        hir::ExprKind::MethodCall(segment, receiver, args, _) => {
            (segment.ident.name.as_str() == "send" && args.len() == 1).then_some(*receiver)
        }
        hir::ExprKind::Match(scrutinee, _, hir::MatchSource::AwaitDesugar) => {
            let hir::ExprKind::Call(_, [future_expr]) = &scrutinee.kind else {
                return None;
            };
            let hir::ExprKind::MethodCall(segment, receiver, args, _) = &future_expr.kind else {
                return None;
            };
            (segment.ident.name.as_str() == "send" && args.len() == 1).then_some(*receiver)
        }
        _ => None,
    }
}

fn is_shutdown_or_close_context(cx: &LateContext<'_>, hir_id: hir::HirId) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        let name = match node {
            hir::Node::ImplItem(impl_item) => Some(impl_item.ident.name),
            hir::Node::TraitItem(trait_item) => Some(trait_item.ident.name),
            hir::Node::Item(item) => cx.tcx.hir_opt_name(cx.tcx.local_def_id_to_hir_id(item.owner_id.def_id)),
            _ => None,
        };

        if name.is_some_and(|name| matches!(name.as_str(), "shutdown" | "close")) {
            return true;
        }
    }

    false
}

fn is_fire_and_forget_shutdown_send(cx: &LateContext<'_>, hir_id: hir::HirId, init: &hir::Expr<'_>) -> bool {
    is_shutdown_or_close_context(cx, hir_id)
        && send_receiver_from_expr(init).is_some_and(|receiver| supported_sender_type(cx, receiver))
}

fn ignored_result_facts(cx: &LateContext<'_>, hir_id: hir::HirId, expr: &hir::Expr<'_>) -> IgnoredResultFacts {
    IgnoredResultFacts {
        is_result_type: expr_is_result_type(cx, expr),
        is_drop_impl_context: is_drop_impl_context(cx, hir_id),
        is_fire_and_forget_shutdown_send: is_fire_and_forget_shutdown_send(cx, hir_id, expr),
    }
}

impl<'tcx> LateLintPass<'tcx> for IgnoredResult {
    fn check_local(&mut self, cx: &LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        if !matches!(local.pat.kind, PatKind::Wild) {
            return;
        }

        let Some(init) = local.init else {
            return;
        };

        let facts = ignored_result_facts(cx, local.hir_id, init);
        if !classify_ignored_result(facts) {
            return;
        }

        let message = "`Result` value silently discarded with `let _ = ...`";
        if let Some(suggestion) = ignored_result_suggestion(cx, local) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                IGNORED_RESULT,
                local.span,
                message,
                "ignored_result",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, IGNORED_RESULT, local.span, message, "ignored_result");
        }
    }

    fn check_stmt(&mut self, cx: &LateContext<'tcx>, stmt: &'tcx hir::Stmt<'tcx>) {
        let StmtKind::Semi(expr) = stmt.kind else {
            return;
        };

        let facts = ignored_result_facts(cx, stmt.hir_id, expr);
        if !classify_ignored_result(facts) {
            return;
        }

        let message = "`Result` value silently discarded in statement position";
        if let Some(suggestion) = ignored_result_expr_suggestion(cx, expr) {
            emit_tiger_style_lint_with_suggestion(cx, IGNORED_RESULT, expr.span, message, "ignored_result", suggestion);
        } else {
            emit_tiger_style_lint(cx, IGNORED_RESULT, expr.span, message, "ignored_result");
        }
    }
}

fn ignored_result_expr_suggestion(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let expr_snippet = sm.span_to_snippet(expr.span).ok()?;
    let replacement = format!("{expr_snippet}?");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "propagate the error with `?` instead of discarding".to_owned(),
        replacement,
        applicability: Applicability::MaybeIncorrect,
    })
}

fn ignored_result_suggestion(cx: &LateContext<'_>, local: &hir::LetStmt<'_>) -> Option<LintSuggestion> {
    let init = local.init?;
    let sm = cx.sess().source_map();
    let init_snippet = sm.span_to_snippet(init.span).ok()?;
    let replacement = format!("{init_snippet}?");

    Some(LintSuggestion::Single {
        span: local.span,
        message: "propagate the error with `?` instead of discarding".to_owned(),
        replacement,
        applicability: Applicability::MaybeIncorrect,
    })
}

#[cfg(test)]
mod tests {
    use super::IgnoredResultFacts;
    use super::classify_ignored_result;

    fn result_discard() -> IgnoredResultFacts {
        IgnoredResultFacts {
            is_result_type: true,
            is_drop_impl_context: false,
            is_fire_and_forget_shutdown_send: false,
        }
    }

    #[test]
    fn classifies_result_discard() {
        assert!(classify_ignored_result(result_discard()));
    }

    #[test]
    fn skips_non_result_values() {
        let facts = IgnoredResultFacts {
            is_result_type: false,
            ..result_discard()
        };

        assert!(!classify_ignored_result(facts));
    }

    #[test]
    fn preserves_drop_and_shutdown_exceptions() {
        let drop_context = IgnoredResultFacts {
            is_drop_impl_context: true,
            ..result_discard()
        };
        let shutdown_send = IgnoredResultFacts {
            is_fire_and_forget_shutdown_send: true,
            ..result_discard()
        };

        assert!(!classify_ignored_result(drop_context));
        assert!(!classify_ignored_result(shutdown_send));
    }
}
