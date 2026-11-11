//! Exact binary/source/requirement linkage for the binary debug-map rail.
//!
//! Linkage is pure: it compares normalized row positions with supplied
//! function-address and typed requirement-reference artifacts. A demangled
//! symbol-name match alone never selects a candidate.

use serde::Deserialize;

use super::RequirementRefSet;
use super::RequirementReference;
use super::identity;
use super::is_b3_identity;
use super::is_requirement_id;
use super::is_revision_identity;
use super::is_safe_relative_path;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct FunctionAddressArtifact {
    #[serde(skip)]
    pub(crate) identity: String,
    #[serde(default)]
    pub(crate) records: Vec<FunctionAddressRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct FunctionAddressRecord {
    pub(crate) path: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) line: u64,
    #[serde(default)]
    pub(crate) column: u64,
    #[serde(default)]
    pub(crate) start_byte: u64,
    #[serde(default)]
    pub(crate) end_byte: u64,
    pub(crate) function_address: String,
    #[serde(default)]
    pub(crate) source_identity: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LinkRequest<'a> {
    pub(crate) repository_path: &'a str,
    pub(crate) source_name: &'a str,
    pub(crate) line_start: u64,
    pub(crate) line_end: u64,
    pub(crate) column_start: u64,
    pub(crate) column_end: u64,
    pub(crate) function_addresses: Option<&'a FunctionAddressArtifact>,
    pub(crate) requirement_refs: Option<&'a RequirementRefSet>,
    pub(crate) strict: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LinkIssue {
    AmbiguousFunction,
    StaleSpan,
    AmbiguousRequirement,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LinkResult {
    pub(crate) function_address: Option<String>,
    pub(crate) source_identity: Option<String>,
    pub(crate) requirement: Option<RequirementReference>,
    pub(crate) legacy_requirement_ids: Vec<String>,
    pub(crate) issue: Option<LinkIssue>,
    pub(crate) strict: bool,
}

pub(crate) fn parse_function_address_artifact(bytes: &[u8]) -> Result<FunctionAddressArtifact, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|error| format!("function-address artifact is not UTF-8: {error}"))?;
    let mut artifact: FunctionAddressArtifact =
        serde_json::from_str(text).map_err(|error| format!("parse function-address artifact: {error}"))?;
    artifact.identity = identity(bytes);
    validate_function_address_artifact(&artifact)?;
    Ok(artifact)
}

pub(crate) fn validate_function_address_artifact(artifact: &FunctionAddressArtifact) -> Result<(), String> {
    let mut diagnostics = Vec::new();
    if artifact.records.is_empty() {
        diagnostics.push("function-address artifact must contain at least one record".to_owned());
    }
    let mut identities = std::collections::BTreeSet::new();
    for record in &artifact.records {
        if !is_safe_relative_path(&record.path) {
            diagnostics.push("function-address record path is not canonical and repository-relative".to_owned());
        }
        if record.name.trim().is_empty() {
            diagnostics.push("function-address record name must not be empty".to_owned());
        }
        if record.function_address.trim().is_empty() {
            diagnostics.push("function-address record address must not be empty".to_owned());
        }
        if record.line == 0 {
            diagnostics.push("function-address record line must be greater than zero".to_owned());
        }
        if record.start_byte > record.end_byte {
            diagnostics.push("function-address record byte span is inverted".to_owned());
        }
        if !identities.insert(record.function_address.clone()) {
            diagnostics.push(format!("duplicate function-address record `{}`", record.function_address));
        }
        if let Some(source_identity) = &record.source_identity
            && source_identity.trim().is_empty()
        {
            diagnostics.push("function-address record source identity must not be empty".to_owned());
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid function-address artifact:\n- {}", diagnostics.join("\n- ")))
    }
}

pub(crate) fn validate_requirement_ref_set(references: &RequirementRefSet) -> Result<(), String> {
    let diagnostics = super::validate_requirement_refs(references);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid requirement reference handoff:\n- {}", diagnostics.join("\n- ")))
    }
}

pub(crate) fn link_row(request: &LinkRequest<'_>) -> LinkResult {
    let mut result = LinkResult {
        strict: request.strict,
        ..LinkResult::default()
    };
    let Some(artifact) = request.function_addresses else {
        return result;
    };
    let same_symbol = artifact
        .records
        .iter()
        .filter(|record| record.path == request.repository_path && record.name == request.source_name);
    let mut in_span = Vec::new();
    let mut same_symbol_count = 0_u64;
    for record in same_symbol {
        same_symbol_count = same_symbol_count.saturating_add(1);
        if span_contains(request, record) {
            in_span.push(record);
        }
    }
    if in_span.len() == 1 {
        let record = in_span[0];
        result.function_address = Some(record.function_address.clone());
        result.source_identity.clone_from(&record.source_identity);
        link_requirement(request, record, &mut result);
        return result;
    }
    if in_span.len() > 1 {
        result.issue = Some(LinkIssue::AmbiguousFunction);
        return result;
    }
    if same_symbol_count > 0 {
        result.issue = Some(LinkIssue::StaleSpan);
        return result;
    }
    let overlapping = artifact
        .records
        .iter()
        .filter(|record| record.path == request.repository_path && span_contains(request, record))
        .count();
    if overlapping > 0 {
        result.issue = Some(LinkIssue::AmbiguousFunction);
    }
    result
}

fn span_contains(request: &LinkRequest<'_>, record: &FunctionAddressRecord) -> bool {
    if record.line < request.line_start || record.line > request.line_end {
        return false;
    }
    if record.line == request.line_start && request.column_start > 0 && record.column > 0 {
        return record.column >= request.column_start;
    }
    if record.line == request.line_end && request.column_end > 0 && record.column > 0 {
        return record.column <= request.column_end;
    }
    true
}

fn link_requirement(request: &LinkRequest<'_>, record: &FunctionAddressRecord, result: &mut LinkResult) {
    let Some(references) = request.requirement_refs else {
        return;
    };
    let mut matches = references
        .references
        .iter()
        .filter(|reference| reference.function_address == record.function_address);
    let Some(entry) = matches.next() else {
        return;
    };
    if matches.next().is_some() {
        result.issue = Some(LinkIssue::AmbiguousRequirement);
        return;
    }
    result.requirement = Some(RequirementReference {
        repository: references.repository.clone(),
        revision: references.revision.clone(),
        registry_identity: references.registry_identity.clone(),
        requirement_id: entry.requirement_id.clone(),
    });
    result.legacy_requirement_ids.clone_from(&entry.legacy_requirement_ids);
}

/// Whether a typed requirement reference is internally well-formed. Recorded-only
/// legacy text ids are deliberately not admitted as typed references.
pub(crate) fn is_typed_requirement_reference(reference: &RequirementReference) -> bool {
    !reference.repository.trim().is_empty()
        && is_revision_identity(&reference.revision)
        && is_b3_identity(&reference.registry_identity)
        && is_requirement_id(&reference.requirement_id)
}
