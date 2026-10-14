use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

declare_tool_lint! {
    /// Detects functions with more than a configurable number of parameters.
    /// Tiger Style prefers grouping related options into a named struct instead
    /// of growing long positional signatures.
    pub tigerstyle::TOO_MANY_PARAMETERS,
    Warn,
    "function takes too many parameters"
}

pub struct TooManyParameters {
    max_parameters: usize,
}

impl TooManyParameters {
    pub fn new(max_parameters: usize) -> Self {
        Self { max_parameters }
    }

    fn lint_if_needed(&self, cx: &LateContext<'_>, hir_id: hir::HirId, span: rustc_span::Span, count: usize) {
        if test_context::late_hir_id_is_in_test_context(cx, hir_id, span) || count <= self.max_parameters {
            return;
        }

        emit_tiger_style_lint(
            cx,
            TOO_MANY_PARAMETERS,
            span,
            format!("function has {count} parameters (limit: {}) — consider an options struct", self.max_parameters),
            "too_many_parameters",
        );
    }
}

impl_lint_pass!(TooManyParameters => [TOO_MANY_PARAMETERS]);

impl<'tcx> LateLintPass<'tcx> for TooManyParameters {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, .. } = &item.kind else {
            return;
        };
        self.lint_if_needed(cx, item.hir_id(), item.span, sig.decl.inputs.len());
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &impl_item.kind else {
            return;
        };
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(impl_item.owner_id.def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() {
            return;
        }

        let has_self = sig.decl.implicit_self != hir::ImplicitSelfKind::None;
        self.lint_if_needed(
            cx,
            impl_item.hir_id(),
            impl_item.span,
            sig.decl.inputs.len().saturating_sub(usize::from(has_self)),
        );
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(sig, _) = &trait_item.kind else {
            return;
        };
        let has_self = sig.decl.implicit_self != hir::ImplicitSelfKind::None;
        self.lint_if_needed(
            cx,
            trait_item.hir_id(),
            trait_item.span,
            sig.decl.inputs.len().saturating_sub(usize::from(has_self)),
        );
    }
}
