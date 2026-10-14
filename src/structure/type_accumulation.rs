use std::collections::BTreeMap;

use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "type_accumulation";
const PUBLIC_ITEM_TOKEN: &str = "pub ";

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
    impl_counts: BTreeMap<String, usize>,
}

impl TypeAccumulation {
    pub fn new(impl_blocks_max: usize, module_pub_items_max: usize) -> Self {
        Self {
            impl_blocks_max,
            module_pub_items_max,
            impl_counts: BTreeMap::new(),
        }
    }
}

impl_lint_pass!(TypeAccumulation => [TYPE_ACCUMULATION]);

fn span_snippet(cx: &LateContext<'_>, span: Span) -> String {
    cx.sess().source_map().span_to_snippet(span).unwrap_or_default()
}

fn public_item_count(cx: &LateContext<'_>, span: Span) -> usize {
    span_snippet(cx, span).matches(PUBLIC_ITEM_TOKEN).count()
}

impl<'tcx> LateLintPass<'tcx> for TypeAccumulation {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if item.span.from_expansion() || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span) {
            return;
        }

        match &item.kind {
            hir::ItemKind::Impl(impl_block) if impl_block.of_trait.is_none() => {
                let type_name = span_snippet(cx, impl_block.self_ty.span).trim().to_owned();
                if type_name.is_empty() {
                    return;
                }
                let count = self.impl_counts.entry(type_name.clone()).or_default();
                *count = count.saturating_add(1);
                if *count == self.impl_blocks_max.saturating_add(1) {
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
            hir::ItemKind::Mod(..) => {
                let count = public_item_count(cx, item.span);
                if count > self.module_pub_items_max {
                    emit_tiger_style_lint(
                        cx,
                        TYPE_ACCUMULATION,
                        item.span,
                        format!(
                            "module exposes {count} public items (limit: {}) — split unrelated public surface into focused modules",
                            self.module_pub_items_max
                        ),
                        LINT_NAME,
                    );
                }
            }
            _ => {}
        }
    }
}
