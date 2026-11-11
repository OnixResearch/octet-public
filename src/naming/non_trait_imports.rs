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

const PRELUDE_SEGMENT: &str = "prelude";

declare_tool_lint! {
    /// Detects non-trait `use` imports that hide the searchable owner path.
    /// Prefer qualified owner paths at the use site unless the import is a
    /// trait brought into scope for method resolution or a public re-export.
    pub tigerstyle::NON_TRAIT_IMPORTS,
    Warn,
    "non-trait item imported into local scope"
}

pub struct NonTraitImports {
    exceptions: IndexableLintExceptions,
    workspace_root: std::path::PathBuf,
}

impl NonTraitImports {
    pub fn new(exceptions: IndexableLintExceptions) -> Self {
        Self {
            exceptions,
            workspace_root: crate::source::workspace_root(),
        }
    }
}

impl_lint_pass!(NonTraitImports => [NON_TRAIT_IMPORTS]);

fn path_segments(path: &hir::UsePath<'_>) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.name.as_str().to_owned())
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn is_prelude(path: &hir::UsePath<'_>) -> bool {
    path_segments(path).iter().any(|segment| segment == PRELUDE_SEGMENT)
}

fn is_trait_res(res: Option<Res>) -> bool {
    matches!(res, Some(Res::Def(DefKind::Trait | DefKind::TraitAlias, _)))
}

fn is_derive_macro_res(res: Option<Res>) -> bool {
    matches!(
        res,
        Some(Res::Def(DefKind::Macro(kinds), _)) if kinds == rustc_hir::def::MacroKinds::DERIVE
    )
}

/// A trait import may also bring in the same-named derive macro (`std::hash::Hash`,
/// `serde::Serialize`); it still imports the trait for method resolution.
fn imports_trait(path: &hir::UsePath<'_>) -> bool {
    is_trait_res(path.res.type_ns)
        && path.res.value_ns.is_none()
        && (path.res.macro_ns.is_none() || is_derive_macro_res(path.res.macro_ns))
}

fn imported_name(path: &hir::UsePath<'_>, ident: Ident) -> String {
    let segments = path_segments(path);
    let source = segments.last().cloned().unwrap_or_else(|| ident.name.as_str().to_owned());
    if ident.name.as_str() == source {
        source
    } else {
        format!("{source} as {}", ident.name)
    }
}

impl<'tcx> LateLintPass<'tcx> for NonTraitImports {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Use(path, hir::UseKind::Single(ident)) = item.kind else {
            return;
        };

        if item.span.from_expansion()
            || is_prelude(path)
            || is_generated_source_span(cx.sess().source_map(), item.span, &self.workspace_root, self.exceptions)
            || cx.tcx.visibility(item.owner_id.def_id).is_public()
            || imports_trait(path)
            || is_canonical_rustc_private_alias(&path_segments(path), ident.name.as_str(), self.exceptions)
        {
            return;
        }

        let name = imported_name(path, ident);
        emit_tiger_style_lint(
            cx,
            NON_TRAIT_IMPORTS,
            item.span,
            format!("non-trait import `{name}` hides the owner path; prefer the qualified path at the use site"),
            "non_trait_imports",
        );
    }
}
