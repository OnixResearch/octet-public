//! Discarded synchronization and configured commit results.

use octet_lint_metadata::durability::DurabilityBoundary;
use rustc_hir as hir;
use rustc_hir::intravisit;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::ty::TyCtxt;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::durability::Context;
use crate::durability::scan;
use crate::durability::scan::Handling;
use crate::durability::scan::SurfaceKind;
use crate::test_context;

pub(crate) const LINT_NAME: &str = "discarded_sync_result";

declare_tool_lint! {
    /// Flags discarded `sync_all`, `sync_data`, and configured commit results in
    /// consumer-declared durability boundaries.
    pub tigerstyle::DISCARDED_SYNC_RESULT,
    Warn,
    "discarded synchronization or configured commit result in a declared durability boundary"
}

impl_lint_pass!(SyncDiscard => [DISCARDED_SYNC_RESULT]);

/// The `discarded_sync_result` late pass.
pub(crate) struct SyncDiscard {
    context: Option<Context>,
}

impl SyncDiscard {
    pub(crate) fn new(context: Option<Context>) -> Self {
        Self { context }
    }
}

impl<'tcx> LateLintPass<'tcx> for SyncDiscard {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: rustc_span::Span,
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
        DiscardFinder { cx, context, boundary }.visit_expr(body.value);
    }
}

struct DiscardFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    context: &'a Context,
    boundary: &'a DurabilityBoundary,
}

impl<'tcx> Visitor<'tcx> for DiscardFinder<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if !expr.span.from_expansion() {
            self.report(expr);
        }
        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> DiscardFinder<'_, 'tcx> {
    fn report(&self, expr: &'tcx hir::Expr<'tcx>) {
        let Some(kind) = scan::surface_kind(self.cx, self.context.catalog(), self.boundary, expr) else {
            return;
        };
        if !kind.is_observation() {
            return;
        }
        let handling = scan::handling(self.cx, self.context.catalog(), self.boundary, expr);
        let Some(message) = finding_message(kind, handling) else {
            return;
        };
        self.context.emit(self.cx, DISCARDED_SYNC_RESULT, expr.span.source_callsite(), message, LINT_NAME);
    }
}

fn finding_message(kind: SurfaceKind, handling: Handling) -> Option<String> {
    let surface = match kind {
        SurfaceKind::Synchronization => "synchronization",
        SurfaceKind::Commit => "configured commit",
        SurfaceKind::Adapter | SurfaceKind::Flush => return None,
    };
    match handling {
        Handling::Discarded => {
            Some(format!("{surface} result is discarded; handle the failure or record an explicit uncertain outcome"))
        }
        Handling::EmptyError => Some(format!(
            "{surface} failure is ignored by an empty error arm; handle the failure or record an explicit uncertain outcome"
        )),
        Handling::SuccessMapping => {
            Some(format!("{surface} failure is mapped to success without an explicit uncertain outcome"))
        }
        Handling::Handled | Handling::Propagated | Handling::Unanalyzed => None,
    }
}
