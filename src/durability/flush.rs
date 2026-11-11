//! `Write::flush()` as the only durability observation.

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
use rustc_span::Span;

use crate::durability::Context;
use crate::durability::scan;
use crate::durability::scan::Handling;
use crate::test_context;

pub(crate) const LINT_NAME: &str = "flush_only_durability";

declare_tool_lint! {
    /// Flags declared durable boundaries whose only observation is
    /// `Write::flush()`, which drains user-space buffers rather than proving
    /// durable persistence.
    pub tigerstyle::FLUSH_ONLY_DURABILITY,
    Warn,
    "declared durable boundary observes only Write::flush()"
}

impl_lint_pass!(FlushOnly => [FLUSH_ONLY_DURABILITY]);

/// The `flush_only_durability` late pass.
pub(crate) struct FlushOnly {
    context: Option<Context>,
}

impl FlushOnly {
    pub(crate) fn new(context: Option<Context>) -> Self {
        Self { context }
    }
}

impl<'tcx> LateLintPass<'tcx> for FlushOnly {
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
        let mut finder = ObservationFinder {
            cx,
            context,
            boundary,
            handled_observation: false,
            first_flush: None,
        };
        finder.visit_expr(body.value);
        if finder.handled_observation {
            return;
        }
        let Some(flush_span) = finder.first_flush else {
            return;
        };
        context.emit(
            cx,
            FLUSH_ONLY_DURABILITY,
            flush_span.source_callsite(),
            "declared durable boundary ends after `flush()`; `Write::flush()` drains user-space buffers and is not a durable-persistence observation".to_owned(),
            LINT_NAME,
        );
    }
}

struct ObservationFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    context: &'a Context,
    boundary: &'a DurabilityBoundary,
    handled_observation: bool,
    first_flush: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for ObservationFinder<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        self.observe(expr);
        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> ObservationFinder<'_, 'tcx> {
    fn observe(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let Some(kind) = scan::surface_kind(self.cx, self.context.catalog(), self.boundary, expr) else {
            return;
        };
        if kind.is_observation() {
            let handling = scan::handling(self.cx, self.context.catalog(), self.boundary, expr);
            self.handled_observation |= matches!(handling, Handling::Handled | Handling::Propagated);
            return;
        }
        if self.first_flush.is_none() && !expr.span.from_expansion() {
            self.first_flush = Some(expr.span);
        }
    }
}
