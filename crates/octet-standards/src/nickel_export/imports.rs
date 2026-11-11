//! Static Nickel import observation for export dependency closures.
//!
//! The shell supplies exact source and dependency texts. This pure scanner
//! reports every `import` target reachable from the source through those
//! texts, so the standalone core can compare the observed closure with the
//! declared dependencies. Comments and string contents are skipped, while
//! string interpolations are scanned as code. Quoted targets resolve against
//! the importing file's directory; package imports keep their bare name.
//! A dependency's text is scanned only when Nickel parses it as Nickel source:
//! its `as 'Format` tag says so, or, untagged, its extension is not a data one.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::Path;

use super::NamedText;

const IMPORT_KEYWORD: &[u8] = b"import";
/// Extensions Nickel infers as data formats for untagged imports.
const DATA_EXTENSIONS: &[&str] = &["json", "yaml", "yml", "toml", "txt"];
/// Format tag that makes Nickel parse an imported file as Nickel source.
const NICKEL_FORMAT_TAG: &str = "Nickel";

/// Lexical scanner state for one nesting level.
enum Mode {
    /// Nickel code; `braces` counts open `{` inside an interpolation.
    Code { braces: usize },
    /// Standard `"…"` string.
    Standard,
    /// Multiline (`m%"`) or symbolic (`prefix-s%"`) string whose opener has
    /// `percents` `%` characters.
    Multiline { percents: usize },
}

/// Return the sorted, deduplicated import closure observed from `source_path`.
///
/// Imports are followed through dependency texts whose normalized path matches
/// a resolved import Nickel parses as Nickel source. The source itself is never
/// reported.
pub(super) fn observed_import_closure(
    source_path: &str,
    source_text: &str,
    dependencies: &[NamedText<'_>],
) -> Vec<String> {
    let source = normalize_path(source_path);
    let mut observed = BTreeSet::new();
    let mut scanned = BTreeSet::from([source.clone()]);
    let mut pending = vec![(source.clone(), source_text)];
    while let Some((importer, text)) = pending.pop() {
        for target in import_targets(text) {
            let (resolved, nickel_source) = match target {
                ImportTarget::Path { path, nickel_source } => (resolve_import(&importer, &path), nickel_source),
                ImportTarget::Package(name) => (name, true),
            };
            if resolved == source {
                continue;
            }
            observed.insert(resolved.clone());
            if nickel_source
                && !scanned.contains(&resolved)
                && let Some(dependency) =
                    dependencies.iter().find(|dependency| normalize_path(dependency.path) == resolved)
            {
                scanned.insert(resolved.clone());
                pending.push((resolved, dependency.text));
            }
        }
    }
    observed.into_iter().collect()
}

/// One `import` target as written in source.
#[derive(Debug, Eq, PartialEq)]
enum ImportTarget {
    /// Quoted file path and whether Nickel parses the file as Nickel source.
    Path { path: String, nickel_source: bool },
    /// Bare package identifier.
    Package(String),
}

fn import_targets(text: &str) -> Vec<ImportTarget> {
    let bytes = text.as_bytes();
    let mut targets = Vec::new();
    let mut stack = vec![Mode::Code { braces: 0 }];
    let mut pos = 0;
    while pos < bytes.len() {
        let nested = stack.len() > 1;
        pos = match stack.last_mut() {
            Some(Mode::Code { braces }) => match bytes[pos] {
                b'#' => skip_line(bytes, pos),
                b'"' => {
                    stack.push(Mode::Standard);
                    pos + 1
                }
                b'\'' => match bytes.get(pos + 1) {
                    Some(b'"') => {
                        stack.push(Mode::Standard);
                        pos + 2
                    }
                    Some(byte) if is_identifier_start(*byte) => identifier_end(bytes, pos + 1),
                    _ => pos + 1,
                },
                b'{' => {
                    *braces += 1;
                    pos + 1
                }
                b'}' => {
                    if *braces > 0 {
                        *braces -= 1;
                    } else if nested {
                        stack.pop();
                    }
                    pos + 1
                }
                byte if byte.is_ascii_digit() => number_end(bytes, pos),
                byte if is_identifier_start(byte) => {
                    let end = identifier_end(bytes, pos);
                    if let Some(percents) = string_opener_percents(bytes, &bytes[pos..end], end) {
                        stack.push(Mode::Multiline { percents });
                        end + percents + 1
                    } else if &bytes[pos..end] == IMPORT_KEYWORD
                        && let Some((target, next)) = import_target(bytes, end)
                    {
                        targets.push(target);
                        next
                    } else {
                        end
                    }
                }
                _ => pos + 1,
            },
            Some(Mode::Standard) => match bytes[pos] {
                b'\\' => pos + 2,
                b'"' => {
                    stack.pop();
                    pos + 1
                }
                b'%' if bytes.get(pos + 1) == Some(&b'{') => {
                    stack.push(Mode::Code { braces: 0 });
                    pos + 2
                }
                _ => pos + 1,
            },
            Some(Mode::Multiline { percents }) => match bytes[pos] {
                // Nickel lexes `"%…{` and `%…{` as one candidate interpolation
                // and `"%…` as a candidate end. Interpolation needs at least the
                // opener's `%` count, and any extra leading `%` stay literal.
                // An end needs at least that count; a longer run is a Nickel
                // lexical error.
                byte @ (b'"' | b'%') => {
                    let quote = usize::from(byte == b'"');
                    let run = run_while(bytes, pos + quote, |byte| byte == b'%');
                    let after = pos + quote + run;
                    if run > 0 && bytes.get(after) == Some(&b'{') {
                        if run >= *percents {
                            stack.push(Mode::Code { braces: 0 });
                        }
                        after + 1
                    } else {
                        if quote == 1 && run >= *percents {
                            stack.pop();
                        }
                        after
                    }
                }
                _ => pos + 1,
            },
            None => bytes.len(),
        };
    }
    targets
}

/// Read the target after an `import` keyword ending at `pos`.
fn import_target(bytes: &[u8], pos: usize) -> Option<(ImportTarget, usize)> {
    let pos = skip_trivia(bytes, pos)?;
    if bytes[pos] == b'"' {
        let (path, end) = static_string(bytes, pos + 1)?;
        let (nickel_source, next) = match import_format(bytes, end) {
            Some((tag, next)) => (tag == NICKEL_FORMAT_TAG, next),
            None => (inferred_nickel_source(&path), end),
        };
        return Some((ImportTarget::Path { path, nickel_source }, next));
    }
    if is_identifier_start(bytes[pos]) {
        let end = identifier_end(bytes, pos);
        return Some((ImportTarget::Package(String::from_utf8_lossy(&bytes[pos..end]).into_owned()), end));
    }
    None
}

/// Read the optional `as 'Format` or `as '"Format"` after an import path
/// ending at `pos`, returning the tag name and the position after it.
fn import_format(bytes: &[u8], pos: usize) -> Option<(String, usize)> {
    let pos = skip_trivia(bytes, pos)?;
    let end = identifier_end(bytes, pos);
    if &bytes[pos..end] != b"as" {
        return None;
    }
    let pos = skip_trivia(bytes, end)?;
    if bytes[pos] != b'\'' {
        return None;
    }
    match *bytes.get(pos + 1)? {
        b'"' => static_string(bytes, pos + 2),
        byte if is_identifier_start(byte) => {
            let end = identifier_end(bytes, pos + 1);
            Some((String::from_utf8_lossy(&bytes[pos + 1..end]).into_owned(), end))
        }
        _ => None,
    }
}

/// Read a static string whose opening quote ends at `pos`, returning its
/// unescaped value and the position after the closing quote.
fn static_string(bytes: &[u8], mut pos: usize) -> Option<(String, usize)> {
    let mut value = Vec::new();
    loop {
        match *bytes.get(pos)? {
            b'"' => return Some((String::from_utf8_lossy(&value).into_owned(), pos + 1)),
            b'\\' => {
                value.push(match *bytes.get(pos + 1)? {
                    b'n' => b'\n',
                    b't' => b'\t',
                    b'r' => b'\r',
                    escaped => escaped,
                });
                pos += 2;
            }
            byte => {
                value.push(byte);
                pos += 1;
            }
        }
    }
}

/// Skip whitespace and comments from `pos`, returning the next token start.
fn skip_trivia(bytes: &[u8], mut pos: usize) -> Option<usize> {
    loop {
        match bytes.get(pos)? {
            byte if byte.is_ascii_whitespace() => pos += 1,
            b'#' => pos = skip_line(bytes, pos),
            _ => return Some(pos),
        }
    }
}

/// Resolve a quoted import against the importing file's directory.
///
/// Absolute targets stay absolute and parent traversal beyond the root stays
/// visible, so the core rejects both as unsafe undeclared dependencies.
fn resolve_import(importer: &str, target: &str) -> String {
    if target.starts_with('/') {
        return target.to_owned();
    }
    let mut components = importer.split('/').filter(|component| !component.is_empty()).collect::<Vec<_>>();
    components.pop();
    push_components(&mut components, target);
    components.join("/")
}

fn normalize_path(path: &str) -> String {
    let mut components = Vec::new();
    push_components(&mut components, path.trim());
    components.join("/")
}

fn push_components<'a>(components: &mut Vec<&'a str>, path: &'a str) {
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." if components.last().is_some_and(|last| *last != "..") => {
                components.pop();
            }
            other => components.push(other),
        }
    }
}

/// Infer whether Nickel parses an untagged import as Nickel source.
///
/// Like Nickel, this reads the extension of the path as written; an unknown or
/// missing extension falls back to Nickel source.
fn inferred_nickel_source(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .is_none_or(|extension| !DATA_EXTENSIONS.contains(&extension))
}

fn skip_line(bytes: &[u8], pos: usize) -> usize {
    bytes[pos..].iter().position(|byte| *byte == b'\n').map_or(bytes.len(), |offset| pos + offset + 1)
}

/// Count the `%` of a multiline (`m%"`) or symbolic (`prefix-s%"`) string
/// opener whose identifier part `word` ends at `end`.
///
/// Nickel only lexes these openers when the whole identifier token is `m` or
/// a letter-led prefix ending in `-s`, so `foom%"` is `foom % "…"`.
fn string_opener_percents(bytes: &[u8], word: &[u8], end: usize) -> Option<usize> {
    let opener = word == b"m" || (word.len() >= 3 && word[0].is_ascii_alphabetic() && word.ends_with(b"-s"));
    let percents = run_while(bytes, end, |byte| byte == b'%');
    (opener && percents > 0 && bytes.get(end + percents) == Some(&b'"')).then_some(percents)
}

/// Return the end of the numeric literal starting at `pos`, following Nickel's
/// decimal, exponent, and `0x`/`0o`/`0b` forms, so its letters never start an
/// identifier.
fn number_end(bytes: &[u8], pos: usize) -> usize {
    let radix_digits = match bytes.get(pos..pos + 2) {
        Some(b"0x") => run_while(bytes, pos + 2, |byte| byte.is_ascii_hexdigit()),
        Some(b"0o") => run_while(bytes, pos + 2, |byte| matches!(byte, b'0'..=b'7')),
        Some(b"0b") => run_while(bytes, pos + 2, |byte| matches!(byte, b'0' | b'1')),
        _ => 0,
    };
    if radix_digits > 0 {
        return pos + 2 + radix_digits;
    }
    let mut end = pos + run_while(bytes, pos, |byte| byte.is_ascii_digit());
    if bytes.get(end) == Some(&b'.') {
        let fraction = run_while(bytes, end + 1, |byte| byte.is_ascii_digit());
        if fraction > 0 {
            end += 1 + fraction;
        }
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let sign = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
        let exponent = run_while(bytes, end + 1 + sign, |byte| byte.is_ascii_digit());
        if exponent > 0 {
            end += 1 + sign + exponent;
        }
    }
    end
}

fn run_while(bytes: &[u8], pos: usize, predicate: impl Fn(u8) -> bool) -> usize {
    bytes.get(pos..).map_or(0, |rest| rest.iter().take_while(|byte| predicate(**byte)).count())
}

fn identifier_end(bytes: &[u8], pos: usize) -> usize {
    bytes[pos..]
        .iter()
        .position(|byte| !is_identifier_byte(*byte))
        .map_or(bytes.len(), |offset| pos + offset)
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\'' | b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_reports_code_and_interpolated_imports_only() {
        let text = concat!(
            "# import \"comment.ncl\"\n",
            "let a = import \"a.ncl\" in\n",
            "let b = import # comment between keyword and target\n  \"dir/b.ncl\" in\n",
            "let reimport = 1 in\n",
            "let pkg = import some_package in\n",
            "let text = import \"text.ncl\" as 'Text in\n",
            "{\n",
            "  s = \"import \\\"string.ncl\\\" %{ import \"interpolated.ncl\" }\",\n",
            "  m = m%%\"import \"multiline.ncl\" %{ ignored } %%{ import \"multi-interp.ncl\" }\"%%,\n",
            "  r = { nested = { x = 1 } },\n",
            "  after = import \"after.ncl\",\n",
            "}\n",
        );

        let targets = import_targets(text);

        assert_eq!(targets, vec![
            nickel_path("a.ncl"),
            nickel_path("dir/b.ncl"),
            ImportTarget::Package("some_package".to_owned()),
            ImportTarget::Path {
                path: "text.ncl".to_owned(),
                nickel_source: false,
            },
            nickel_path("interpolated.ncl"),
            nickel_path("multi-interp.ncl"),
            nickel_path("after.ncl"),
        ]);
    }

    fn nickel_path(path: &str) -> ImportTarget {
        ImportTarget::Path {
            path: path.to_owned(),
            nickel_source: true,
        }
    }

    #[test]
    fn string_delimiters_and_interpolations_follow_the_nickel_lexer() {
        let cases = [
            ("multiline", r#"m%"import "lit.ncl" %{ import "found.ncl" }"%"#),
            (
                "double-percent multiline",
                r#"m%%"import "lit.ncl" %{ import "lit.ncl" } %%{ import "found.ncl" }"%%"#,
            ),
            ("extra percents before interpolation", r#"m%"import "lit.ncl" %%{ import "found.ncl" }"%"#),
            (
                "extra percents before double-percent interpolation",
                r#"m%%"%%%{ import "found.ncl" } "% import "lit.ncl" "%%"#,
            ),
            ("quote before interpolation", r#"m%" "%{ import "found.ncl" } import "lit.ncl" "%"#),
            ("quote before short interpolation", r#"m%%" "%{ import "lit.ncl" } "%%{ import "found.ncl" } "%%"#),
            (
                "quote and extra percents before interpolation",
                r#"m%%" "%%%{ import "found.ncl" } import "lit.ncl" "%%"#,
            ),
            ("symbolic string", r#"nix-s%%"import "lit.ncl" %{ import "lit.ncl" } %%{ import "found.ncl" }"%%"#),
            ("number before multiline", r#"1e2m%"import "lit.ncl" %{ import "found.ncl" }"%"#),
            ("enum tag before standard string", r#"'foo-s%"import \"lit.ncl\" %{ import "found.ncl" }""#),
            ("modulo before standard string", r#"foom%"import \"lit.ncl\" %{ import "found.ncl" }""#),
            ("standard string", r#""import \"lit.ncl\" \%{ import \"lit.ncl\" } %%{ import "found.ncl" }""#),
        ];

        let mismatches = cases
            .iter()
            .filter_map(|(label, string)| {
                let source = format!("{{ s = {string}, after = import \"after.ncl\" }}");
                let observed = observed_import_closure("main.ncl", &source, &[]);
                (observed != ["after.ncl", "found.ncl"]).then_some((*label, observed))
            })
            .collect::<Vec<_>>();

        assert!(mismatches.is_empty(), "{mismatches:?}");
    }

    #[test]
    fn closure_scans_dependencies_by_effective_import_format() {
        let dependencies = [
            NamedText {
                path: "data.json",
                text: "import \"from-json-as-nickel.ncl\"",
            },
            NamedText {
                path: "quoted.yaml",
                text: "import \"from-yaml-as-quoted-nickel.ncl\"",
            },
            NamedText {
                path: "text.ncl",
                text: "import \"from-ncl-as-text.ncl\"",
            },
            NamedText {
                path: "both.txt",
                text: "import \"from-txt-as-text-then-nickel.ncl\"",
            },
            NamedText {
                path: "plain.toml",
                text: "import \"from-plain-toml.ncl\"",
            },
            NamedText {
                path: "plain.ncl",
                text: "import \"from-plain-ncl.ncl\"",
            },
            NamedText {
                path: "config.nix",
                text: "import \"from-unknown-extension.ncl\"",
            },
        ];
        let source = concat!(
            "[\n",
            "  import \"data.json\" as 'Nickel,\n",
            "  import \"quoted.yaml\" as # comment\n '\"Nickel\",\n",
            "  import \"text.ncl\" as 'Text,\n",
            "  import \"both.txt\" as 'Text,\n",
            "  import \"both.txt\" as 'Nickel,\n",
            "  import \"plain.toml\",\n",
            "  import \"plain.ncl\",\n",
            "  import \"config.nix\",\n",
            "]\n",
        );

        let observed = observed_import_closure("main.ncl", source, &dependencies);

        assert_eq!(observed, vec![
            "both.txt".to_owned(),
            "config.nix".to_owned(),
            "data.json".to_owned(),
            "from-json-as-nickel.ncl".to_owned(),
            "from-plain-ncl.ncl".to_owned(),
            "from-txt-as-text-then-nickel.ncl".to_owned(),
            "from-unknown-extension.ncl".to_owned(),
            "from-yaml-as-quoted-nickel.ncl".to_owned(),
            "plain.ncl".to_owned(),
            "plain.toml".to_owned(),
            "quoted.yaml".to_owned(),
            "text.ncl".to_owned(),
        ]);
    }

    #[test]
    fn closure_resolves_relative_imports_and_follows_supplied_nickel_dependencies() {
        let dependencies = [
            NamedText {
                path: "./standards/contracts.ncl",
                text: "let data = import \"data/values.json\" in import \"../shared/base.ncl\"",
            },
            NamedText {
                path: "shared/base.ncl",
                text: "import \"../config/source.ncl\"",
            },
            NamedText {
                path: "standards/data/values.json",
                text: "{\"key\": import \"ignored-data.ncl\"}",
            },
        ];

        let observed = observed_import_closure(
            "config/source.ncl",
            "import \"../standards/./contracts.ncl\" & import \"/abs.ncl\" & import \"../../escape.ncl\"",
            &dependencies,
        );

        assert_eq!(observed, vec![
            "../escape.ncl".to_owned(),
            "/abs.ncl".to_owned(),
            "shared/base.ncl".to_owned(),
            "standards/contracts.ncl".to_owned(),
            "standards/data/values.json".to_owned(),
        ]);
    }
}
