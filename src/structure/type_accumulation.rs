use rustc_hir as hir;
use rustc_hir::attrs::AttributeKind;
use rustc_hir::find_attr;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "type_accumulation";

declare_tool_lint! {
    /// Detects types/modules accumulating too many responsibilities.
    /// High impl-block or public-item counts are SRP review signals.
    pub tigerstyle::TYPE_ACCUMULATION,
    Allow,
    "type or module accumulates too many responsibilities"
}

pub struct TypeAccumulation {
    impl_blocks_max: usize,
    module_pub_items_max: usize,
    impl_counts: std::collections::HashMap<rustc_middle::ty::fast_reject::SimplifiedType, usize>,
}

impl TypeAccumulation {
    pub fn new(impl_blocks_max: usize, module_pub_items_max: usize) -> Self {
        Self {
            impl_blocks_max,
            module_pub_items_max,
            impl_counts: std::collections::HashMap::new(),
        }
    }
}

impl_lint_pass!(TypeAccumulation => [TYPE_ACCUMULATION]);

fn in_skipped_context(cx: &LateContext<'_>, hir_id: hir::HirId, span: Span) -> bool {
    span.from_expansion() || test_context::late_hir_id_is_in_test_context(cx, hir_id, span)
}

/// Reports a module at its declaration; the crate root has none, so it reports over the root file.
fn module_report_span(cx: &LateContext<'_>, module: &hir::Mod<'_>, hir_id: hir::HirId) -> Span {
    match cx.tcx.hir_node(hir_id) {
        hir::Node::Item(item) => item.span,
        _ => module.spans.inner_span,
    }
}

/// Counts the module's own public free items, for inline and file-backed modules alike.
/// Items inside nested modules, impl members, and struct fields are not counted, nor are
/// the public `#[rustc_main]` entry point and `#[rustc_test_marker]` consts that the test
/// harness adds under `--test`.
fn public_item_count(cx: &LateContext<'_>, module: &hir::Mod<'_>) -> usize {
    module
        .item_ids
        .iter()
        .filter(|item_id| {
            // `use a::{B, C}` lowers to `use a::B; use a::C;` plus a `use a::{}` stem carrying the same
            // visibility.
            !matches!(cx.tcx.hir_item(**item_id).kind, hir::ItemKind::Use(_, hir::UseKind::ListStem))
                && !find_attr!(
                    cx.tcx.hir_attrs(item_id.hir_id()),
                    AttributeKind::RustcMain | AttributeKind::RustcTestMarker(..)
                )
                && cx.tcx.visibility(item_id.owner_id.def_id).is_public()
        })
        .count()
}

impl<'tcx> LateLintPass<'tcx> for TypeAccumulation {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Impl(impl_block) = &item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() || in_skipped_context(cx, item.hir_id(), item.span) {
            return;
        }

        // Inherent impls are grouped by their resolved self type, as rustc groups them, so
        // spellings such as `T` and `self::T` share a count and same-named types do not.
        let self_ty = cx.tcx.type_of(item.owner_id).instantiate_identity();
        let treat_params = rustc_middle::ty::fast_reject::TreatParams::InstantiateWithInfer;
        let Some(key) = rustc_middle::ty::fast_reject::simplify_type(cx.tcx, self_ty, treat_params) else {
            return;
        };
        let count = self.impl_counts.entry(key).or_default();
        *count = count.saturating_add(1);
        if *count == self.impl_blocks_max.saturating_add(1) {
            // Name an ADT without the generic arguments of whichever impl crossed the limit.
            let type_name = match key {
                rustc_middle::ty::fast_reject::SimplifiedType::Adt(def_id) => cx.tcx.def_path_str(def_id),
                _ => self_ty.to_string(),
            };
            emit_tiger_style_lint(
                cx,
                TYPE_ACCUMULATION,
                item.span,
                format!(
                    "type `{type_name}` has more than {} inherent impl blocks; split responsibilities or consolidate cohesive behavior",
                    self.impl_blocks_max
                ),
                LINT_NAME,
            );
        }
    }

    fn check_mod(&mut self, cx: &LateContext<'tcx>, module: &'tcx hir::Mod<'tcx>, hir_id: hir::HirId) {
        let span = module_report_span(cx, module, hir_id);
        if in_skipped_context(cx, hir_id, span) {
            return;
        }

        let count = public_item_count(cx, module);
        if count > self.module_pub_items_max {
            emit_tiger_style_lint(
                cx,
                TYPE_ACCUMULATION,
                span,
                format!(
                    "module exposes {count} public items (limit: {}) — split unrelated public surface into focused modules",
                    self.module_pub_items_max
                ),
                LINT_NAME,
            );
        }
    }
}
