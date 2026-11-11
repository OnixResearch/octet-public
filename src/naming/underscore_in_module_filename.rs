use std::path::Path;

use rustc_ast::Crate;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::SyntaxContext;

use crate::config::IndexableLintExceptions;
use crate::diagnostics::emit_tiger_style_lint;
use crate::naming::indexable_exceptions::is_lint_identity_file_stem;

const IGNORED_SOURCE_COMPONENTS: &[&str] = &[
    "target",
    "ui_tests",
    "generated",
    "OUT_DIR",
    "rustlib",
    "registry",
    ".cargo",
];
const VENDOR_CARGO_DEPS_MARKER: &str = "vendor-cargo-deps";

declare_tool_lint! {
    /// Detects module filenames with underscore-separated compound names.
    /// Prefer directory structure such as `compute/vm.rs` over `compute_vm.rs`.
    pub tigerstyle::UNDERSCORE_IN_MODULE_FILENAME,
    Warn,
    "module filename contains an underscore"
}

pub struct UnderscoreInModuleFilename {
    exceptions: IndexableLintExceptions,
}

impl UnderscoreInModuleFilename {
    pub fn new(exceptions: IndexableLintExceptions) -> Self {
        Self { exceptions }
    }
}

impl_lint_pass!(UnderscoreInModuleFilename => [UNDERSCORE_IN_MODULE_FILENAME]);

fn path_components(path: &Path) -> Vec<String> {
    path.components().map(|component| component.as_os_str().to_string_lossy().into_owned()).collect()
}

fn is_ignored_source_component(component: &str) -> bool {
    IGNORED_SOURCE_COMPONENTS.contains(&component) || component.contains(VENDOR_CARGO_DEPS_MARKER)
}

fn is_generated_or_fixture_path(path: &Path) -> bool {
    let components = path_components(path);
    if components.iter().any(|component| component == "policy_inputs")
        && !components.iter().any(|component| component == "underscore_module_filename")
    {
        return true;
    }

    components.iter().any(|component| is_ignored_source_component(component))
        || components.windows(2).any(|window| window[0] == "tests" && window[1] == "fixtures")
        || path.to_string_lossy().ends_with(".rs.in")
}

fn file_stem(path: &Path) -> Option<&str> {
    path.file_stem().and_then(|stem| stem.to_str())
}

fn replacement_layout(stem: &str) -> String {
    let mut segments = stem.split('_').filter(|segment| !segment.is_empty()).collect::<Vec<_>>();
    let Some(file_name) = segments.pop() else {
        return stem.to_owned();
    };
    if segments.is_empty() {
        return format!("{file_name}.rs");
    }
    format!("{}/{}.rs", segments.join("/"), file_name)
}

/// `crate_root` is the file rustc compiled as the crate root. Its name is the
/// Cargo target's name (`src/bin/load_gen.rs`, `examples/basic_usage.rs`), so a
/// rename would break the target and it never warns.
fn should_warn(
    path: &Path,
    crate_root: &Path,
    workspace_root: &Path,
    exceptions: IndexableLintExceptions,
) -> Option<(String, String)> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") || path == crate_root {
        return None;
    }
    if !crate::source::is_workspace_source_path(path, workspace_root) || is_generated_or_fixture_path(path) {
        return None;
    }

    let stem = file_stem(path)?;
    if !stem.contains('_') || is_lint_identity_file_stem(stem, exceptions) {
        return None;
    }

    Some((stem.to_owned(), replacement_layout(stem)))
}

impl EarlyLintPass for UnderscoreInModuleFilename {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &Crate) {
        let workspace_root = crate::source::workspace_root();
        let source_map = cx.sess().source_map();
        let crate_root = crate::source::normalize_path(&crate::source::source_file_path(
            &source_map.lookup_source_file(krate.spans.inner_span.lo()),
        ));

        for source_file in source_map.files().iter() {
            if !crate::source::is_local(source_file) {
                continue;
            }
            let path = crate::source::normalize_path(&crate::source::source_file_path(source_file));
            let Some((stem, replacement)) = should_warn(&path, &crate_root, &workspace_root, self.exceptions) else {
                continue;
            };

            emit_tiger_style_lint(
                cx,
                UNDERSCORE_IN_MODULE_FILENAME,
                Span::new(source_file.start_pos, source_file.end_position(), SyntaxContext::root(), None),
                format!("module filename `{stem}.rs` contains an underscore; prefer `{replacement}`"),
                "underscore_in_module_filename",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::should_warn;
    use crate::config::IndexableLintExceptions;
    use crate::source::is_workspace_source_path;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    fn all_exceptions() -> IndexableLintExceptions {
        IndexableLintExceptions {
            rustc_private_aliases: true,
            lint_identity_names: true,
            generated_searchable_names: true,
        }
    }

    fn warn_in_library(path: &str) -> Option<(String, String)> {
        should_warn(Path::new(path), Path::new("src/lib.rs"), workspace_root(), all_exceptions())
    }

    #[test]
    fn warns_for_workspace_underscore_module_filename() {
        assert!(warn_in_library("src/test_context.rs").is_some());
    }

    #[test]
    fn skips_underscore_crate_root() {
        let crate_root = Path::new("src/test_context.rs");
        assert!(should_warn(crate_root, crate_root, workspace_root(), all_exceptions()).is_none());
    }

    #[test]
    fn accepts_local_filename_without_underscores() {
        assert!(warn_in_library("src/source.rs").is_none());
    }

    #[test]
    fn skips_lint_identity_module_filename() {
        assert!(warn_in_library("src/naming/acronym_style.rs").is_none());
    }

    #[test]
    fn skips_remapped_dependency_paths_absent_from_workspace() {
        assert!(!is_workspace_source_path(Path::new("<WORKSPACE>/src/address_lookup.rs"), workspace_root()));
        assert!(warn_in_library("<WORKSPACE>/src/address_lookup.rs").is_none());
    }

    #[test]
    fn skips_vendored_dependency_underscore_filename() {
        assert!(warn_in_library("/nix/store/hash-vendor-cargo-deps/serde_core-1.0.228/src/crate_root.rs").is_none());
    }
}
