use rustc_hir as hir;
use rustc_hir::intravisit;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;
use rustc_span::Symbol;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects `loop { }` blocks that appear to lack an iteration bound.
    /// Unbounded loops are a class of liveness bug — a retry loop without
    /// a max-attempts counter can stall a system indefinitely.
    pub tigerstyle::UNBOUNDED_LOOP,
    Warn,
    "loop without visible iteration bound"
}

declare_lint_pass!(UnboundedLoop => [UNBOUNDED_LOOP]);

const COUNTER_MARKERS: &[&str] = &["MAX_RETRIES", "max_retries", "max_attempts", "attempt", "remaining"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokioReceiverKind {
    Bounded,
    Unbounded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LoopSourceKind {
    ExplicitLoop,
    While,
    For,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LoopBoundsFacts {
    source_kind: LoopSourceKind,
    is_from_expansion: bool,
    has_explicit_exit: bool,
    has_structural_counter_bound: bool,
    has_cancellation_path: bool,
    is_bounded_receiver_loop: bool,
    is_unbounded_receiver_loop: bool,
    is_iter_next_loop: bool,
}

fn classify_unbounded_loop(facts: LoopBoundsFacts) -> bool {
    if facts.is_from_expansion {
        return false;
    }

    match facts.source_kind {
        LoopSourceKind::For => false,
        LoopSourceKind::ExplicitLoop => {
            !(facts.has_explicit_exit || facts.has_structural_counter_bound || facts.has_cancellation_path)
        }
        LoopSourceKind::While => {
            if facts.has_explicit_exit || facts.has_structural_counter_bound || facts.is_bounded_receiver_loop {
                return false;
            }
            facts.is_unbounded_receiver_loop || facts.is_iter_next_loop
        }
    }
}

fn loop_source_kind(loop_source: hir::LoopSource) -> LoopSourceKind {
    match loop_source {
        hir::LoopSource::Loop => LoopSourceKind::ExplicitLoop,
        hir::LoopSource::While => LoopSourceKind::While,
        hir::LoopSource::ForLoop => LoopSourceKind::For,
    }
}

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn normalized_without_whitespace(snippet: &str) -> String {
    snippet.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn loop_has_counter_marker(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let Ok(snippet) = cx.sess().source_map().span_to_snippet(expr.span.source_callsite()) else {
        return false;
    };

    COUNTER_MARKERS.iter().any(|marker| snippet.contains(marker))
}

struct ExplicitExitVisitor {
    found: bool,
}

impl<'tcx> Visitor<'tcx> for ExplicitExitVisitor {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }

        match expr.kind {
            hir::ExprKind::Break(..) | hir::ExprKind::Ret(..) if !expr.span.from_expansion() => {
                self.found = true;
            }
            hir::ExprKind::Break(..) | hir::ExprKind::Ret(..) => {}
            hir::ExprKind::Closure(_) | hir::ExprKind::Loop(..) => {}
            _ => intravisit::walk_expr(self, expr),
        }
    }
}

fn block_has_explicit_exit<'tcx>(block: &'tcx hir::Block<'tcx>) -> bool {
    let mut visitor = ExplicitExitVisitor { found: false };
    visitor.visit_block(block);
    visitor.found
}

fn loop_has_structural_counter_bound(cx: &LateContext<'_>, expr: &hir::Expr<'_>, block: &hir::Block<'_>) -> bool {
    block_has_explicit_exit(block) && loop_has_counter_marker(cx, expr)
}

fn span_comes_from_tokio_select(cx: &LateContext<'_>, span: Span) -> bool {
    if !span.from_expansion() {
        return false;
    }

    let Ok(snippet) = cx.sess().source_map().span_to_snippet(span.source_callsite()) else {
        return false;
    };
    let normalized = normalized_without_whitespace(&snippet);
    if !normalized.starts_with("tokio::select!") && !normalized.starts_with("select!") {
        return false;
    }

    span.macro_backtrace().any(|expn| {
        let Some(def_id) = expn.macro_def_id else {
            return false;
        };
        let Some(crate_name) = cx.get_def_path(def_id).first().copied() else {
            return false;
        };
        matches!(crate_name.as_str(), "tokio" | "tokio_macros")
    })
}

struct TokioSelectVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for TokioSelectVisitor<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }

        if span_comes_from_tokio_select(self.cx, expr.span) {
            self.found = true;
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

fn block_contains_tokio_select<'tcx>(cx: &LateContext<'tcx>, block: &'tcx hir::Block<'tcx>) -> bool {
    let mut visitor = TokioSelectVisitor { cx, found: false };
    visitor.visit_block(block);
    visitor.found
}

fn while_let_header_snippet(cx: &LateContext<'_>, header_span: Span) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(header_span.source_callsite()).ok()?;
    Some(normalized_without_whitespace(&snippet))
}

fn header_is_recv_while_let(cx: &LateContext<'_>, header_span: Span) -> bool {
    let Some(normalized) = while_let_header_snippet(cx, header_span) else {
        return false;
    };
    normalized.starts_with("whileletSome(") && normalized.contains(".recv().await")
}

fn header_is_iter_next_while_let(cx: &LateContext<'_>, header_span: Span) -> bool {
    let Some(normalized) = while_let_header_snippet(cx, header_span) else {
        return false;
    };
    normalized.starts_with("whileletSome(") && normalized.contains(".next()")
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

fn recv_receiver_from_expr<'tcx>(expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let expr = peel_transparent_expr(expr);

    let hir::ExprKind::Match(scrutinee, _, hir::MatchSource::AwaitDesugar) = &expr.kind else {
        return None;
    };
    let hir::ExprKind::Call(_, [future_expr]) = &scrutinee.kind else {
        return None;
    };
    let hir::ExprKind::MethodCall(segment, receiver, args, _) = &future_expr.kind else {
        return None;
    };

    (segment.ident.name.as_str() == "recv" && args.is_empty()).then_some(*receiver)
}

fn while_let_recv_receiver<'tcx>(expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    let hir::ExprKind::Loop(block, _, hir::LoopSource::While, _) = &expr.kind else {
        return None;
    };
    let condition = block.expr?;
    let hir::ExprKind::If(cond, _, Some(_)) = &condition.kind else {
        return None;
    };
    let hir::ExprKind::Let(let_expr) = &cond.kind else {
        return None;
    };

    recv_receiver_from_expr(let_expr.init)
}

fn tokio_receiver_kind(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<TokioReceiverKind> {
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(expr).peel_refs().kind() else {
        return None;
    };

    if match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "Receiver"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "bounded", "Receiver"])
    {
        Some(TokioReceiverKind::Bounded)
    } else if match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "UnboundedReceiver"])
        || match_def_path(cx, adt.did(), &["tokio", "sync", "mpsc", "unbounded", "UnboundedReceiver"])
    {
        Some(TokioReceiverKind::Unbounded)
    } else {
        None
    }
}

fn generated_explicit_loop(expr: &hir::Expr<'_>, source_kind: LoopSourceKind) -> bool {
    // Compiler-generated `.await` polling loops arrive as expanded explicit loops;
    // user-written `while let` and `for` loops can also carry desugaring spans, so
    // expansion alone cannot suppress every loop source.
    source_kind == LoopSourceKind::ExplicitLoop && expr.span.from_expansion()
}

fn loop_bounds_facts<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    body: &'tcx hir::Block<'tcx>,
    loop_source: hir::LoopSource,
    header_span: Span,
) -> LoopBoundsFacts {
    let source_kind = loop_source_kind(loop_source);
    let receiver_kind = if header_is_recv_while_let(cx, header_span) {
        while_let_recv_receiver(expr).and_then(|receiver| tokio_receiver_kind(cx, receiver))
    } else {
        None
    };

    LoopBoundsFacts {
        source_kind,
        is_from_expansion: generated_explicit_loop(expr, source_kind),
        has_explicit_exit: block_has_explicit_exit(body),
        has_structural_counter_bound: loop_has_structural_counter_bound(cx, expr, body),
        has_cancellation_path: block_contains_tokio_select(cx, body),
        is_bounded_receiver_loop: receiver_kind == Some(TokioReceiverKind::Bounded),
        is_unbounded_receiver_loop: receiver_kind == Some(TokioReceiverKind::Unbounded),
        is_iter_next_loop: header_is_iter_next_while_let(cx, header_span),
    }
}

fn lint_unbounded_loop(cx: &LateContext<'_>, span: Span) {
    emit_tiger_style_lint(cx, UNBOUNDED_LOOP, span, "loop without visible iteration bound", "unbounded_loop");
}

impl<'tcx> LateLintPass<'tcx> for UnboundedLoop {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let hir::ExprKind::Loop(body, _, loop_source, header_span) = &expr.kind else {
            return;
        };

        let facts = loop_bounds_facts(cx, expr, body, *loop_source, *header_span);
        if classify_unbounded_loop(facts) {
            lint_unbounded_loop(cx, expr.span.source_callsite());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LoopBoundsFacts;
    use super::LoopSourceKind;
    use super::classify_unbounded_loop;

    fn explicit_loop() -> LoopBoundsFacts {
        LoopBoundsFacts {
            source_kind: LoopSourceKind::ExplicitLoop,
            is_from_expansion: false,
            has_explicit_exit: false,
            has_structural_counter_bound: false,
            has_cancellation_path: false,
            is_bounded_receiver_loop: false,
            is_unbounded_receiver_loop: false,
            is_iter_next_loop: false,
        }
    }

    #[test]
    fn classifies_explicit_loop_without_exit() {
        assert!(classify_unbounded_loop(explicit_loop()));
    }

    #[test]
    fn skips_explicit_exit_or_cancellation_path() {
        let with_exit = LoopBoundsFacts {
            has_explicit_exit: true,
            ..explicit_loop()
        };
        let with_cancellation = LoopBoundsFacts {
            has_cancellation_path: true,
            ..explicit_loop()
        };

        assert!(!classify_unbounded_loop(with_exit));
        assert!(!classify_unbounded_loop(with_cancellation));
    }

    #[test]
    fn marker_name_without_structural_bound_still_warns() {
        let marker_only = LoopBoundsFacts {
            has_structural_counter_bound: false,
            ..explicit_loop()
        };

        assert!(classify_unbounded_loop(marker_only));
    }

    #[test]
    fn bounded_receiver_is_clean_and_unbounded_receiver_warns() {
        let bounded = LoopBoundsFacts {
            source_kind: LoopSourceKind::While,
            is_bounded_receiver_loop: true,
            ..explicit_loop()
        };
        let unbounded = LoopBoundsFacts {
            source_kind: LoopSourceKind::While,
            is_unbounded_receiver_loop: true,
            ..explicit_loop()
        };

        assert!(!classify_unbounded_loop(bounded));
        assert!(classify_unbounded_loop(unbounded));
    }

    #[test]
    fn for_loops_and_expansion_loops_are_clean() {
        let for_loop = LoopBoundsFacts {
            source_kind: LoopSourceKind::For,
            ..explicit_loop()
        };
        let expansion_loop = LoopBoundsFacts {
            is_from_expansion: true,
            ..explicit_loop()
        };

        assert!(!classify_unbounded_loop(for_loop));
        assert!(!classify_unbounded_loop(expansion_loop));
    }
}
