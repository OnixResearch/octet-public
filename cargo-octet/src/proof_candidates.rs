//! Read-only proof-candidate triage over Tiger Style allowances and waivers.
//!
//! Every suppression of a Tiger Style lint is an unproven claim. This module lists the allowances
//! in the checked source that have no validated `proof-ref`, maps only the rails that already
//! exist, and ranks the result by lint severity and trust-boundary membership.
//!
//! The command is a triage aid. It does not run a verifier, does not emit `formal_proof`
//! provenance, and does not claim that a mapped rail proves the suppressed property.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use octet_lint_metadata::GROUP_NAMES;
use octet_lint_metadata::lint_names_for_group;
use octet_lint_metadata::metadata_for_lint;
use serde::Serialize;

use crate::cli::NegativeCheckArgs;
use crate::cli::OutputFormat;
use crate::cli::ProveArgs;
use crate::proof_ref;

const SCHEMA_VERSION: &str = "octet-proof-candidates/v1";
const NEGATIVE_CHECK_SCHEMA_VERSION: &str = proof_ref::NEGATIVE_CHECK_SCHEMA_VERSION_V2;
const NEGATIVE_CHECK_DECK_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const DEFAULT_PROOF_REF: &str = "proof-ref.json";
const TRUST_BOUNDARY_FENCE: &str = "trust-boundary-prefixes";
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const SKIPPED_DIRECTORIES: &[&str] = &[".git", ".jj", ".cairn", "node_modules", "openspec", "target", "vendor"];
const NON_CLAIM: &str =
    "candidate ranking is triage evidence only; a mapped rail proves only its declared scoped property";
const NEGATIVE_CHECK_NON_CLAIM: &str = "mutant rejection demonstrates discrimination for this mutation only; it does not prove the theorem matches the intended property in general";
const STATUS_MISSING: &str = "missing";
const STATUS_UNVALIDATED: &str = "unvalidated";
const STATUS_INVALID: &str = "invalid";
const STATUS_UNBOUND: &str = "unbound";
const STATUS_VALIDATED: &str = "validated";
const NEGATIVE_STATUS_DEMONSTRATED: &str = proof_ref::NEGATIVE_STATUS_DEMONSTRATED;
#[cfg(test)]
const NEGATIVE_STATUS_NOT_DEMONSTRATED: &str = proof_ref::NEGATIVE_STATUS_NOT_DEMONSTRATED;
const NEGATIVE_STATUS_INCONCLUSIVE: &str = proof_ref::NEGATIVE_STATUS_INCONCLUSIVE;
#[cfg(test)]
const CARD_STATUS_PASSED: &str = "passed";
#[cfg(test)]
const CARD_STATUS_FAILED: &str = "failed";

/// One existing rail mapped to a Tiger Style lint.
struct CandidateRail {
    lint: &'static str,
    proof_or_property_id: &'static str,
    policy_id: &'static str,
    rail_index: usize,
    scope_note: &'static str,
}

/// Only rails that already exist in `standards/proof-rails/manifest.ncl` are listed. The README
/// narrowing example binds `unchecked_narrowing` to the addition-bound production card; that rail
/// proves the declared addition-bound property, not the narrowing site.
const CANDIDATE_RAILS: &[CandidateRail] = &[CandidateRail {
    lint: "unchecked_narrowing",
    proof_or_property_id: "octet.verus_proof_rails.addition-bound-production#rail-0",
    policy_id: "octet.verus_proof_rails.addition-bound-production",
    rail_index: 0,
    scope_note: "existing production rail proves only the declared addition-bound property; the negative check must run before claiming the narrowing property",
}];

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub(crate) struct ProofCandidateReport {
    pub(crate) schema_version: String,
    pub(crate) candidates: Vec<ProofCandidate>,
    pub(crate) unsupported_lints: Vec<UnsupportedLint>,
    pub(crate) discharged: usize,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub(crate) struct ProofCandidate {
    pub(crate) rank: usize,
    pub(crate) rank_score: u32,
    pub(crate) lint: String,
    pub(crate) severity: String,
    pub(crate) group: String,
    pub(crate) trust_boundary: bool,
    pub(crate) path: String,
    pub(crate) line: u32,
    pub(crate) column: u32,
    pub(crate) attribute: String,
    pub(crate) context: String,
    pub(crate) proof_ref: ProofRefStatus,
    pub(crate) rail: Option<CandidateRailMapping>,
    pub(crate) unsupported: bool,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub(crate) struct ProofRefStatus {
    pub(crate) status: String,
    pub(crate) reference_id: Option<String>,
    pub(crate) proof_or_property_id: Option<String>,
    pub(crate) path: Option<String>,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub(crate) struct CandidateRailMapping {
    pub(crate) proof_or_property_id: String,
    pub(crate) policy_id: String,
    pub(crate) rail_index: usize,
    pub(crate) scope_note: String,
}

#[derive(Clone, Debug, Serialize, Eq, PartialEq)]
pub(crate) struct UnsupportedLint {
    pub(crate) lint: String,
    pub(crate) severity: String,
    pub(crate) group: String,
    pub(crate) candidates: usize,
}

#[derive(Debug, Serialize)]
struct NegativeCheckReport {
    schema_version: String,
    proof_or_property_id: String,
    status: String,
    mutant_source: String,
    mutant_cards: String,
    deck: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    verifier_evidence: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    measurement: Option<proof_ref::NegativeControlMeasurement>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

struct SourceInput {
    display: String,
    canonical: PathBuf,
    text: String,
}

struct LoadedReference {
    path: String,
    reference: proof_ref::ProofEvidenceRef,
}

struct ScannedAllowance {
    lint: String,
    attribute: String,
    spelling: String,
    line: u32,
    column: u32,
}

pub(crate) fn run(args: &ProveArgs) -> Result<i32, String> {
    let report = build_report(args)?;
    let blocking = report.is_blocking();
    print_candidate_report(args.output_format, &report)?;
    Ok(i32::from(blocking))
}

pub(crate) fn build_report(args: &ProveArgs) -> Result<ProofCandidateReport, String> {
    let mut diagnostics = Vec::new();
    let source_paths = if args.source.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.source.clone()
    };
    let sources = collect_sources(&source_paths, &mut diagnostics)?;
    let trust_prefixes = read_trust_boundary_prefixes(args.trust_boundary.as_deref(), &mut diagnostics);
    let deck = match &args.proof_cards {
        Some(path) => Some(proof_ref::read_deck_file(path)?),
        None => None,
    };
    let reference_paths = discover_reference_paths(&args.proof_refs);
    let references = load_references(&reference_paths, &mut diagnostics);

    let mut candidates = Vec::new();
    let mut discharged = 0;
    for source in &sources {
        for allowance in scan_allowances(&source.text) {
            let proof_ref_status =
                candidate_reference_status(&allowance, source, &references, deck.as_ref(), &mut diagnostics);
            if proof_ref_status.status == STATUS_VALIDATED {
                discharged += 1;
                continue;
            }
            let metadata = metadata_for_lint(&allowance.lint);
            let rail = rail_mapping(&allowance.lint);
            let severity = metadata.map_or("unknown", |meta| meta.default_level).to_owned();
            let group = metadata.map_or("tigerstyle::unknown", |meta| meta.domain_group).to_owned();
            let trust_boundary = on_trust_boundary(&source.display, &trust_prefixes);
            candidates.push(ProofCandidate {
                rank: 0,
                rank_score: rank_score(&severity, trust_boundary),
                lint: allowance.lint.clone(),
                severity,
                group,
                trust_boundary,
                path: source.display.clone(),
                line: allowance.line,
                column: allowance.column,
                attribute: allowance.attribute.clone(),
                context: allowance.context(),
                proof_ref: proof_ref_status,
                unsupported: rail.is_none(),
                rail: rail.map(|rail| CandidateRailMapping {
                    proof_or_property_id: rail.proof_or_property_id.to_owned(),
                    policy_id: rail.policy_id.to_owned(),
                    rail_index: rail.rail_index,
                    scope_note: rail.scope_note.to_owned(),
                }),
            });
        }
    }
    candidates.sort_by(|left, right| {
        right
            .rank_score
            .cmp(&left.rank_score)
            .then_with(|| left.lint.cmp(&right.lint))
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.attribute.cmp(&right.attribute))
    });
    for (index, candidate) in candidates.iter_mut().enumerate() {
        candidate.rank = index.saturating_add(1);
    }
    let unsupported_lints = summarize_unsupported(&candidates);
    diagnostics.sort();
    diagnostics.dedup();
    Ok(ProofCandidateReport {
        schema_version: SCHEMA_VERSION.to_owned(),
        candidates,
        unsupported_lints,
        discharged,
        diagnostics,
        non_claims: vec![NON_CLAIM.to_owned()],
    })
}

impl ProofCandidateReport {
    pub(crate) fn is_blocking(&self) -> bool {
        !self.diagnostics.is_empty()
            || self.candidates.iter().any(|candidate| {
                matches!(candidate.proof_ref.status.as_str(), STATUS_INVALID | STATUS_UNVALIDATED | STATUS_UNBOUND)
            })
    }
}

impl ScannedAllowance {
    fn context(&self) -> String {
        format!("#[{}({})]", self.attribute, self.spelling)
    }
}

fn rank_score(severity: &str, trust_boundary: bool) -> u32 {
    let severity_weight: u32 = match severity {
        "deny" | "error" => 3,
        "warn" => 2,
        "allow" => 1,
        _ => 0,
    };
    severity_weight.saturating_mul(2).saturating_add(u32::from(trust_boundary))
}

fn rail_mapping(lint: &str) -> Option<&'static CandidateRail> {
    CANDIDATE_RAILS.iter().find(|rail| rail.lint == lint)
}

fn summarize_unsupported(candidates: &[ProofCandidate]) -> Vec<UnsupportedLint> {
    let mut summary: BTreeMap<&str, (&str, &str, usize)> = BTreeMap::new();
    for candidate in candidates.iter().filter(|candidate| candidate.unsupported) {
        let entry = summary.entry(&candidate.lint).or_insert((&candidate.severity, &candidate.group, 0));
        entry.2 = entry.2.saturating_add(1);
    }
    summary
        .into_iter()
        .map(|(lint, (severity, group, candidates))| UnsupportedLint {
            lint: lint.to_owned(),
            severity: severity.to_owned(),
            group: group.to_owned(),
            candidates,
        })
        .collect()
}

fn candidate_reference_status(
    allowance: &ScannedAllowance,
    source: &SourceInput,
    references: &[LoadedReference],
    deck: Option<&proof_ref::ProofCardDeck>,
    diagnostics: &mut Vec<String>,
) -> ProofRefStatus {
    let mut matches = references
        .iter()
        .filter(|reference| reference_matches_source(&reference.reference, source))
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| left.reference.reference_id.cmp(&right.reference.reference_id));
    if matches.is_empty() {
        return ProofRefStatus {
            status: STATUS_MISSING.to_owned(),
            reference_id: None,
            proof_or_property_id: None,
            path: None,
            diagnostics: Vec::new(),
        };
    }

    let mut best: Option<ProofRefStatus> = None;
    for reference in matches {
        let status = evaluate_reference(allowance, source, reference, deck, diagnostics);
        let replace = best
            .as_ref()
            .is_none_or(|current| status_priority(&status.status) < status_priority(&current.status));
        if replace {
            best = Some(status);
        }
    }
    best.unwrap_or(ProofRefStatus {
        status: STATUS_MISSING.to_owned(),
        reference_id: None,
        proof_or_property_id: None,
        path: None,
        diagnostics: Vec::new(),
    })
}

fn evaluate_reference(
    allowance: &ScannedAllowance,
    source: &SourceInput,
    loaded: &LoadedReference,
    deck: Option<&proof_ref::ProofCardDeck>,
    diagnostics: &mut Vec<String>,
) -> ProofRefStatus {
    let reference = &loaded.reference;
    let mut status = ProofRefStatus {
        status: STATUS_MISSING.to_owned(),
        reference_id: Some(reference.reference_id.clone()),
        proof_or_property_id: Some(reference.proof_or_property_id.clone()),
        path: Some(loaded.path.clone()),
        diagnostics: Vec::new(),
    };
    if !consumer_context_names_site(
        &reference.consumer_context,
        &reference.consumer_source_path,
        &source.display,
        allowance.line,
    ) {
        STATUS_UNBOUND.clone_into(&mut status.status);
        status.diagnostics.push(format!(
            "proof-ref `{}` consumer_context does not name `{}:{}`",
            reference.reference_id, source.display, allowance.line
        ));
        return status;
    }
    let Some(deck) = deck else {
        STATUS_UNVALIDATED.clone_into(&mut status.status);
        status
            .diagnostics
            .push(format!("proof-ref `{}` has no --proof-cards deck to validate against", loaded.path));
        return status;
    };
    let validation =
        proof_ref::validate_reference_against_source(reference, deck, &reference.consumer_source_path, &source.text);
    if validation.is_empty() {
        STATUS_VALIDATED.clone_into(&mut status.status);
    } else {
        STATUS_INVALID.clone_into(&mut status.status);
        status.diagnostics = validation;
    }
    if !status.diagnostics.is_empty() {
        diagnostics
            .extend(status.diagnostics.iter().map(|diagnostic| format!("proof-ref `{}`: {diagnostic}", loaded.path)));
    }
    status
}

fn consumer_context_names_site(context: &str, reference_path: &str, display_path: &str, line: u32) -> bool {
    let reference_path = normalize_path(reference_path);
    let display_path = normalize_path(display_path);
    [reference_path, display_path].iter().any(|path| context.contains(&format!("at {path}:{line}")))
}

fn status_priority(status: &str) -> u8 {
    match status {
        STATUS_VALIDATED => 0,
        STATUS_UNVALIDATED => 1,
        STATUS_INVALID => 2,
        STATUS_UNBOUND => 3,
        _ => 4,
    }
}

fn reference_matches_source(reference: &proof_ref::ProofEvidenceRef, source: &SourceInput) -> bool {
    let reference_path = Path::new(&reference.consumer_source_path);
    match fs::canonicalize(reference_path) {
        Ok(canonical) => canonical == source.canonical,
        Err(_) => normalize_path(&reference.consumer_source_path) == normalize_path(&source.display),
    }
}

fn discover_reference_paths(explicit: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = explicit.to_vec();
    if paths.is_empty() {
        let default = PathBuf::from(DEFAULT_PROOF_REF);
        if default.is_file() {
            paths.push(default);
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

fn load_references(paths: &[PathBuf], diagnostics: &mut Vec<String>) -> Vec<LoadedReference> {
    let mut loaded = Vec::new();
    for path in paths {
        match proof_ref::read_reference_file(path) {
            Ok(reference) => loaded.push(LoadedReference {
                path: display_path(path),
                reference,
            }),
            Err(error) => diagnostics.push(format!("cannot load proof-ref `{}`: {error}", path.display())),
        }
    }
    loaded
}

fn collect_sources(paths: &[PathBuf], diagnostics: &mut Vec<String>) -> Result<Vec<SourceInput>, String> {
    let mut files = Vec::new();
    for path in paths {
        collect_rust_files(path, &mut files)?;
    }
    files.sort();
    files.dedup();
    let mut sources = Vec::new();
    for file in files {
        let metadata = fs::metadata(&file).map_err(|error| format!("read source {}: {error}", file.display()))?;
        if metadata.len() > MAX_SOURCE_BYTES {
            diagnostics.push(format!("skipped source `{}` larger than {MAX_SOURCE_BYTES} bytes", file.display()));
            continue;
        }
        let text = fs::read_to_string(&file).map_err(|error| format!("read source {}: {error}", file.display()))?;
        let canonical =
            fs::canonicalize(&file).map_err(|error| format!("resolve source {}: {error}", file.display()))?;
        sources.push(SourceInput {
            display: display_path(&file),
            canonical,
            text,
        });
    }
    Ok(sources)
}

fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("read source path {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }
    if metadata.is_file() {
        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("read source directory {}: {error}", path.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read source directory {}: {error}", path.display()))?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let file_type = entry.file_type().map_err(|error| format!("inspect {}: {error}", entry.path().display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let name = entry.file_name();
            if SKIPPED_DIRECTORIES.iter().any(|skipped| name.to_string_lossy() == *skipped) {
                continue;
            }
            collect_rust_files(&entry.path(), files)?;
        } else if file_type.is_file() && entry.path().extension().is_some_and(|extension| extension == "rs") {
            files.push(entry.path());
        }
    }
    Ok(())
}

fn display_path(path: &Path) -> String {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if let Ok(cwd) = std::env::current_dir().and_then(fs::canonicalize)
        && let Ok(relative) = canonical.strip_prefix(&cwd)
    {
        let display = normalize_path(&relative.display().to_string());
        return if display.is_empty() { ".".to_owned() } else { display };
    }
    normalize_path(&canonical.display().to_string())
}

fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized.strip_prefix("./").unwrap_or(&normalized).to_owned()
}

fn read_trust_boundary_prefixes(path: Option<&Path>, diagnostics: &mut Vec<String>) -> Vec<String> {
    let path = path.unwrap_or_else(|| Path::new(DEFAULT_TRUST_BOUNDARY_PATH));
    match fs::read_to_string(path) {
        Ok(text) => parse_trust_boundary_prefixes(&text),
        Err(error) => {
            diagnostics.push(format!("cannot read trust-boundary document `{}`: {error}", path.display()));
            Vec::new()
        }
    }
}

const DEFAULT_TRUST_BOUNDARY_PATH: &str = "docs/trust-boundary-evidence.md";

fn parse_trust_boundary_prefixes(text: &str) -> Vec<String> {
    let mut prefixes = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if !in_block {
            if let Some(info) = trimmed.strip_prefix("```")
                && info.split_whitespace().any(|part| part == TRUST_BOUNDARY_FENCE)
            {
                in_block = true;
            }
        } else if trimmed == "```" {
            break;
        } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
            prefixes.push(trimmed.to_owned());
        }
    }
    prefixes.sort();
    prefixes.dedup();
    prefixes
}

fn on_trust_boundary(path: &str, prefixes: &[String]) -> bool {
    let path = normalize_path(path);
    prefixes.iter().any(|prefix| {
        let prefix = normalize_path(prefix);
        path == prefix.trim_end_matches('/') || path.starts_with(&prefix)
    })
}

fn scan_allowances(text: &str) -> Vec<ScannedAllowance> {
    let mut allowances = Vec::new();
    let bytes = text.as_bytes();
    let mut index = 0usize;
    let mut line = 1u32;
    let mut column = 1u32;
    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                line = line.saturating_add(1);
                column = 1;
                index = index.saturating_add(1);
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                let (next, next_line, next_column) = skip_line_comment(bytes, index, line, column);
                index = next;
                line = next_line;
                column = next_column;
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let (next, next_line, next_column) = skip_block_comment(bytes, index, line, column);
                index = next;
                line = next_line;
                column = next_column;
            }
            b'"' => {
                let (next, next_line, next_column) = skip_string(bytes, index, line, column);
                index = next;
                line = next_line;
                column = next_column;
            }
            b'\'' => {
                let (next, next_line, next_column) = skip_char_literal(bytes, index, line, column);
                index = next;
                line = next_line;
                column = next_column;
            }
            b'#' => {
                if let Some((end, inner)) = attribute_bounds(text, index) {
                    scan_attribute(&text[index..end], inner, line, column, &mut allowances);
                    let (next, next_line, next_column) = advance_position(bytes, index, end, line, column);
                    index = next;
                    line = next_line;
                    column = next_column;
                } else {
                    index = index.saturating_add(1);
                    column = column.saturating_add(1);
                }
            }
            _ => {
                index = index.saturating_add(1);
                column = column.saturating_add(1);
            }
        }
    }
    allowances.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then_with(|| left.column.cmp(&right.column))
            .then_with(|| left.lint.cmp(&right.lint))
            .then_with(|| left.attribute.cmp(&right.attribute))
    });
    allowances.dedup_by(|left, right| {
        left.line == right.line
            && left.column == right.column
            && left.lint == right.lint
            && left.attribute == right.attribute
    });
    allowances
}

fn attribute_bounds(text: &str, start: usize) -> Option<(usize, bool)> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&b'#') {
        return None;
    }
    let (open, inner) = match (bytes.get(start + 1), bytes.get(start + 2)) {
        (Some(b'!'), Some(b'[')) => (start + 2, true),
        (Some(b'['), _) => (start + 1, false),
        _ => return None,
    };
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'[' => depth = depth.saturating_add(1),
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((index.saturating_add(1), inner));
                }
            }
            b'"' => {
                index = skip_string(bytes, index, 1, 1).0;
                continue;
            }
            _ => {}
        }
        index = index.saturating_add(1);
    }
    None
}

fn scan_attribute(attribute: &str, _inner: bool, line: u32, column: u32, allowances: &mut Vec<ScannedAllowance>) {
    let sanitized = strip_attribute_comments(attribute);
    for attribute_name in ["allow", "expect"] {
        let mut search = 0usize;
        while let Some(position) = find_identifier(&sanitized, attribute_name, search) {
            let mut cursor = position.saturating_add(attribute_name.len());
            cursor = skip_whitespace(&sanitized, cursor);
            if sanitized.as_bytes().get(cursor) != Some(&b'(') {
                search = position.saturating_add(attribute_name.len());
                continue;
            }
            let Some((end, body)) = parenthesized_body(&sanitized, cursor) else {
                break;
            };
            for item in split_top_level(&body) {
                let item = item.trim();
                if item.is_empty() || item.starts_with('#') {
                    continue;
                }
                let Some(spelling) = lint_spelling(item) else {
                    continue;
                };
                for lint in expand_lint(&spelling) {
                    allowances.push(ScannedAllowance {
                        lint,
                        attribute: attribute_name.to_owned(),
                        spelling: spelling.clone(),
                        line,
                        column,
                    });
                }
            }
            search = end;
        }
    }
}

fn expand_lint(spelling: &str) -> Vec<String> {
    let bare = spelling.strip_prefix("tigerstyle::").unwrap_or(spelling);
    if let Some(meta) = metadata_for_lint(bare) {
        return vec![meta.lint_name.to_owned()];
    }
    let group = if GROUP_NAMES.contains(&spelling) {
        Some(spelling)
    } else {
        GROUP_NAMES.iter().find(|group| group.strip_prefix("tigerstyle::") == Some(bare)).copied()
    };
    if let Some(group) = group {
        return lint_names_for_group(group).into_iter().map(str::to_owned).collect();
    }
    if spelling.starts_with("tigerstyle::") {
        return vec![bare.to_owned()];
    }
    Vec::new()
}

fn lint_spelling(item: &str) -> Option<String> {
    let item = item.trim();
    if item.contains('=') || item.starts_with("cfg(") || item.starts_with("reason") {
        return None;
    }
    let token = item.split_whitespace().next()?;
    if token.is_empty()
        || token.starts_with('"')
        || token.starts_with("clippy::")
        || token.starts_with("rustdoc::")
        || token.starts_with("rustc::")
    {
        return None;
    }
    Some(token.trim_end_matches(',').to_owned())
}

fn strip_attribute_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = Vec::with_capacity(text.len());
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index = index.saturating_add(1);
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                let (next, _, _) = skip_block_comment(bytes, index, 1, 1);
                index = next;
            }
            byte => {
                output.push(byte);
                index = index.saturating_add(1);
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn find_identifier(text: &str, identifier: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = start;
    while index + identifier.len() <= bytes.len() {
        if bytes[index..].starts_with(identifier.as_bytes()) {
            let before_ok = index == 0 || !is_identifier_byte(bytes[index - 1]);
            let after = index + identifier.len();
            let after_ok = after >= bytes.len() || !is_identifier_byte(bytes[after]);
            if before_ok && after_ok {
                return Some(index);
            }
        }
        index = index.saturating_add(1);
    }
    None
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn parenthesized_body(text: &str, open: usize) -> Option<(usize, String)> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = open;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => depth = depth.saturating_add(1),
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((index.saturating_add(1), text[open + 1..index].to_owned()));
                }
            }
            b'"' => {
                index = skip_string(bytes, index, 1, 1).0;
                continue;
            }
            _ => {}
        }
        index = index.saturating_add(1);
    }
    None
}

fn split_top_level(text: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' | b'[' | b'{' => depth = depth.saturating_add(1),
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                items.push(text[start..index].to_owned());
                start = index.saturating_add(1);
            }
            b'"' => {
                index = skip_string(bytes, index, 1, 1).0;
                continue;
            }
            _ => {}
        }
        index = index.saturating_add(1);
    }
    if start < text.len() {
        items.push(text[start..].to_owned());
    }
    items
}

fn skip_whitespace(text: &str, mut index: usize) -> usize {
    let bytes = text.as_bytes();
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index = index.saturating_add(1);
    }
    index
}

fn skip_line_comment(bytes: &[u8], mut index: usize, line: u32, column: u32) -> (usize, u32, u32) {
    let mut column = column.saturating_add(2);
    index = index.saturating_add(2);
    while index < bytes.len() && bytes[index] != b'\n' {
        index = index.saturating_add(1);
        column = column.saturating_add(1);
    }
    (index, line, column)
}

fn skip_block_comment(bytes: &[u8], mut index: usize, mut line: u32, mut column: u32) -> (usize, u32, u32) {
    index = index.saturating_add(2);
    column = column.saturating_add(2);
    while index < bytes.len() {
        if bytes[index] == b'\n' {
            line = line.saturating_add(1);
            column = 1;
            index = index.saturating_add(1);
        } else if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
            return (index.saturating_add(2), line, column.saturating_add(2));
        } else {
            index = index.saturating_add(1);
            column = column.saturating_add(1);
        }
    }
    (index, line, column)
}

fn skip_string(bytes: &[u8], mut index: usize, mut line: u32, mut column: u32) -> (usize, u32, u32) {
    index = index.saturating_add(1);
    column = column.saturating_add(1);
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => {
                index = index.saturating_add(2);
                column = column.saturating_add(2);
            }
            b'"' => {
                return (index.saturating_add(1), line, column.saturating_add(1));
            }
            b'\n' => {
                line = line.saturating_add(1);
                column = 1;
                index = index.saturating_add(1);
            }
            _ => {
                index = index.saturating_add(1);
                column = column.saturating_add(1);
            }
        }
    }
    (index, line, column)
}

fn skip_char_literal(bytes: &[u8], index: usize, line: u32, column: u32) -> (usize, u32, u32) {
    let mut cursor = index.saturating_add(1);
    let mut consumed = 1usize;
    if bytes.get(cursor) == Some(&b'\\') {
        cursor = cursor.saturating_add(2);
        consumed = consumed.saturating_add(2);
    } else if bytes.get(cursor).is_some() {
        cursor = cursor.saturating_add(1);
        consumed = consumed.saturating_add(1);
    }
    if bytes.get(cursor) == Some(&b'\'') {
        return (
            cursor.saturating_add(1),
            line,
            column.saturating_add(u32::try_from(consumed).unwrap_or(u32::MAX)).saturating_add(1),
        );
    }
    (index.saturating_add(1), line, column.saturating_add(1))
}

fn advance_position(bytes: &[u8], start: usize, end: usize, mut line: u32, mut column: u32) -> (usize, u32, u32) {
    let mut index = start;
    while index < end {
        if bytes[index] == b'\n' {
            line = line.saturating_add(1);
            column = 1;
        } else {
            column = column.saturating_add(1);
        }
        index = index.saturating_add(1);
    }
    (end, line, column)
}

fn print_candidate_report(output_format: OutputFormat, report: &ProofCandidateReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove candidates total={} discharged={} unsupported_lints={} diagnostics={}",
                report.candidates.len(),
                report.discharged,
                report.unsupported_lints.len(),
                report.diagnostics.len()
            );
            let mut by_lint: BTreeMap<&str, Vec<&ProofCandidate>> = BTreeMap::new();
            for candidate in &report.candidates {
                by_lint.entry(&candidate.lint).or_default().push(candidate);
            }
            for (lint, candidates) in by_lint {
                let first = candidates.first().copied();
                println!(
                    "lint={} severity={} group={} candidates={}",
                    lint,
                    first.map_or("unknown", |candidate| candidate.severity.as_str()),
                    first.map_or("tigerstyle::unknown", |candidate| candidate.group.as_str()),
                    candidates.len()
                );
                for candidate in candidates {
                    println!(
                        "candidate rank={} score={} trust_boundary={} path={} line={} column={} attribute={} proof_ref={} rail={} status={}",
                        candidate.rank,
                        candidate.rank_score,
                        candidate.trust_boundary,
                        candidate.path,
                        candidate.line,
                        candidate.column,
                        candidate.attribute,
                        candidate.proof_ref.status,
                        candidate
                            .rail
                            .as_ref()
                            .map_or("unsupported".to_owned(), |rail| rail.proof_or_property_id.clone()),
                        candidate.rail.as_ref().map_or("unsupported", |_| "mapped")
                    );
                    for diagnostic in &candidate.proof_ref.diagnostics {
                        println!("candidate-diagnostic: {diagnostic}");
                    }
                }
            }
            for unsupported in &report.unsupported_lints {
                println!(
                    "unsupported lint={} severity={} group={} candidates={}",
                    unsupported.lint, unsupported.severity, unsupported.group, unsupported.candidates
                );
            }
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &report.non_claims {
                println!("non-claim: {non_claim}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            let text =
                serde_json::to_string_pretty(report).map_err(|error| format!("serialize candidates: {error}"))?;
            println!("{text}");
            Ok(())
        }
        OutputFormat::Sarif => Err("prove --candidates does not support SARIF output; use human or json".to_owned()),
    }
}

/// Records a v2 negative-control record for the selected proof card.
///
/// This wrapper retains the original call shape. Without explicit verifier evidence the recorder
/// cannot attribute a mutant rejection, so the written record is `inconclusive`. Callers that wire
/// `--verifier-evidence` must use [`run_record_negative_check_with_evidence`].
#[cfg(test)]
pub(crate) fn run_record_negative_check(args: &NegativeCheckArgs, output_format: OutputFormat) -> Result<i32, String> {
    run_record_negative_check_with_evidence(args, None, output_format)
}

/// Records a v2 negative-control record using the explicit typed verifier evidence at
/// `verifier_evidence`. The recorder measures both source and deck bytes, binds both proof cards,
/// and recomputes the decision from the retained artifacts.
#[allow(
    clippy::too_many_lines,
    reason = "the negative-check path keeps deck loading, card binding, measurement, and report writing in one auditable sequence"
)]
pub(crate) fn run_record_negative_check_with_evidence(
    args: &NegativeCheckArgs,
    verifier_evidence: Option<&Path>,
    output_format: OutputFormat,
) -> Result<i32, String> {
    let baseline_text = fs::read_to_string(&args.proof_cards)
        .map_err(|error| format!("read {}: {error}", args.proof_cards.display()))?;
    let mutant_text = fs::read_to_string(&args.mutant_cards)
        .map_err(|error| format!("read {}: {error}", args.mutant_cards.display()))?;
    let mut baseline_value: serde_json::Value = serde_json::from_str(&baseline_text)
        .map_err(|error| format!("parse {} as JSON: {error}", args.proof_cards.display()))?;
    let mutant_value: serde_json::Value = serde_json::from_str(&mutant_text)
        .map_err(|error| format!("parse {} as JSON: {error}", args.mutant_cards.display()))?;
    check_negative_deck_schema(&baseline_value, &args.proof_cards)?;
    check_negative_deck_schema(&mutant_value, &args.mutant_cards)?;
    let baseline_deck: proof_ref::ProofCardDeck = serde_json::from_value(baseline_value.clone())
        .map_err(|error| format!("parse {} as proof cards: {error}", args.proof_cards.display()))?;
    let mutant_deck: proof_ref::ProofCardDeck = serde_json::from_value(mutant_value)
        .map_err(|error| format!("parse {} as proof cards: {error}", args.mutant_cards.display()))?;
    let baseline_deck_blake3 = blake3::hash(baseline_text.as_bytes()).to_hex().to_string();
    let mutant_deck_blake3 = blake3::hash(mutant_text.as_bytes()).to_hex().to_string();

    let baseline_positions = deck_card_positions(&baseline_deck, &args.proof_or_property_id);
    if baseline_positions.len() != 1 {
        return Err(format!(
            "baseline deck must contain exactly one card for `{}`; found {}",
            args.proof_or_property_id,
            baseline_positions.len()
        ));
    }
    let baseline_index = baseline_positions[0];
    let baseline_card = &baseline_deck.cards[baseline_index];
    let mutant_positions = deck_card_positions(&mutant_deck, &args.proof_or_property_id);
    let mut diagnostics = Vec::new();
    let mutant_card = match mutant_positions.as_slice() {
        [index] => Some(&mutant_deck.cards[*index]),
        _ => {
            diagnostics.push(format!(
                "mutant deck must contain exactly one card for `{}`; found {}",
                args.proof_or_property_id,
                mutant_positions.len()
            ));
            None
        }
    };

    let evidence = match verifier_evidence {
        Some(path) => Some(proof_ref::read_negative_check_evidence(path)?),
        None => None,
    };
    let measured = match (evidence.as_ref(), mutant_card) {
        (Some(evidence), Some(mutant_card)) => Some(proof_ref::NegativeControlMeasurement {
            baseline_source_blake3: measure_source_bytes(&evidence.baseline.source_path),
            mutant_source_blake3: measure_source_bytes(&evidence.mutant.source_path),
            mutation_id: evidence.mutation.mutation_id.clone(),
            patch_blake3: evidence.mutation.patch_blake3.clone(),
            baseline_deck_blake3: baseline_deck_blake3.clone(),
            mutant_deck_blake3: mutant_deck_blake3.clone(),
            binding: negative_card_binding(baseline_card, mutant_card),
        }),
        _ => None,
    };
    let mut record = proof_ref::ProofCardNegativeCheck {
        schema_version: NEGATIVE_CHECK_SCHEMA_VERSION.to_owned(),
        status: NEGATIVE_STATUS_INCONCLUSIVE.to_owned(),
        mutant_source: args.mutant_source.clone(),
        mutant_card_status: mutant_card.map_or("absent", |card| card.status.as_str()).to_owned(),
        mutant_card_evidence_tier: mutant_card.map_or("", |card| card.evidence_tier.as_str()).to_owned(),
        mutant_cards_blake3: mutant_deck_blake3,
        non_claims: vec![NEGATIVE_CHECK_NON_CLAIM.to_owned()],
        diagnostics: Vec::new(),
        evidence,
        measured,
    };
    let expected = proof_ref::NegativeControlExpectation {
        proof_or_property_id: &baseline_card.proof_or_property_id,
        source_requirement_or_policy_id: &baseline_card.source_requirement_or_policy_id,
        policy_id: &baseline_card.policy_id,
        checked_rust_source_scope: &baseline_card.checked_rust_source_scope,
        evidence_tier: &baseline_card.evidence_tier,
        baseline_card_status: &baseline_card.status,
        baseline_card_diagnostics: &baseline_card.diagnostics,
        verifier_mode: &baseline_card.verifier_mode,
        verifier_identity_summary: &baseline_card.verifier_identity_summary,
        toolchain_identity_summary: &baseline_card.toolchain_identity_summary,
        baseline_source_paths: &baseline_card.source_paths,
        require_readable_artifacts: true,
    };
    let verdict = proof_ref::negative_control_verdict(&record, &expected);
    record.status = verdict.status.to_owned();
    diagnostics.extend(verdict.problems);
    diagnostics.sort();
    diagnostics.dedup();
    record.diagnostics = diagnostics.clone();

    let Some(card) = baseline_value
        .get_mut("cards")
        .and_then(serde_json::Value::as_array_mut)
        .and_then(|cards| cards.get_mut(baseline_index))
        .and_then(serde_json::Value::as_object_mut)
    else {
        return Err(format!("baseline card `{}` is not a JSON object", args.proof_or_property_id));
    };
    card.insert(
        "negative_check".to_owned(),
        serde_json::to_value(&record).map_err(|error| format!("serialize negative check: {error}"))?,
    );
    let output_path = args.output.as_ref().unwrap_or(&args.proof_cards);
    if paths_alias(output_path, &args.mutant_cards) {
        return Err("negative-check output must not overwrite the mutant proof-cards deck".to_owned());
    }
    if let Some(evidence_path) = verifier_evidence
        && paths_alias(output_path, evidence_path)
    {
        return Err("negative-check output must not overwrite the verifier evidence input".to_owned());
    }
    let text = serde_json::to_string_pretty(&baseline_value)
        .map_err(|error| format!("serialize proof cards: {error}"))?
        + "\n";
    fs::write(output_path, text).map_err(|error| format!("write {}: {error}", output_path.display()))?;
    let report = NegativeCheckReport {
        schema_version: NEGATIVE_CHECK_SCHEMA_VERSION.to_owned(),
        proof_or_property_id: args.proof_or_property_id.clone(),
        status: record.status.clone(),
        mutant_source: args.mutant_source.clone(),
        mutant_cards: args.mutant_cards.display().to_string(),
        deck: output_path.display().to_string(),
        verifier_evidence: verifier_evidence.map(|path| path.display().to_string()),
        measurement: record.measured.clone(),
        diagnostics,
        non_claims: vec![NEGATIVE_CHECK_NON_CLAIM.to_owned()],
    };
    print_negative_check_report(output_format, &report)?;
    Ok(i32::from(record.status != NEGATIVE_STATUS_DEMONSTRATED))
}

fn check_negative_deck_schema(deck: &serde_json::Value, path: &Path) -> Result<(), String> {
    match deck.get("schema_version").and_then(serde_json::Value::as_str) {
        Some(NEGATIVE_CHECK_DECK_SCHEMA_VERSION) => Ok(()),
        Some(other) => Err(format!(
            "proof-cards deck `{}` declares unsupported schema `{other}`; expected `{NEGATIVE_CHECK_DECK_SCHEMA_VERSION}`",
            path.display()
        )),
        None => Err(format!(
            "proof-cards deck `{}` does not declare schema `{NEGATIVE_CHECK_DECK_SCHEMA_VERSION}`",
            path.display()
        )),
    }
}

fn deck_card_positions(deck: &proof_ref::ProofCardDeck, proof_or_property_id: &str) -> Vec<usize> {
    deck.cards
        .iter()
        .enumerate()
        .filter(|(_, card)| card.proof_or_property_id == proof_or_property_id)
        .map(|(index, _)| index)
        .collect()
}

fn measure_source_bytes(path: &str) -> String {
    fs::read(path).map_or_else(|_| String::new(), |bytes| blake3::hash(&bytes).to_hex().to_string())
}

fn negative_card_binding(
    baseline: &proof_ref::ProofCard,
    mutant: &proof_ref::ProofCard,
) -> proof_ref::NegativeControlCardBinding {
    proof_ref::NegativeControlCardBinding {
        baseline_proof_or_property_id: baseline.proof_or_property_id.clone(),
        mutant_proof_or_property_id: mutant.proof_or_property_id.clone(),
        baseline_source_requirement_or_policy_id: baseline.source_requirement_or_policy_id.clone(),
        mutant_source_requirement_or_policy_id: mutant.source_requirement_or_policy_id.clone(),
        baseline_policy_id: baseline.policy_id.clone(),
        mutant_policy_id: mutant.policy_id.clone(),
        baseline_checked_rust_source_scope: baseline.checked_rust_source_scope.clone(),
        mutant_checked_rust_source_scope: mutant.checked_rust_source_scope.clone(),
        baseline_evidence_tier: baseline.evidence_tier.clone(),
        mutant_evidence_tier: mutant.evidence_tier.clone(),
        baseline_verifier_mode: baseline.verifier_mode.clone(),
        mutant_verifier_mode: mutant.verifier_mode.clone(),
        baseline_verifier_identity_summary: baseline.verifier_identity_summary.clone(),
        mutant_verifier_identity_summary: mutant.verifier_identity_summary.clone(),
        baseline_toolchain_identity_summary: baseline.toolchain_identity_summary.clone(),
        mutant_toolchain_identity_summary: mutant.toolchain_identity_summary.clone(),
        baseline_source_paths: baseline.source_paths.clone(),
        mutant_source_paths: mutant.source_paths.clone(),
    }
}

fn paths_alias(left: &Path, right: &Path) -> bool {
    left == right
        || matches!(
            (fs::canonicalize(left), fs::canonicalize(right)),
            (Ok(left), Ok(right)) if left == right
        )
}

fn print_negative_check_report(output_format: OutputFormat, report: &NegativeCheckReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove negative-check proof={} status={} mutant_cards={} deck={} diagnostics={}",
                report.proof_or_property_id,
                report.status,
                report.mutant_cards,
                report.deck,
                report.diagnostics.len()
            );
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &report.non_claims {
                println!("non-claim: {non_claim}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            let text =
                serde_json::to_string_pretty(report).map_err(|error| format!("serialize negative check: {error}"))?;
            println!("{text}");
            Ok(())
        }
        OutputFormat::Sarif => Err("prove --record-negative-check does not support SARIF output".to_owned()),
    }
}

#[cfg(test)]
mod tests;
