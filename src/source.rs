//! Compiler-owned source provenance for file-level lint passes.

// r[impl octet.lint_signal_quality.source_ownership]
pub(crate) fn is_local(file: &rustc_span::SourceFile) -> bool {
    !file.is_imported()
}

pub(crate) fn source_file_path(file: &rustc_span::SourceFile) -> std::path::PathBuf {
    std::path::PathBuf::from(file.name.prefer_local_unconditionally().to_string_lossy().into_owned())
}

pub(crate) fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut normalized = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

/// Cargo runs rustc from the workspace root. Files outside it, such as build
/// script output under a custom `CARGO_TARGET_DIR`, are not workspace source.
pub(crate) fn workspace_root() -> std::path::PathBuf {
    std::env::current_dir().map(|path| normalize_path(&path)).unwrap_or_default()
}

pub(crate) fn is_workspace_source_path(path: &std::path::Path, workspace_root: &std::path::Path) -> bool {
    if path.is_absolute() {
        return path.starts_with(workspace_root);
    }

    workspace_root.join(path).is_file()
}

/// Normalizes one configured workspace-relative source scope the way
/// `cargo octet` does: trims whitespace, strips leading `./` and trailing `/`.
pub(crate) fn normalize_scope(raw: &str) -> String {
    let mut scope = raw.trim();
    while let Some(rest) = scope.strip_prefix("./") {
        scope = rest.trim_start_matches('/');
    }
    scope.trim_end_matches('/').to_owned()
}

/// Whether a workspace-relative source path is one of `scopes` or lies below
/// one. Matching is by whole path components, so `crates/core` covers
/// `crates/core/src/lib.rs` but not `crates/core-shell/src/lib.rs`.
pub(crate) fn path_in_scopes(path: &std::path::Path, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| !scope.is_empty() && path.starts_with(scope))
}

/// Where a normalized source scope points once resolved on disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScopeResolution {
    /// The scope's canonical path, relative to the canonical workspace root.
    Inside(String),
    /// The workspace root itself.
    Root,
    /// A path outside the workspace root.
    Outside,
    /// No existing file or directory.
    Missing,
}

/// Resolves a normalized scope against the canonical workspace root,
/// following symlinks and `..` the way `cargo octet` validates scopes.
pub(crate) fn resolve_scope(canonical_root: &std::path::Path, scope: &str) -> ScopeResolution {
    let Ok(resolved) = canonical_root.join(scope).canonicalize() else {
        return ScopeResolution::Missing;
    };
    match resolved.strip_prefix(canonical_root) {
        Ok(relative) if relative.as_os_str().is_empty() => ScopeResolution::Root,
        Ok(relative) => ScopeResolution::Inside(relative.to_string_lossy().into_owned()),
        Err(_) => ScopeResolution::Outside,
    }
}

/// Whether `span` comes from a file inside one of the normalized `scopes`.
/// Cargo runs rustc from the workspace root, so an absolute file name is made
/// relative to rustc's working directory before matching.
pub(crate) fn span_in_scopes(sess: &rustc_session::Session, span: rustc_span::Span, scopes: &[String]) -> bool {
    if scopes.is_empty() {
        return false;
    }
    let file = sess.source_map().lookup_source_file(span.lo());
    let path = normalize_path(&source_file_path(&file));
    let relative = sess
        .opts
        .working_dir
        .local_path()
        .map(normalize_path)
        .and_then(|root| path.strip_prefix(root).ok().map(std::path::Path::to_path_buf));
    path_in_scopes(relative.as_deref().unwrap_or(&path), scopes)
}

/// Whether the source text at `ident`'s span spells `ident`, so the user wrote
/// that token. Attribute proc macros such as `#[tokio::main]` build code with
/// `quote_spanned!` at a user token: the generated `.expect` then has no
/// expansion context, yet its span covers other user text, such as a `;`.
pub(crate) fn ident_is_spelled_at_its_span(sess: &rustc_session::Session, ident: rustc_span::Ident) -> bool {
    sess.source_map().span_to_snippet(ident.span).is_ok_and(|snippet| {
        let spelled = snippet.strip_prefix("r#").unwrap_or(&snippet);
        spelled == ident.as_str()
    })
}

/// The comment lines that document an item from above, nearest first: comments
/// between its outer attributes and the contiguous comment lines above the
/// earliest attribute. `prefix` is the source text before the item. A comment
/// line must touch the attribute, comment, or item below it; attributes may be
/// separated by blank lines.
///
/// Attributes are recognised from source text because HIR cannot report the
/// span of every parsed attribute, such as `#[inline]` or `#[repr(C)]`.
pub(crate) fn leading_comment_lines(prefix: &str) -> impl Iterator<Item = &str> {
    let mut rest = prefix;
    std::iter::from_fn(move || {
        loop {
            let code = rest.trim_end();
            let line_start = code.rfind('\n').map_or(0, |index| index + 1);
            let last_line = code[line_start..].trim_start();
            if last_line.starts_with("//") {
                let has_blank_line_below = rest[code.len()..].matches('\n').nth(1).is_some();
                if has_blank_line_below {
                    return None;
                }
                rest = &code[..line_start];
                return Some(last_line);
            }
            rest = &code[..outer_attribute_start(code)?];
        }
    })
}

/// Byte offset of the `#[` that opens the outer attribute ending `code`.
fn outer_attribute_start(code: &str) -> Option<usize> {
    if !code.ends_with(']') {
        return None;
    }
    let mut depth = 0_usize;
    for (index, byte) in code.bytes().enumerate().rev() {
        match byte {
            b']' => depth += 1,
            b'[' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    let hash = index.checked_sub(1)?;
                    return (code.as_bytes()[hash] == b'#').then_some(hash);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether any of [`leading_comment_lines`] for the item at `item_span`
/// satisfies `predicate`.
pub(crate) fn any_leading_comment_line(
    source_map: &rustc_span::source_map::SourceMap,
    item_span: rustc_span::Span,
    predicate: impl FnMut(&str) -> bool,
) -> bool {
    let file = source_map.lookup_source_file(item_span.lo());
    let offset = rustc_span::Pos::to_usize(&(item_span.lo() - file.start_pos));
    file.src
        .as_deref()
        .and_then(|src| src.get(..offset))
        .is_some_and(|prefix| leading_comment_lines(prefix).any(predicate))
}

#[cfg(test)]
mod tests {
    fn fixture() -> rustc_span::SourceFile {
        rustc_span::SourceFile::new(
            rustc_span::FileName::Real(rustc_span::RealFileName::from_virtual_path(std::path::Path::new(
                "/workspace/patches/example/bad_file.rs",
            ))),
            "pub struct Example;".to_owned(),
            rustc_span::SourceFileHashAlgorithm::Blake3,
            None,
        )
        .expect("small fixture source fits the compiler offset range")
    }

    // r[verify octet.lint_signal_quality.source_ownership]
    #[test]
    fn local_source_is_admitted_even_under_a_patch_directory() {
        rustc_span::create_default_session_globals_then(|| {
            let file = fixture();
            assert!(super::is_local(&file));
        });
    }

    #[test]
    fn imported_source_is_rejected_without_changing_its_path() {
        rustc_span::create_default_session_globals_then(|| {
            let mut file = fixture();
            assert!(super::is_local(&file));
            // Rustc records imported source separately and leaves `src` absent.
            file.src = None;
            assert!(!super::is_local(&file));
        });
    }

    #[test]
    fn scopes_normalize_like_the_runner() {
        for raw in [
            "crates/domain/src",
            "./crates/domain/src",
            " crates/domain/src/ ",
            "././crates/domain/src//",
        ] {
            assert_eq!(super::normalize_scope(raw), "crates/domain/src", "{raw:?}");
        }
        assert_eq!(super::normalize_scope(" ./ "), "");
    }

    #[test]
    fn scopes_match_whole_path_components() {
        let scopes = vec![
            super::normalize_scope("./crates/core/"),
            super::normalize_scope("src/lib.rs"),
        ];
        let inside = ["crates/core/src/lib.rs", "crates/core", "src/lib.rs"];
        let outside = [
            "crates/core-shell/src/lib.rs",
            "vendor/crates/core/src/lib.rs",
            "src/lib.rs.bak",
            "src/main.rs",
        ];

        for path in inside {
            assert!(super::path_in_scopes(std::path::Path::new(path), &scopes), "{path} is in scope");
        }
        for path in outside {
            assert!(!super::path_in_scopes(std::path::Path::new(path), &scopes), "{path} is out of scope");
        }
        assert!(!super::path_in_scopes(std::path::Path::new("src/lib.rs"), &[String::new()]));
    }

    #[cfg(unix)]
    #[test]
    fn scopes_resolve_through_symlinks_and_parent_components() {
        use super::ScopeResolution;

        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after the epoch")
            .as_nanos();
        let scratch = std::env::temp_dir().join(format!("octet-scope-resolution-{}-{unique}", std::process::id()));
        let root = scratch.join("workspace");
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::create_dir_all(root.join("crates/shell")).expect("crate directory");
        std::fs::create_dir_all(scratch.join("outside")).expect("outside directory");
        std::fs::write(root.join("src/lib.rs"), "").expect("source file");
        std::os::unix::fs::symlink(root.join("src"), root.join("linked")).expect("in-root symlink");
        std::os::unix::fs::symlink(scratch.join("outside"), root.join("escape")).expect("escaping symlink");
        let root = root.canonicalize().expect("canonical root");

        let cases = [
            ("src", ScopeResolution::Inside("src".to_owned())),
            ("src/lib.rs", ScopeResolution::Inside("src/lib.rs".to_owned())),
            ("linked", ScopeResolution::Inside("src".to_owned())),
            ("crates/shell/../../src", ScopeResolution::Inside("src".to_owned())),
            ("../outside", ScopeResolution::Outside),
            ("escape", ScopeResolution::Outside),
            ("crates/..", ScopeResolution::Root),
            ("missing/src", ScopeResolution::Missing),
        ];
        let resolved = cases.iter().map(|(scope, _expected)| super::resolve_scope(&root, scope)).collect::<Vec<_>>();
        std::fs::remove_dir_all(&scratch).expect("remove scratch directory");

        for ((scope, expected), actual) in cases.iter().zip(resolved) {
            assert_eq!(&actual, expected, "{scope}");
        }
    }

    #[test]
    fn leading_comment_lines_cross_attributes_and_stop_at_gaps() {
        let prefix = "struct Before;\n\n// detached\n\n// first\n/// second\n#[inline]\n// between\n#[cfg_attr(\n    test,\n    derive(Debug)\n)]\n\n#[doc(hidden)]\n    ";
        let lines = super::leading_comment_lines(prefix).collect::<Vec<_>>();
        assert_eq!(lines, ["// between", "/// second", "// first"]);
    }

    #[test]
    fn leading_comment_lines_stop_at_code_and_inner_attributes() {
        assert_eq!(super::leading_comment_lines("const A: [u8; 1] = [0];\n").count(), 0);
        assert_eq!(super::leading_comment_lines("#![allow(dead_code)]\n").count(), 0);
        assert_eq!(super::leading_comment_lines("// note\n\nfn f() {}\n").count(), 0);
        assert_eq!(super::leading_comment_lines("mod m { ").count(), 0);
        assert_eq!(super::leading_comment_lines("// note\n").collect::<Vec<_>>(), ["// note"]);
    }
}
