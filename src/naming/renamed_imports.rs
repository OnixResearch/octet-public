use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Ident;

use crate::config::IndexableLintExceptions;
use crate::diagnostics::emit_tiger_style_lint;
use crate::naming::indexable_exceptions::is_canonical_rustc_private_alias;
use crate::naming::indexable_exceptions::is_generated_source_span;

/// Whole words that document a compatibility alias; `incompatible` does not count.
const COMPATIBILITY_WORDS: &[&str] = &["compat", "compatibility"];
const PRELUDE_SEGMENT: &str = "prelude";
const UNDERSCORE_IMPORT: &str = "_";

declare_tool_lint! {
    /// Detects `use ... as ...` imports that hide the original searchable name.
    /// Trait imports renamed to `_` for method resolution and documented public
    /// compatibility aliases are intentionally allowed.
    pub tigerstyle::RENAMED_IMPORTS,
    Warn,
    "renamed import hides the original searchable name"
}

pub struct RenamedImports {
    exceptions: IndexableLintExceptions,
    workspace_root: std::path::PathBuf,
}

impl RenamedImports {
    pub fn new(exceptions: IndexableLintExceptions) -> Self {
        Self {
            exceptions,
            workspace_root: crate::source::workspace_root(),
        }
    }
}

impl_lint_pass!(RenamedImports => [RENAMED_IMPORTS]);

fn path_segments(path: &hir::UsePath<'_>) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.name.as_str().to_owned())
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn source_name(path: &hir::UsePath<'_>, ident: Ident) -> String {
    path_segments(path).last().cloned().unwrap_or_else(|| ident.name.as_str().to_owned())
}

fn is_prelude(path: &hir::UsePath<'_>) -> bool {
    path_segments(path).iter().any(|segment| segment == PRELUDE_SEGMENT)
}

fn is_trait_res(res: Option<Res>) -> bool {
    matches!(res, Some(Res::Def(DefKind::Trait | DefKind::TraitAlias, _)))
}

fn imports_only_trait(path: &hir::UsePath<'_>) -> bool {
    is_trait_res(path.res.type_ns) && path.res.value_ns.is_none() && path.res.macro_ns.is_none()
}

impl<'tcx> LateLintPass<'tcx> for RenamedImports {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Use(path, hir::UseKind::Single(ident)) = item.kind else {
            return;
        };

        if item.span.from_expansion()
            || is_prelude(path)
            || is_generated_source_span(cx.sess().source_map(), item.span, &self.workspace_root, self.exceptions)
        {
            return;
        }

        let source = source_name(path, ident);
        let alias = ident.name.as_str();
        if alias == source {
            return;
        }

        if alias == UNDERSCORE_IMPORT && imports_only_trait(path) {
            return;
        }

        if is_canonical_rustc_private_alias(&path_segments(path), alias, self.exceptions) {
            return;
        }

        if cx.tcx.visibility(item.owner_id.def_id).is_public()
            && crate::naming::indexable_exceptions::item_documentation_mentions(cx, item, COMPATIBILITY_WORDS)
        {
            return;
        }

        emit_tiger_style_lint(
            cx,
            RENAMED_IMPORTS,
            item.span,
            format!("renamed import `{source} as {alias}` hides the original searchable name"),
            "renamed_imports",
        );
    }
}
