use std::collections::BTreeSet;

use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Ident;

use crate::config::IndexableLintExceptions;
use crate::diagnostics::emit_tiger_style_lint;
use crate::naming::indexable_exceptions::is_generated_source_span;
use crate::naming::indexable_exceptions::is_lint_identity_name;

/// Whole words that document a compatibility or externally mandated name;
/// `rapid` does not count as `api`.
const COMPATIBILITY_WORDS: &[&str] = &["compat", "compatibility", "external", "mandated", "api"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Repetition {
    repeated_word: String,
    path_segment: String,
}

declare_tool_lint! {
    /// Detects item names that repeat words already supplied by the crate or
    /// ancestor module path. Prefer `guest::spawn_vm` over
    /// `guest::spawn_guest_vm`.
    pub tigerstyle::PATH_SEGMENT_REPETITION,
    Warn,
    "item name repeats an ancestor path segment"
}

pub struct PathSegmentRepetition {
    exceptions: IndexableLintExceptions,
    workspace_root: std::path::PathBuf,
    /// The crate name when users write it as a path segment; see
    /// [`crate_name_is_path_segment`].
    crate_name_segment: Option<String>,
}

impl PathSegmentRepetition {
    pub fn new(exceptions: IndexableLintExceptions) -> Self {
        Self {
            exceptions,
            workspace_root: crate::source::workspace_root(),
            crate_name_segment: None,
        }
    }
}

impl_lint_pass!(PathSegmentRepetition => [PATH_SEGMENT_REPETITION]);

/// The non-library target kind Cargo marks in the compiler's environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CargoTargetMark {
    /// No mark: a library, a build script, or a plain `rustc` run.
    Unmarked,
    /// `CARGO_BIN_NAME`: binaries and examples, including their `--test`
    /// builds.
    Binary,
    /// `CARGO_TARGET_TMPDIR`: integration tests and benches.
    IntegrationTest,
}

/// How the crate being compiled was built.
#[derive(Clone, Copy, Debug)]
struct CrateFacts {
    /// `crate_types()` holds a library type such as `lib`, `rlib`, or `proc-macro`.
    is_library: bool,
    /// Built with `--test`.
    is_test_harness: bool,
    cargo_target: CargoTargetMark,
}

/// Only a library's crate name is a path segment users write
/// (`octet::naming::...`). Cargo derives the crate names of build scripts
/// (`build_script_build`), binaries, examples, integration tests, and benches
/// from target or file names that no path spells. The `--test` build of a
/// library keeps the library's name and its paths.
///
/// Build scripts and binaries compile as executables without `--test`.
/// Comparing `CARGO_CRATE_NAME` with `CARGO_PKG_NAME` would miss a custom
/// `[lib] name` and would match an integration test named after its package,
/// so the `--test` case relies on the variables Cargo sets for the other
/// target kinds instead.
fn crate_name_is_path_segment(facts: CrateFacts) -> bool {
    facts.is_library || (facts.is_test_harness && facts.cargo_target == CargoTargetMark::Unmarked)
}

fn identifier_for_item(item: &hir::Item<'_>) -> Option<Ident> {
    match item.kind {
        hir::ItemKind::Fn { ident, .. }
        | hir::ItemKind::Struct(ident, ..)
        | hir::ItemKind::Union(ident, ..)
        | hir::ItemKind::Enum(ident, ..)
        | hir::ItemKind::TyAlias(ident, ..)
        | hir::ItemKind::Const(ident, ..) => Some(ident),
        _ => None,
    }
}

fn normalized_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in name.chars() {
        if ch == '_' || ch == '-' || ch == ':' || ch == '<' || ch == '>' {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
        } else if ch.is_ascii_uppercase() {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            current.push(ch.to_ascii_lowercase());
        } else if ch.is_ascii_alphanumeric() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
    }

    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }

    words.into_iter().filter(|word| word.len() > 1).collect()
}

fn path_word_set(path_segments: &[&str]) -> BTreeSet<String> {
    path_segments.iter().flat_map(|segment| normalized_words(segment)).collect()
}

fn repeated_path_word(item_name: &str, path_segments: &[&str]) -> Option<Repetition> {
    let path_words = path_word_set(path_segments);
    normalized_words(item_name).into_iter().find_map(|item_word| {
        if !path_words.contains(&item_word) {
            return None;
        }

        let path_segment = path_segments
            .iter()
            .copied()
            .find(|segment| normalized_words(segment).contains(&item_word))
            .unwrap_or("<path>")
            .to_owned();

        Some(Repetition {
            repeated_word: item_word,
            path_segment,
        })
    })
}

/// The crate name, when it is a path segment, followed by the ancestor module
/// path of `item`. `def_path_str` omits the local crate name.
fn item_path_segments(cx: &LateContext<'_>, item: &hir::Item<'_>, crate_name_segment: Option<&str>) -> Vec<String> {
    let path = cx.tcx.def_path_str(item.owner_id.def_id.to_def_id());
    let mut segments = crate_name_segment
        .map(str::to_owned)
        .into_iter()
        .chain(path.split("::").map(str::to_owned))
        .collect::<Vec<_>>();
    segments.pop();
    segments
}

impl<'tcx> LateLintPass<'tcx> for PathSegmentRepetition {
    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        let facts = CrateFacts {
            is_library: cx
                .tcx
                .crate_types()
                .iter()
                .any(|crate_type| *crate_type != rustc_session::config::CrateType::Executable),
            is_test_harness: cx.sess().opts.test,
            cargo_target: if std::env::var_os("CARGO_BIN_NAME").is_some() {
                CargoTargetMark::Binary
            } else if std::env::var_os("CARGO_TARGET_TMPDIR").is_some() {
                CargoTargetMark::IntegrationTest
            } else {
                CargoTargetMark::Unmarked
            },
        };
        self.crate_name_segment = crate_name_is_path_segment(facts)
            .then(|| cx.tcx.crate_name(rustc_hir::def_id::LOCAL_CRATE).as_str().to_owned());
    }

    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if item.span.from_expansion() {
            return;
        }

        let Some(ident) = identifier_for_item(item) else {
            return;
        };

        if is_generated_source_span(cx.sess().source_map(), item.span, &self.workspace_root, self.exceptions) {
            return;
        }

        let segments = item_path_segments(cx, item, self.crate_name_segment.as_deref());
        if is_lint_identity_name(ident.name.as_str(), &segments, self.exceptions) {
            return;
        }
        let segment_refs = segments.iter().map(String::as_str).collect::<Vec<_>>();
        let Some(repetition) = repeated_path_word(ident.name.as_str(), &segment_refs) else {
            return;
        };
        if crate::naming::indexable_exceptions::item_documentation_mentions(cx, item, COMPATIBILITY_WORDS) {
            return;
        }

        emit_tiger_style_lint(
            cx,
            PATH_SEGMENT_REPETITION,
            item.span,
            format!(
                "item name `{}` repeats `{}` from ancestor path segment `{}`",
                ident.name.as_str(),
                repetition.repeated_word,
                repetition.path_segment
            ),
            "path_segment_repetition",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::CargoTargetMark;
    use super::CrateFacts;
    use super::crate_name_is_path_segment;
    use super::repeated_path_word;

    fn facts() -> CrateFacts {
        CrateFacts {
            is_library: false,
            is_test_harness: false,
            cargo_target: CargoTargetMark::Unmarked,
        }
    }

    #[test]
    fn only_library_crate_names_are_path_segments() {
        let library = CrateFacts {
            is_library: true,
            ..facts()
        };
        let library_unit_tests = CrateFacts {
            is_test_harness: true,
            ..facts()
        };
        // Build scripts and binaries compile as plain executables.
        let build_script_or_binary = facts();
        let example = CrateFacts {
            cargo_target: CargoTargetMark::Binary,
            ..facts()
        };
        let binary_unit_tests = CrateFacts {
            is_test_harness: true,
            cargo_target: CargoTargetMark::Binary,
            ..facts()
        };
        let integration_test_or_bench = CrateFacts {
            is_test_harness: true,
            cargo_target: CargoTargetMark::IntegrationTest,
            ..facts()
        };

        assert!(crate_name_is_path_segment(library));
        assert!(crate_name_is_path_segment(library_unit_tests));
        assert!(!crate_name_is_path_segment(build_script_or_binary));
        assert!(!crate_name_is_path_segment(example));
        assert!(!crate_name_is_path_segment(binary_unit_tests));
        assert!(!crate_name_is_path_segment(integration_test_or_bench));
    }

    #[test]
    fn crate_name_hyphens_separate_words() {
        let repetition = repeated_path_word("vm_config", &["octet-vm", "guest"]).expect("`vm` repeats the crate name");
        assert_eq!(repetition.repeated_word, "vm");
        assert_eq!(repetition.path_segment, "octet-vm");
        assert!(repeated_path_word("octetvm_config", &["octet-vm"]).is_none());
    }
}
