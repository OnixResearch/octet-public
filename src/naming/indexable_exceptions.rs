use crate::config::IndexableLintExceptions;
use crate::config::canonical_lint_names;

const RUSTC_PRIVATE_ALIASES: &[(&str, &str)] = &[
    ("rustc_abi", "abi"),
    ("rustc_ast", "ast"),
    ("rustc_errors", "errors"),
    ("rustc_hir", "hir"),
    ("rustc_lint", "lint"),
    ("rustc_middle", "middle"),
    ("rustc_session", "session"),
    ("rustc_span", "span"),
];

/// Whole path components that mark generated or vendored source below the
/// workspace root. `registry` is deliberately absent: Cargo registry sources
/// live under `.cargo` or outside the workspace, while a workspace `registry`
/// component is an ordinary module name.
const GENERATED_SOURCE_COMPONENTS: &[&str] = &["target", "generated", "OUT_DIR", "rustlib", ".cargo"];
const VENDOR_CARGO_DEPS_MARKER: &str = "vendor-cargo-deps";

pub(crate) fn is_canonical_rustc_private_alias(
    path_segments: &[String],
    alias: &str,
    exceptions: IndexableLintExceptions,
) -> bool {
    if !exceptions.rustc_private_aliases {
        return false;
    }

    let Some(source) = path_segments.last().map(String::as_str) else {
        return false;
    };
    RUSTC_PRIVATE_ALIASES
        .iter()
        .any(|(rustc_crate, canonical_alias)| source == *rustc_crate && alias == *canonical_alias)
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

fn normalized_identity(name: &str) -> String {
    normalized_words(name).join("_")
}

pub(crate) fn is_lint_identity_name(
    item_name: &str,
    path_segments: &[String],
    exceptions: IndexableLintExceptions,
) -> bool {
    if !exceptions.lint_identity_names {
        return false;
    }

    let item_identity = normalized_identity(item_name);
    canonical_lint_names().into_iter().any(|lint_name| {
        let lint_identity = normalized_identity(lint_name);
        item_identity.contains(&lint_identity)
            && path_segments.iter().any(|segment| normalized_identity(segment).contains(&lint_identity))
    })
}

pub(crate) fn is_lint_identity_file_stem(stem: &str, exceptions: IndexableLintExceptions) -> bool {
    exceptions.lint_identity_names && canonical_lint_names().into_iter().any(|lint_name| stem == lint_name)
}

/// `generated_searchable_names` exception: source outside the workspace root
/// (build-script output under a custom target dir, registry or vendored crates,
/// the sysroot) is generated, and inside it only whole path components below
/// the root count, so ordinary modules such as `src/registry.rs` or
/// `src/targeting.rs` stay linted.
pub(crate) fn is_generated_source_path(
    path: &std::path::Path,
    workspace_root: &std::path::Path,
    exceptions: IndexableLintExceptions,
) -> bool {
    if !exceptions.generated_searchable_names {
        return false;
    }
    if !crate::source::is_workspace_source_path(path, workspace_root) {
        return true;
    }

    path.strip_prefix(workspace_root).unwrap_or(path).components().any(|component| {
        let component = component.as_os_str().to_string_lossy();
        GENERATED_SOURCE_COMPONENTS.contains(&component.as_ref()) || component.contains(VENDOR_CARGO_DEPS_MARKER)
    })
}

/// Applies [`is_generated_source_path`] to the source file that contains `span`.
pub(crate) fn is_generated_source_span(
    source_map: &rustc_span::source_map::SourceMap,
    span: rustc_span::Span,
    workspace_root: &std::path::Path,
    exceptions: IndexableLintExceptions,
) -> bool {
    if !exceptions.generated_searchable_names {
        return false;
    }

    let file = source_map.lookup_source_file(span.lo());
    let path = crate::source::normalize_path(&crate::source::source_file_path(&file));
    is_generated_source_path(&path, workspace_root, exceptions)
}

/// Whether the documentation of `item` names one of `words` as a whole word.
/// Documentation is the item's doc attributes plus the comment lines between
/// its outer attributes and directly above them, so a note above
/// `#[doc(hidden)]` or on the first line of a multi-line doc comment counts.
/// For an import inside a `{...}` group, the lines above its `use` statement
/// count.
pub(crate) fn item_documentation_mentions(
    cx: &rustc_lint::LateContext<'_>,
    item: &rustc_hir::Item<'_>,
    words: &[&str],
) -> bool {
    let mentions = |text: &str| crate::naming::identifier_words::identifier_has_word(text, words);
    cx.tcx
        .hir_attrs(item.hir_id())
        .iter()
        .filter_map(rustc_hir::Attribute::doc_str)
        .any(|doc| mentions(doc.as_str()))
        || crate::source::any_leading_comment_line(cx.tcx.sess.source_map(), use_statement_span(cx, item), mentions)
}

/// rustc gives an import nested in a `{...}` group the span of its own tree,
/// such as `Config as LegacyConfig`, which does not reach the comments above
/// the `use` statement. Returns the span of the outermost group stem (the
/// `ListStem` item, whose span is the whole statement) that contains `item`,
/// or `item.span` for any other item.
fn use_statement_span(cx: &rustc_lint::LateContext<'_>, item: &rustc_hir::Item<'_>) -> rustc_span::Span {
    if !matches!(item.kind, rustc_hir::ItemKind::Use(_, rustc_hir::UseKind::Single(_) | rustc_hir::UseKind::Glob)) {
        return item.span;
    }
    let (module, ..) = cx.tcx.hir_get_module(cx.tcx.parent_module(item.hir_id()));
    module
        .item_ids
        .iter()
        .map(|item_id| cx.tcx.hir_item(*item_id))
        .filter(|stem| {
            matches!(stem.kind, rustc_hir::ItemKind::Use(_, rustc_hir::UseKind::ListStem))
                && stem.span.contains(item.span)
        })
        .map(|stem| stem.span)
        .min_by_key(|span| span.lo())
        .unwrap_or(item.span)
}

#[cfg(test)]
mod tests {
    use super::is_canonical_rustc_private_alias;
    use super::is_generated_source_path;
    use super::is_lint_identity_file_stem;
    use super::is_lint_identity_name;
    use crate::config::IndexableLintExceptions;

    fn all_exceptions() -> IndexableLintExceptions {
        IndexableLintExceptions {
            rustc_private_aliases: true,
            lint_identity_names: true,
            generated_searchable_names: true,
        }
    }

    #[test]
    fn accepts_canonical_rustc_private_aliases() {
        let path = vec!["rustc_hir".to_owned()];

        assert!(is_canonical_rustc_private_alias(&path, "hir", all_exceptions()));
        assert!(!is_canonical_rustc_private_alias(&path, "Hidden", all_exceptions()));
    }

    #[test]
    fn respects_disabled_rustc_private_alias_exception() {
        let path = vec!["rustc_hir".to_owned()];
        let exceptions = IndexableLintExceptions {
            rustc_private_aliases: false,
            ..all_exceptions()
        };

        assert!(!is_canonical_rustc_private_alias(&path, "hir", exceptions));
    }

    #[test]
    fn accepts_lint_identity_names_only_when_path_preserves_same_identity() {
        let lint_path = vec![
            "src".to_owned(),
            "naming".to_owned(),
            "path_segment_repetition".to_owned(),
        ];
        let consumer_path = vec!["src".to_owned(), "guest".to_owned()];

        assert!(is_lint_identity_name("path_segment_repetition_classifier", &lint_path, all_exceptions()));
        assert!(!is_lint_identity_name("spawn_guest_vm", &consumer_path, all_exceptions()));
    }

    #[test]
    fn accepts_lint_identity_file_stems() {
        assert!(is_lint_identity_file_stem("path_segment_repetition", all_exceptions()));
        assert!(!is_lint_identity_file_stem("address_lookup", all_exceptions()));
    }

    fn workspace_root() -> &'static std::path::Path {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn generated_source_is_decided_by_whole_path_components() {
        let root = workspace_root();

        assert!(is_generated_source_path(&root.join("target/debug/build/out/bindings.rs"), root, all_exceptions()));
        assert!(is_generated_source_path(
            &root.join("policy_inputs/generated_searchable_names/non_trait_imports/generated/imports.rs"),
            root,
            all_exceptions()
        ));
        assert!(!is_generated_source_path(&root.join("src/registry.rs"), root, all_exceptions()));
        assert!(!is_generated_source_path(&root.join("src/registry/mod.rs"), root, all_exceptions()));
        assert!(!is_generated_source_path(&root.join("src/targeting.rs"), root, all_exceptions()));
        assert!(!is_generated_source_path(&root.join("src/regenerated.rs"), root, all_exceptions()));
    }

    #[test]
    fn generated_source_ignores_components_above_the_workspace_root() {
        let root = std::path::Path::new("/home/user/target/generated/workspace");

        assert!(!is_generated_source_path(&root.join("src/lib.rs"), root, all_exceptions()));
    }

    #[test]
    fn source_outside_the_workspace_root_is_generated() {
        let root = workspace_root();

        assert!(is_generated_source_path(
            std::path::Path::new("/custom-target/debug/build/example/out/bindings.rs"),
            root,
            all_exceptions()
        ));
        assert!(is_generated_source_path(
            std::path::Path::new("/nix/store/hash-vendor-cargo-deps/serde-1.0.0/src/lib.rs"),
            root,
            all_exceptions()
        ));
    }

    #[test]
    fn respects_disabled_generated_searchable_names_exception() {
        let root = workspace_root();
        let exceptions = IndexableLintExceptions {
            generated_searchable_names: false,
            ..all_exceptions()
        };

        assert!(!is_generated_source_path(&root.join("target/debug/build/out/bindings.rs"), root, exceptions));
        assert!(!is_generated_source_path(
            std::path::Path::new("/custom-target/debug/build/example/out/bindings.rs"),
            root,
            exceptions
        ));
    }
}
