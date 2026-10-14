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

#[cfg(test)]
mod tests {
    use super::is_canonical_rustc_private_alias;
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
}
