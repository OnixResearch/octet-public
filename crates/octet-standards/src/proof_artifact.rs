//! Shared pure proof/artifact identity and binding helpers.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use crate::model::ProofRail;
use crate::source::SourceFile;

/// Receipt binding mode that binds command text and matched source contents.
pub const RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE: &str = "blake3-command-and-source";

const FIELD_CLOSURE_GRAPH: &str = "verifier_nix_closure_graph_paths";

/// Deterministic identity for a text artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactTextIdentity {
    /// Normalized path or artifact locator.
    pub path: String,
    /// BLAKE3 digest over exact text bytes.
    pub blake3: String,
    /// Exact text byte length.
    pub bytes: u64,
}

/// Return the lowercase BLAKE3 digest for exact bytes.
#[must_use]
pub fn blake3_hex_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Return the lowercase BLAKE3 digest for exact UTF-8 text bytes.
#[must_use]
pub fn blake3_hex_text(text: &str) -> String {
    blake3_hex_bytes(text.as_bytes())
}

/// Build a deterministic text identity for manifest or receipt binding.
///
/// # Panics
///
/// Panics only if the text length does not fit in `u64`.
#[must_use]
pub fn artifact_text_identity(path: &str, text: &str) -> ArtifactTextIdentity {
    ArtifactTextIdentity {
        path: path.to_owned(),
        blake3: blake3_hex_text(text),
        bytes: u64::try_from(text.len()).expect("artifact text length should fit u64"),
    }
}

/// Return normalized, sorted matches for exact/glob rail path patterns.
#[must_use]
pub fn matching_sources<'a>(patterns: &[String], sources: &'a [SourceFile]) -> Vec<&'a SourceFile> {
    let mut matched_paths = BTreeSet::new();
    let mut matched_sources = Vec::new();
    for pattern in patterns {
        for source in sources.iter().filter(|source| path_matches_pattern(pattern, &source.path)) {
            let normalized = normalize_path(&source.path);
            if matched_paths.insert(normalized) {
                matched_sources.push(source);
            }
        }
    }
    matched_sources.sort_by_key(|source| normalize_path(&source.path));
    matched_sources
}

/// Return deterministic diagnostics for exact/glob path patterns.
#[must_use]
pub fn path_pattern_diagnostics(patterns: &[String], sources: &[SourceFile], field_name: &str) -> Vec<String> {
    if patterns.is_empty() {
        return vec![format!("proof rail field `{field_name}` is empty")];
    }

    let mut diagnostics = Vec::new();
    for pattern in patterns {
        let normalized_pattern = normalize_path(pattern);
        if normalized_pattern.is_empty() {
            diagnostics.push(format!("proof rail field `{field_name}` contains an empty path pattern"));
            continue;
        }
        let matched_paths = sources
            .iter()
            .filter(|source| path_matches_normalized_pattern(&normalized_pattern, &source.path))
            .map(|source| normalize_path(&source.path))
            .collect::<Vec<_>>();
        if matched_paths.is_empty() {
            diagnostics.push(format!(
                "proof rail field `{field_name}` pattern `{normalized_pattern}` matches no checked path"
            ));
        }
        if !is_glob_pattern(&normalized_pattern) && matched_paths.len() > 1 {
            diagnostics.push(format!(
                "proof rail field `{field_name}` exact pattern `{normalized_pattern}` matched multiple paths"
            ));
        }
    }
    diagnostics
}

/// Build BLAKE3 identities for matched sources, or deterministic diagnostics for bad patterns.
///
/// # Errors
///
/// Returns diagnostics when a pattern is empty, absolute, parent-traversing,
/// unmatched, or otherwise ambiguous for the checked source set.
#[must_use = "source identity diagnostics must be handled before accepting proof artifacts"]
pub fn source_identities_for_patterns(
    patterns: &[String],
    sources: &[SourceFile],
    field_name: &str,
) -> Result<Vec<ArtifactTextIdentity>, Vec<String>> {
    let diagnostics = path_pattern_diagnostics(patterns, sources, field_name);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let identities = matching_sources(patterns, sources)
        .into_iter()
        .map(|source| artifact_text_identity(&normalize_path(&source.path), &source.text))
        .collect::<Vec<_>>();
    Ok(identities)
}

/// Return proof receipt path patterns, falling back to proof paths for legacy rails.
#[must_use]
pub fn receipt_paths(rail: &ProofRail) -> Vec<String> {
    if rail.receipt_paths.is_empty() {
        rail.proof_paths.clone()
    } else {
        rail.receipt_paths.clone()
    }
}

/// Return true when a proof rail is relevant to the checked source set.
#[must_use]
pub fn proof_rail_is_relevant(rail: &ProofRail, sources: &[SourceFile]) -> bool {
    patterns_match_any_source(&rail.source_paths, sources) || patterns_match_any_source(&receipt_paths(rail), sources)
}

/// Build deterministic legacy receipt binding lines.
#[must_use]
pub fn receipt_binding_lines(rail: &ProofRail, sources: &[SourceFile]) -> Vec<String> {
    if rail.receipt_binding != RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE {
        return Vec::new();
    }
    let mut lines = vec![format!("proof-command-blake3: {}", blake3_hex_text(&rail.command))];
    let mut matched_sources = matching_sources(&rail.source_paths, sources);
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    lines.extend(
        matched_sources
            .into_iter()
            .map(|source| format!("proof-source-blake3: {}:{}", source.path, blake3_hex_text(&source.text))),
    );
    lines
}

/// Build deterministic legacy receipt binding text.
#[must_use]
pub fn receipt_binding_text(rail: &ProofRail, sources: &[SourceFile]) -> String {
    let lines = receipt_binding_lines(rail, sources);
    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    }
}

/// Build a BLAKE3 digest over matched closure manifest sources.
#[must_use]
pub fn build_closure_digest(paths: &[String], sources: &[SourceFile]) -> Option<String> {
    if paths.is_empty() {
        return None;
    }
    let mut matched_sources = matching_sources(paths, sources);
    if matched_sources.is_empty() {
        return None;
    }
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    let mut manifest = String::new();
    for source in matched_sources {
        writeln!(manifest, "{}:{}", source.path, blake3_hex_text(&source.text))
            .expect("writing to string should not fail");
    }
    Some(blake3_hex_text(&manifest))
}

/// Build a BLAKE3 digest over canonical Nix exportReferencesGraph artifacts.
///
/// # Errors
///
/// Returns a deterministic diagnostic when any matched graph is malformed.
pub fn nix_closure_graph_digest(paths: &[String], sources: &[SourceFile]) -> Result<Option<String>, String> {
    if paths.is_empty() {
        return Ok(None);
    }
    let mut matched_sources = matching_sources(paths, sources);
    if matched_sources.is_empty() {
        return Ok(None);
    }
    matched_sources.sort_by(|left, right| left.path.cmp(&right.path));
    let mut canonical_graphs = String::new();
    for source in matched_sources {
        let canonical = canonical_nix_closure_graph(&source.text)
            .map_err(|diagnostic| format!("{FIELD_CLOSURE_GRAPH} `{}`: {diagnostic}", source.path))?;
        canonical_graphs.push_str(&canonical);
    }
    Ok(Some(blake3_hex_text(&canonical_graphs)))
}

/// Canonicalize a Nix exportReferencesGraph text artifact.
///
/// # Errors
///
/// Returns a deterministic diagnostic naming the malformed graph field.
pub fn canonical_nix_closure_graph(text: &str) -> Result<String, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut index = 0;
    let mut records = Vec::new();
    while index < lines.len() {
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= lines.len() {
            break;
        }
        let path = lines[index].trim().to_owned();
        if path.is_empty() {
            return Err("empty closure graph path".to_owned());
        }
        index += 1;
        while index < lines.len() && lines[index].trim().is_empty() {
            index += 1;
        }
        if index >= lines.len() {
            return Err(format!("closure graph path `{path}` is missing reference count"));
        }
        let reference_count = lines[index].trim().parse::<usize>().map_err(|_| {
            format!("closure graph path `{path}` has malformed reference count `{}`", lines[index].trim())
        })?;
        index += 1;
        let mut references = Vec::with_capacity(reference_count);
        for reference_index in 0..reference_count {
            while index < lines.len() && lines[index].trim().is_empty() {
                index += 1;
            }
            if index >= lines.len() {
                return Err(format!(
                    "closure graph path `{path}` expected {reference_count} references but found {reference_index}"
                ));
            }
            references.push(lines[index].trim().to_owned());
            index += 1;
        }
        references.sort();
        records.push((path, references));
    }
    if records.is_empty() {
        return Err("closure graph has no records".to_owned());
    }
    records.sort_by(|left, right| left.0.cmp(&right.0));
    let mut canonical = String::new();
    for (path, references) in records {
        writeln!(canonical, "{path}").expect("writing to string should not fail");
        writeln!(canonical, "{}", references.len()).expect("writing to string should not fail");
        for reference in references {
            writeln!(canonical, "{reference}").expect("writing to string should not fail");
        }
    }
    Ok(canonical)
}

/// Normalize a path for proof matching.
#[must_use]
pub fn normalize_path(path: &str) -> String {
    let replaced = path.replace('\\', "/");
    let mut parts = Vec::new();
    for part in replaced.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

/// Return true when a normalized path or glob pattern matches a path.
#[must_use]
pub fn path_matches_pattern(pattern: &str, path: &str) -> bool {
    let normalized_pattern = normalize_path(pattern);
    path_matches_normalized_pattern(&normalized_pattern, path)
}

fn patterns_match_any_source(patterns: &[String], sources: &[SourceFile]) -> bool {
    !patterns.is_empty() && !matching_sources(patterns, sources).is_empty()
}

fn path_matches_normalized_pattern(normalized_pattern: &str, path: &str) -> bool {
    if normalized_pattern.is_empty() {
        return false;
    }
    let normalized_path = normalize_path(path);
    if is_glob_pattern(normalized_pattern) {
        glob_matches(normalized_pattern, &normalized_path)
    } else {
        normalized_pattern == normalized_path
    }
}

fn is_glob_pattern(pattern: &str) -> bool {
    pattern.contains('*') || pattern.contains('?')
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let pattern_chars = pattern.chars().collect::<Vec<_>>();
    let candidate_chars = candidate.chars().collect::<Vec<_>>();
    let mut pattern_index = 0;
    let mut candidate_index = 0;
    let mut star_index = None;
    let mut retry_candidate_index = 0;

    while candidate_index < candidate_chars.len() {
        if pattern_index < pattern_chars.len()
            && (pattern_chars[pattern_index] == '?' || pattern_chars[pattern_index] == candidate_chars[candidate_index])
        {
            pattern_index += 1;
            candidate_index += 1;
        } else if pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == '*' {
            star_index = Some(pattern_index);
            pattern_index += 1;
            retry_candidate_index = candidate_index;
        } else if let Some(star) = star_index {
            pattern_index = star + 1;
            retry_candidate_index += 1;
            candidate_index = retry_candidate_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern_chars.len() && pattern_chars[pattern_index] == '*' {
        pattern_index += 1;
    }
    pattern_index == pattern_chars.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_PATH: &str = "src/verified/add.rs";
    const PROOF_PATH: &str = "proofs/add.receipt";
    const SOURCE_TEXT: &str = "fn add(left: u64, right: u64) -> u64 { left + right }\n";
    const PROOF_TEXT: &str = "proof-result: pass\n";

    #[test]
    fn same_content_has_same_identity_for_text_and_bytes() {
        let identity = artifact_text_identity(SOURCE_PATH, SOURCE_TEXT);
        let expected_source_bytes = u64::try_from(SOURCE_TEXT.len()).expect("fixture length fits u64");

        assert_eq!(identity.path, SOURCE_PATH);
        assert_eq!(identity.bytes, expected_source_bytes);
        assert_eq!(identity.blake3, blake3_hex_text(SOURCE_TEXT));
        assert_eq!(identity.blake3, blake3_hex_bytes(SOURCE_TEXT.as_bytes()));
    }

    #[test]
    fn relevant_rail_matches_source_or_receipt_paths() {
        let sources = vec![SourceFile::new(PROOF_PATH, PROOF_TEXT)];
        let rail = ProofRail {
            source_paths: vec![SOURCE_PATH.to_owned()],
            proof_paths: vec![PROOF_PATH.to_owned()],
            ..ProofRail::default()
        };

        assert!(proof_rail_is_relevant(&rail, &sources));
    }

    #[test]
    fn receipt_binding_lines_are_stable_and_sorted() {
        let sources = vec![
            SourceFile::new("src/verified/b.rs", "b\n"),
            SourceFile::new("src/verified/a.rs", "a\n"),
        ];
        let rail = ProofRail {
            command: "verus src/verified/*.rs".to_owned(),
            receipt_binding: RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE.to_owned(),
            source_paths: vec!["src/verified/*.rs".to_owned()],
            ..ProofRail::default()
        };

        let lines = receipt_binding_lines(&rail, &sources);

        assert_eq!(lines[0], format!("proof-command-blake3: {}", blake3_hex_text(&rail.command)));
        assert_eq!(lines[1], format!("proof-source-blake3: src/verified/a.rs:{}", blake3_hex_text("a\n")));
        assert_eq!(lines[2], format!("proof-source-blake3: src/verified/b.rs:{}", blake3_hex_text("b\n")));
    }

    #[test]
    fn canonical_closure_graph_sorts_records_and_references() {
        let graph = "/nix/store/b\n2\n/nix/store/d\n/nix/store/c\n/nix/store/a\n0\n";

        let canonical = canonical_nix_closure_graph(graph).expect("canonical graph");

        assert_eq!(canonical, "/nix/store/a\n0\n/nix/store/b\n2\n/nix/store/c\n/nix/store/d\n");
    }

    #[test]
    fn malformed_closure_graph_fails_deterministically() {
        let error = canonical_nix_closure_graph("/nix/store/a\nnot-a-count\n").expect_err("bad graph");

        assert!(error.contains("malformed reference count"));
    }

    #[test]
    fn path_diagnostics_reject_missing_and_ambiguous_exact_patterns() {
        let sources = vec![SourceFile::new("src/lib.rs", ""), SourceFile::new("./src/lib.rs", "")];

        let diagnostics =
            path_pattern_diagnostics(&["src/lib.rs".to_owned(), "missing.rs".to_owned()], &sources, "source_paths");

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("matched multiple paths")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("matches no checked path")));
    }
}
