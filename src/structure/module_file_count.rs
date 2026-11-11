use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use rustc_ast::Crate;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::SyntaxContext;

use crate::diagnostics::emit_tiger_style_lint;

/// Whole path components, below the workspace root, that mark generated or vendored source.
/// `registry` is deliberately absent: Cargo registry sources live under `.cargo` or outside the
/// workspace, while a workspace `registry` component is an ordinary module name.
const IGNORED_SOURCE_COMPONENTS: &[&str] = &["target", "generated", "OUT_DIR", "rustlib", ".cargo"];
const VENDOR_CARGO_DEPS_MARKER: &str = "vendor-cargo-deps";

declare_tool_lint! {
    /// Detects module directories with too many Rust source files
    /// (default: 7).
    pub tigerstyle::MODULE_FILE_COUNT,
    Warn,
    "module directory contains too many Rust source files"
}

pub struct ModuleFileCount {
    max_files: usize,
}

impl ModuleFileCount {
    pub fn new(max_files: usize) -> Self {
        Self { max_files }
    }
}

impl_lint_pass!(ModuleFileCount => [MODULE_FILE_COUNT]);

#[derive(Clone)]
struct ModuleFile {
    path: PathBuf,
    span: Span,
}

fn is_ignored_source_component(component: &str) -> bool {
    IGNORED_SOURCE_COMPONENTS.contains(&component) || component.contains(VENDOR_CARGO_DEPS_MARKER)
}

fn has_ignored_source_component(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        is_ignored_source_component(&value)
    })
}

fn is_countable_rust_source(relative_path: &Path) -> bool {
    relative_path.extension().is_some_and(|extension| extension == "rs") && !has_ignored_source_component(relative_path)
}

/// Only components below the workspace root count, so a workspace checked out under a
/// directory such as `target` is still counted.
fn is_countable_workspace_rust_source(path: &Path, workspace_root: &Path) -> bool {
    crate::source::is_workspace_source_path(path, workspace_root)
        && is_countable_rust_source(path.strip_prefix(workspace_root).unwrap_or(path))
}

fn directory_display(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl EarlyLintPass for ModuleFileCount {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _krate: &Crate) {
        let mut by_directory: BTreeMap<PathBuf, Vec<ModuleFile>> = BTreeMap::new();
        let workspace_root = crate::source::workspace_root();

        for source_file in cx.sess().source_map().files().iter() {
            if !crate::source::is_local(source_file) {
                continue;
            }
            let path = crate::source::normalize_path(&crate::source::source_file_path(source_file));
            if !is_countable_workspace_rust_source(&path, &workspace_root) {
                continue;
            }
            let Some(parent) = path.parent() else {
                continue;
            };

            by_directory.entry(parent.to_path_buf()).or_default().push(ModuleFile {
                path,
                span: Span::new(source_file.start_pos, source_file.end_position(), SyntaxContext::root(), None),
            });
        }

        for (directory, mut files) in by_directory {
            let count = files.len();
            let max = self.max_files;
            if count <= max {
                continue;
            }

            files.sort_by(|left, right| left.path.cmp(&right.path));
            emit_tiger_style_lint(
                cx,
                MODULE_FILE_COUNT,
                files[0].span,
                format!(
                    "module directory `{}` contains {count} Rust source files (limit: {max})",
                    directory_display(&directory)
                ),
                "module_file_count",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_countable_rust_source;
    use super::is_countable_workspace_rust_source;
    use crate::source::is_workspace_source_path;

    fn workspace_root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn counts_workspace_rust_sources() {
        assert!(is_countable_rust_source(Path::new("crates/example/src/lib.rs")));
    }

    #[test]
    fn counts_existing_workspace_source_paths() {
        assert!(is_countable_workspace_rust_source(Path::new("src/lib.rs"), workspace_root()));
        assert!(is_countable_workspace_rust_source(&workspace_root().join("src/lib.rs"), workspace_root()));
    }

    #[test]
    fn skips_remapped_dependency_paths_absent_from_workspace() {
        assert!(!is_workspace_source_path(Path::new("<WORKSPACE>/src/address_lookup.rs"), workspace_root()));
    }

    #[test]
    fn skips_vendored_dependency_sources() {
        assert!(!is_countable_rust_source(Path::new(
            "/nix/store/hash-vendor-cargo-deps/serde_core-1.0.228/src/crate_root.rs"
        )));
    }

    #[test]
    fn counts_ordinary_registry_modules() {
        assert!(is_countable_workspace_rust_source(
            &workspace_root().join("policy_inputs/registry/registry_module_01.rs"),
            workspace_root()
        ));
        assert!(is_countable_rust_source(Path::new("src/registry/mod.rs")));
    }

    #[test]
    fn skips_cargo_registry_sources() {
        let registry_source = "registry/src/index.crates.io-1949cf8c6b5b557f/serde-1.0.228/src/de/mod.rs";
        // The default `CARGO_HOME` lies outside the workspace.
        assert!(!is_countable_workspace_rust_source(
            &Path::new("/home/user/.cargo").join(registry_source),
            workspace_root()
        ));
        // A `CARGO_HOME` inside the workspace sits below its `.cargo` component.
        assert!(!is_countable_workspace_rust_source(
            &Path::new("/workspace/.cargo").join(registry_source),
            Path::new("/workspace")
        ));
    }

    #[test]
    fn ignores_components_above_the_workspace_root() {
        let root = Path::new("/home/user/target/registry/workspace");
        assert!(is_countable_workspace_rust_source(&root.join("src/lib.rs"), root));
        assert!(!is_countable_workspace_rust_source(&root.join("target/debug/build/out/bindings.rs"), root));
    }
}
