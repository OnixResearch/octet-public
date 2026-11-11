//! Bounded same-receiver retry after a handled synchronization error.

use octet_lint_metadata::durability::DurabilityBoundary;
use octet_lint_metadata::durability::DurabilityCatalog;
use rustc_hir as hir;
use rustc_hir::intravisit;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::ty::TyCtxt;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::durability::Context;
use crate::durability::scan;
use crate::durability::scan::Handling;
use crate::durability::scan::SurfaceCall;
use crate::durability::scan::SurfaceKind;
use crate::test_context;

pub(crate) const LINT_NAME: &str = "retry_sync_after_error";

declare_tool_lint! {
    /// Flags a synchronization retried on the same visible receiver after a
    /// handled failure without an intervening write, reopen, replacement,
    /// quarantine, or reconciliation in the analyzed block.
    pub tigerstyle::RETRY_SYNC_AFTER_ERROR,
    Warn,
    "same-receiver synchronization retry after a handled failure"
}

impl_lint_pass!(RetryAfterError => [RETRY_SYNC_AFTER_ERROR]);

/// The `retry_sync_after_error` late pass.
pub(crate) struct RetryAfterError {
    context: Option<Context>,
}

impl RetryAfterError {
    pub(crate) fn new(context: Option<Context>) -> Self {
        Self { context }
    }
}

impl<'tcx> LateLintPass<'tcx> for RetryAfterError {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        let Some(context) = self.context.as_ref() else {
            return;
        };
        if span.from_expansion() || test_context::late_hir_id_is_in_test_context(cx, body.value.hir_id, span) {
            return;
        }
        let def_path = cx.tcx.def_path_str(def_id);
        let Some(boundary) = context.boundary_at(cx.tcx.sess, span, &def_path) else {
            return;
        };
        let hir::ExprKind::Block(block, _) = body.value.kind else {
            return;
        };
        let statements = block
            .stmts
            .iter()
            .filter_map(|stmt| match stmt.kind {
                hir::StmtKind::Expr(expr) | hir::StmtKind::Semi(expr) => Some(expr),
                _ => None,
            })
            .chain(block.expr);
        let mut pending: Vec<String> = Vec::new();
        for statement in statements {
            let mut statement_scan = StatementScan::new(cx, context.catalog(), boundary);
            statement_scan.visit_expr(statement);
            if statement_scan.recovered {
                pending.clear();
                continue;
            }
            report_retries(context, cx, &statement_scan.calls, &mut pending);
        }
    }
}

fn report_retries(context: &Context, cx: &LateContext<'_>, calls: &[SurfaceCall], pending: &mut Vec<String>) {
    for call in calls {
        let Some(receiver) = call.receiver.as_ref() else {
            continue;
        };
        if pending.iter().any(|failed| failed == receiver) {
            let surface = match call.kind {
                SurfaceKind::Commit => "configured commit",
                _ => "synchronization",
            };
            let message = format!(
                "{surface} on `{receiver}` retries after a handled failure without an intervening write, reopen, replacement, quarantine, or reconciliation"
            );
            context.emit(cx, RETRY_SYNC_AFTER_ERROR, call.span.source_callsite(), message, LINT_NAME);
        }
        if call.handling == Handling::Handled && !pending.iter().any(|failed| failed == receiver) {
            pending.push(receiver.clone());
        }
    }
}

struct StatementScan<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    catalog: &'a DurabilityCatalog,
    boundary: &'a DurabilityBoundary,
    calls: Vec<SurfaceCall>,
    recovered: bool,
}

impl<'a, 'tcx> StatementScan<'a, 'tcx> {
    fn new(cx: &'a LateContext<'tcx>, catalog: &'a DurabilityCatalog, boundary: &'a DurabilityBoundary) -> Self {
        Self {
            cx,
            catalog,
            boundary,
            calls: Vec::new(),
            recovered: false,
        }
    }
}

impl<'tcx> Visitor<'tcx> for StatementScan<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        // Nested loops and closures are outside the bounded statement analysis:
        // their repetition and capture make a retry claim unreliable.
        if matches!(expr.kind, hir::ExprKind::Loop(..) | hir::ExprKind::Closure(..)) {
            return;
        }
        if scan::is_recovery_call(self.cx, self.catalog, expr) {
            self.recovered = true;
            return;
        }
        if let Some(kind) = scan::surface_kind(self.cx, self.catalog, self.boundary, expr)
            && kind.is_observation()
            && !expr.span.from_expansion()
        {
            let handling = scan::handling(self.cx, self.catalog, self.boundary, expr);
            let receiver = match expr.kind {
                hir::ExprKind::MethodCall(_, receiver, _, _) => scan::receiver_key(self.cx, receiver),
                _ => None,
            };
            self.calls.push(SurfaceCall {
                kind,
                handling,
                span: expr.span,
                receiver,
            });
        }
        intravisit::walk_expr(self, expr);
    }
}
