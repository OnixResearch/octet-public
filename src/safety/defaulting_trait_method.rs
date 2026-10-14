use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "defaulting_trait_method";

declare_tool_lint! {
    /// Detects trait methods with default bodies.
    /// Defaults can silently change behavioral contracts for implementors.
    pub tigerstyle::DEFAULTING_TRAIT_METHOD,
    Allow,
    "trait method has a default implementation"
}

pub struct DefaultingTraitMethod;

impl_lint_pass!(DefaultingTraitMethod => [DEFAULTING_TRAIT_METHOD]);

impl<'tcx> LateLintPass<'tcx> for DefaultingTraitMethod {
    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(_, hir::TraitFn::Provided(_)) = &item.kind else {
            return;
        };
        if item.span.from_expansion() || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            DEFAULTING_TRAIT_METHOD,
            item.span,
            "trait method has a default body; make the method required or document why the default is safe",
            LINT_NAME,
        );
    }
}
