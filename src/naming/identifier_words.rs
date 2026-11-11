//! Whole-word matching for identifier naming heuristics.

/// Returns whether `name` contains one of `words` as a whole identifier word.
///
/// Words are ASCII-case-insensitive and split at non-alphanumeric separators
/// such as `_` and at camelCase humps, so `report_lines` does not match `port`
/// while `connect_timeout`, `connectTimeout`, and `CONNECT_TIMEOUTS` all match
/// `timeout`. A trailing plural `s` or `es` is accepted.
pub(crate) fn identifier_has_word(name: &str, words: &[&str]) -> bool {
    identifier_segments(name).any(|segment| words.iter().any(|word| segment_matches_word(segment, word)))
}

fn segment_matches_word(segment: &str, word: &str) -> bool {
    let Some((stem, plural)) = segment.split_at_checked(word.len()) else {
        return false;
    };
    stem.eq_ignore_ascii_case(word) && ["", "s", "es"].iter().any(|suffix| plural.eq_ignore_ascii_case(suffix))
}

fn identifier_segments(name: &str) -> impl Iterator<Item = &str> {
    let mut rest = name;
    std::iter::from_fn(move || {
        rest = rest.trim_start_matches(|ch: char| !ch.is_ascii_alphanumeric());
        if rest.is_empty() {
            return None;
        }
        let bytes = rest.as_bytes();
        let end = (1..bytes.len()).find(|&index| is_segment_boundary(bytes, index)).unwrap_or(bytes.len());
        let (segment, tail) = rest.split_at(end);
        rest = tail;
        Some(segment)
    })
}

/// A segment ends before a separator, before `xY`-style camelCase humps, and
/// before the last capital of an acronym followed by lowercase (`HTTPServer`).
fn is_segment_boundary(bytes: &[u8], index: usize) -> bool {
    let current = bytes[index];
    if !current.is_ascii_alphanumeric() {
        return true;
    }
    if !current.is_ascii_uppercase() {
        return false;
    }
    let previous = bytes[index - 1];
    !previous.is_ascii_uppercase() || bytes.get(index + 1).is_some_and(u8::is_ascii_lowercase)
}

#[cfg(test)]
mod tests {
    use super::identifier_has_word;

    #[test]
    fn matches_whole_snake_case_and_screaming_words() {
        assert!(identifier_has_word("connect_timeout", &["timeout"]));
        assert!(identifier_has_word("CONNECT_TIMEOUT_MS", &["timeout"]));
        assert!(identifier_has_word("retry_attempts", &["retry"]));
        assert!(identifier_has_word("timeouts", &["timeout"]));
        assert!(identifier_has_word("max_batches", &["batch"]));
    }

    #[test]
    fn matches_camel_case_segments() {
        assert!(identifier_has_word("connectTimeout", &["timeout"]));
        assert!(identifier_has_word("HTTPTimeout", &["timeout"]));
        assert!(identifier_has_word("utf8Size", &["size"]));
    }

    #[test]
    fn rejects_embedded_substrings() {
        for name in [
            "proportion",
            "report_lines",
            "transport",
            "support",
            "import_count",
            "Exporter",
            "unlimited",
        ] {
            assert!(!identifier_has_word(name, &["port", "limit"]), "{name}");
        }
    }
}
