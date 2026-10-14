use rustc_hir as hir;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_span::Span;

struct UseSiteCollector<'cx, 'tcx> {
    cx: &'cx LateContext<'tcx>,
    target: hir::HirId,
    use_sites: Vec<Span>,
    has_expansion: bool,
}

impl<'tcx> Visitor<'tcx> for UseSiteCollector<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if let hir::ExprKind::Path(ref qpath) = expr.kind {
            let res = self.cx.typeck_results().qpath_res(qpath, expr.hir_id);
            if let hir::def::Res::Local(hir_id) = res
                && hir_id == self.target
            {
                if expr.span.from_expansion() {
                    self.has_expansion = true;
                }
                self.use_sites.push(expr.span);
            }
        }
        intravisit::walk_expr(self, expr);
    }
}

pub(crate) fn enclosing_body<'tcx>(cx: &LateContext<'tcx>, hir_id: hir::HirId) -> &'tcx hir::Body<'tcx> {
    let body_owner = cx.tcx.hir_enclosing_body_owner(hir_id);
    cx.tcx.hir_body_owned_by(body_owner)
}

pub(crate) fn collect_local_use_sites<'tcx>(
    cx: &LateContext<'tcx>,
    body: &'tcx hir::Body<'tcx>,
    binding_hir_id: hir::HirId,
) -> Option<Vec<Span>> {
    let mut collector = UseSiteCollector {
        cx,
        target: binding_hir_id,
        use_sites: Vec::new(),
        has_expansion: false,
    };

    collector.visit_expr(body.value);

    if collector.has_expansion {
        return None;
    }

    Some(collector.use_sites)
}
