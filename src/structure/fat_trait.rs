use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "fat_trait";
const FN_TOKEN: &str = "fn ";

declare_tool_lint! {
    /// Detects traits with more than a configurable number of methods.
    /// Fat interfaces violate ISP by forcing implementors to depend on
    /// methods they do not use.
    pub tigerstyle::FAT_TRAIT,
    Warn,
    "trait has too many methods"
}

pub struct FatTrait {
    max_methods: usize,
}

impl FatTrait {
    pub fn new(max_methods: usize) -> Self {
        Self { max_methods }
    }
}

impl_lint_pass!(FatTrait => [FAT_TRAIT]);

fn trait_method_count(cx: &LateContext<'_>, span: rustc_span::Span) -> usize {
    cx.sess().source_map().span_to_snippet(span).map_or(0, |snippet| snippet.matches(FN_TOKEN).count())
}

impl<'tcx> LateLintPass<'tcx> for FatTrait {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if !matches!(item.kind, hir::ItemKind::Trait(..))
            || item.span.from_expansion()
            || test_context::late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
        {
            return;
        }

        let method_count = trait_method_count(cx, item.span);
        if method_count <= self.max_methods {
            return;
        }

        emit_tiger_style_lint(
            cx,
            FAT_TRAIT,
            item.span,
            format!(
                "trait has {method_count} methods (limit: {}) — split it into cohesive role interfaces",
                self.max_methods
            ),
            LINT_NAME,
        );
    }
}
