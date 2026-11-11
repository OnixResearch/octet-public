//! Bounded ELF/DWARF binary debug-map evidence rail.
//!
//! The pure core in [`core`] parses already-loaded bytes into normalized symbol,
//! address-range, and source-span rows. The shell in [`shell`] owns file access,
//! profile loading, and artifact linkage. This module owns the Nickel-authored
//! profile shape, the Rust-owned receipt DTOs, deterministic identities, and the
//! recorded-only handoff boundary.

pub(crate) mod core;
pub(crate) mod linkage;
pub(crate) mod shell;
#[cfg(test)]
mod tests;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const PROFILE_SCHEMA: &str = "octet-binary-debug-map-profiles/v1";
pub(crate) const PROFILE_ID: &str = "elf-dwarf-v1";
pub(crate) const RECEIPT_SCHEMA: &str = "octet-binary-debug-map-receipt/v1";
pub(crate) const ENVELOPE_SCHEMA: &str = "octet-binary-debug-map-analysis/v1";
pub(crate) const REQUIREMENT_REFS_SCHEMA: &str = "octet-binary-debug-map-requirement-refs/v1";
pub(crate) const RECORDED_ONLY_ROLE: &str = "recorded_only";
pub(crate) const REQUIRED_NON_CLAIM: &str = "Binary debug-map evidence records bounded ELF/DWARF linkage only and does not prove source-to-binary semantic equivalence, compiler correctness, or behavioral correctness";
pub(crate) const HANDOFF_ROLE_NON_CLAIM: &str = "Binary debug-map handoff preserves recorded linkage identity only and does not promote to formal proof, semantic equivalence, requirement satisfaction, behavioral correctness, or release eligibility";

const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const GIT_REVISION_HEX_LENGTH: usize = 40;
const SOURCE_PATH_HASH_DOMAIN: &[u8] = b"octet-binary-debug-map-source-path/v1\0";
const SYMBOL_NAME_HASH_DOMAIN: &[u8] = b"octet-binary-debug-map-symbol-name/v1\0";
const DEBUG_SOURCE_INLINE: &str = "inline";
const DEBUG_SOURCE_SPLIT: &str = "split";
const OBJECT_FORMAT_ELF: &str = "elf";
const DEBUG_FORMAT_DWARF: &str = "dwarf";
const ARCHITECTURE_X86_64: &str = "x86_64";
const ARCHITECTURE_AARCH64: &str = "aarch64";
const UNMAPPED_FAIL: &str = "fail";
const UNMAPPED_EXCLUDE: &str = "exclude";
const COMPANION_MATCH_BUILD_ID: &str = "build-id";
const SEVERITY_ERROR: &str = "error";
const SEVERITY_WARNING: &str = "warning";
const SEVERITY_MANUAL_REVIEW: &str = "manual_review";
const COMPANION_ROLE: &str = "split-debug";
const REQUIREMENT_ID_PREFIX: &str = "r[";
const REQUIREMENT_ID_SUFFIX: &str = "]";
const FORBIDDEN_CLAIM_PHRASES: &[&str] = &[
    "proves source-to-binary semantic equivalence",
    "proves compiler correctness",
    "proves behavioral correctness",
    "proves requirement satisfaction",
    "proves release eligibility",
    "formal proof",
];

/// The checked-in Nickel profile export. `standards/binary-debug-map/profiles.ncl`
/// is the authored source of truth; this is the reviewed runtime input.
#[cfg(test)]
pub(crate) const PROFILE_JSON: &str = include_str!("../../../standards/binary-debug-map/profiles.json");

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileRegistry {
    pub(crate) schema_version: String,
    pub(crate) profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceRoot {
    pub(crate) build_root: String,
    pub(crate) repository_root: String,
}

#[allow(
    clippy::struct_field_names,
    reason = "the profile schema names every resource bound with the shared `max_` prefix"
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResourceLimits {
    pub(crate) max_binary_bytes: u64,
    pub(crate) max_debug_bytes: u64,
    pub(crate) max_sections: u64,
    pub(crate) max_units: u64,
    pub(crate) max_symbols: u64,
    pub(crate) max_ranges: u64,
    pub(crate) max_line_rows: u64,
    pub(crate) max_frames: u64,
    pub(crate) max_path_bytes: u64,
    pub(crate) max_rows: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProfileSeverities {
    pub(crate) malformed: String,
    pub(crate) unsupported: String,
    pub(crate) wrong_companion: String,
    pub(crate) unmapped_path: String,
    pub(crate) stale_span: String,
    pub(crate) ambiguous_function: String,
    pub(crate) resource_limit: String,
}

#[allow(
    clippy::struct_field_names,
    reason = "the reviewed profile schema keeps its stable `profile_` field prefix"
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Profile {
    pub(crate) profile_id: String,
    pub(crate) summary: String,
    pub(crate) object_format: String,
    pub(crate) architecture: String,
    pub(crate) debug_format: String,
    pub(crate) required_sections: Vec<String>,
    pub(crate) source_roots: Vec<SourceRoot>,
    pub(crate) unmapped_path_policy: String,
    pub(crate) strict_linkage: bool,
    pub(crate) companion_match: String,
    pub(crate) resource_limits: ResourceLimits,
    pub(crate) severity: ProfileSeverities,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct DebugCompanionIdentity {
    pub(crate) identity: String,
    pub(crate) role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) build_identity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementReference {
    pub(crate) repository: String,
    pub(crate) revision: String,
    pub(crate) registry_identity: String,
    pub(crate) requirement_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BinaryDebugRow {
    pub(crate) index: u64,
    pub(crate) raw_symbol_name_hash: String,
    pub(crate) demangled_name: String,
    pub(crate) address_start: u64,
    pub(crate) address_end: u64,
    pub(crate) range_index: u64,
    pub(crate) repository_path: String,
    pub(crate) path_hash: String,
    pub(crate) line_start: u64,
    pub(crate) column_start: u64,
    pub(crate) line_end: u64,
    pub(crate) column_end: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) source_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) function_address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) requirement: Option<RequirementReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) legacy_requirement_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BinaryDebugMapFinding {
    pub(crate) issue_code: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) address: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct TruncationState {
    pub(crate) truncated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) limit: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BinaryDebugMapReceiptDetails {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) profile_identity: String,
    pub(crate) registry_identity: String,
    pub(crate) object_format: String,
    pub(crate) architecture: String,
    pub(crate) debug_format: String,
    pub(crate) binary_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) debug_companion: Option<DebugCompanionIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) build_identity: Option<String>,
    pub(crate) debug_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) function_address_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) requirement_refs_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) requirement_registry_identity: Option<String>,
    pub(crate) source_roots: Vec<SourceRoot>,
    pub(crate) limits: ResourceLimits,
    pub(crate) rows: Vec<BinaryDebugRow>,
    pub(crate) excluded_rows: u64,
    pub(crate) findings: Vec<BinaryDebugMapFinding>,
    pub(crate) truncation: TruncationState,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BinaryDebugMapEnvelope {
    pub(crate) schema_version: String,
    pub(crate) details: BinaryDebugMapReceiptDetails,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryDebugMapStatus {
    Passed,
    ManualReview,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BinaryDebugMapEvaluation {
    pub(crate) details: Option<BinaryDebugMapReceiptDetails>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) status: BinaryDebugMapStatus,
}

/// A parsed typed requirement-reference handoff.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementRefSet {
    pub(crate) schema_version: String,
    pub(crate) repository: String,
    pub(crate) revision: String,
    pub(crate) registry_identity: String,
    pub(crate) references: Vec<RequirementRefEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RequirementRefEntry {
    pub(crate) requirement_id: String,
    pub(crate) function_address: String,
    #[serde(default)]
    pub(crate) legacy_requirement_ids: Vec<String>,
}

pub(crate) fn parse_registry(text: &str) -> Result<ProfileRegistry, String> {
    serde_json::from_str(text).map_err(|error| format!("parse binary debug-map profile registry: {error}"))
}

#[cfg(test)]
pub(crate) fn checked_in_registry() -> Result<ProfileRegistry, String> {
    parse_registry(PROFILE_JSON)
}

pub(crate) fn selected_profile<'a>(registry: &'a ProfileRegistry, profile_id: &str) -> Result<&'a Profile, String> {
    registry
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown binary debug-map profile `{profile_id}`"))
}

pub(crate) fn validate_registry(registry: &ProfileRegistry) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if registry.schema_version != PROFILE_SCHEMA {
        diagnostics.push(format!(
            "unsupported binary debug-map profile schema `{}`; expected `{PROFILE_SCHEMA}`",
            registry.schema_version
        ));
    }
    if registry.profiles.is_empty() {
        diagnostics.push("binary debug-map profile registry must declare at least one profile".to_owned());
    }
    for profile in &registry.profiles {
        validate_profile(profile, &mut diagnostics);
    }
    let mut ids = std::collections::BTreeSet::new();
    for profile in &registry.profiles {
        if !ids.insert(profile.profile_id.clone()) {
            diagnostics.push(format!("duplicate binary debug-map profile id `{}`", profile.profile_id));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_profile(profile: &Profile, diagnostics: &mut Vec<String>) {
    if profile.profile_id != PROFILE_ID {
        diagnostics
            .push(format!("unsupported binary debug-map profile id `{}`; expected `{PROFILE_ID}`", profile.profile_id));
    }
    if profile.summary.trim().is_empty() {
        diagnostics.push("binary debug-map profile summary must not be empty".to_owned());
    }
    if profile.object_format != OBJECT_FORMAT_ELF {
        diagnostics.push(format!("unsupported object format `{}`; expected `elf`", profile.object_format));
    }
    if !matches!(profile.architecture.as_str(), ARCHITECTURE_X86_64 | ARCHITECTURE_AARCH64) {
        diagnostics.push(format!("unsupported binary debug-map architecture `{}`", profile.architecture));
    }
    if profile.debug_format != DEBUG_FORMAT_DWARF {
        diagnostics.push(format!("unsupported debug format `{}`; expected `dwarf`", profile.debug_format));
    }
    if profile.required_sections.is_empty() {
        diagnostics.push("binary debug-map profile must declare required debug sections".to_owned());
    }
    for section in &profile.required_sections {
        if !section.starts_with(".debug_") {
            diagnostics.push(format!("required section `{section}` is not a DWARF section name"));
        }
    }
    validate_source_roots(&profile.source_roots, diagnostics);
    if !matches!(profile.unmapped_path_policy.as_str(), UNMAPPED_FAIL | UNMAPPED_EXCLUDE) {
        diagnostics.push(format!("unsupported unmapped path policy `{}`", profile.unmapped_path_policy));
    }
    if profile.companion_match != COMPANION_MATCH_BUILD_ID {
        diagnostics.push(format!("unsupported companion match policy `{}`", profile.companion_match));
    }
    validate_limits(&profile.resource_limits, diagnostics);
    validate_severities(&profile.severity, diagnostics);
    if profile.verification_role != RECORDED_ONLY_ROLE {
        diagnostics.push(format!(
            "binary debug-map profile must declare `{RECORDED_ONLY_ROLE}` verification role, got `{}`",
            profile.verification_role
        ));
    }
    if !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("binary debug-map profile missing the required linkage non-claim".to_owned());
    }
    if profile.non_claims.iter().any(|claim| contains_forbidden_claim(claim)) {
        diagnostics.push("binary debug-map profile contains a promoted claim".to_owned());
    }
}

fn validate_source_roots(source_roots: &[SourceRoot], diagnostics: &mut Vec<String>) {
    if source_roots.is_empty() {
        diagnostics.push("binary debug-map profile must declare at least one source-root mapping".to_owned());
        return;
    }
    let mut seen = std::collections::BTreeSet::new();
    for mapping in source_roots {
        if !mapping.build_root.starts_with('/') {
            diagnostics.push(format!("source build root `{}` must be absolute", mapping.build_root));
        }
        if mapping.build_root.len() > 1 && mapping.build_root.ends_with('/') {
            diagnostics.push(format!("source build root `{}` must not end with a separator", mapping.build_root));
        }
        if mapping.repository_root.starts_with('/') {
            diagnostics
                .push(format!("repository source root `{}` must be repository-relative", mapping.repository_root));
        }
        if !is_safe_relative_path(&mapping.repository_root) {
            diagnostics.push(format!(
                "repository source root `{}` is not a canonical repository-relative path",
                mapping.repository_root
            ));
        }
        if !seen.insert(mapping.build_root.clone()) {
            diagnostics.push(format!("duplicate source build root `{}`", mapping.build_root));
        }
    }
    for left in source_roots {
        for right in source_roots {
            if left.build_root == right.build_root {
                continue;
            }
            if is_path_prefix(&right.build_root, &left.build_root) && left.repository_root != right.repository_root {
                diagnostics.push(format!("conflicting source-root mappings for build root `{}`", left.build_root));
            }
        }
    }
}

fn validate_limits(limits: &ResourceLimits, diagnostics: &mut Vec<String>) {
    for (name, value) in [
        ("max_binary_bytes", limits.max_binary_bytes),
        ("max_debug_bytes", limits.max_debug_bytes),
        ("max_sections", limits.max_sections),
        ("max_units", limits.max_units),
        ("max_symbols", limits.max_symbols),
        ("max_ranges", limits.max_ranges),
        ("max_line_rows", limits.max_line_rows),
        ("max_frames", limits.max_frames),
        ("max_path_bytes", limits.max_path_bytes),
        ("max_rows", limits.max_rows),
    ] {
        if value == 0 {
            diagnostics.push(format!("binary debug-map profile limit `{name}` must be greater than zero"));
        }
    }
}

fn validate_severities(severity: &ProfileSeverities, diagnostics: &mut Vec<String>) {
    for (name, value) in [
        ("malformed", &severity.malformed),
        ("unsupported", &severity.unsupported),
        ("wrong_companion", &severity.wrong_companion),
        ("unmapped_path", &severity.unmapped_path),
        ("stale_span", &severity.stale_span),
        ("ambiguous_function", &severity.ambiguous_function),
        ("resource_limit", &severity.resource_limit),
    ] {
        if !matches!(value.as_str(), SEVERITY_ERROR | SEVERITY_WARNING | SEVERITY_MANUAL_REVIEW) {
            diagnostics.push(format!("unsupported severity `{value}` for `{name}`"));
        }
    }
}

pub(crate) fn profile_identity(profile: &Profile) -> Result<String, String> {
    let bytes = serde_json::to_vec(profile).map_err(|error| format!("serialize binary debug-map profile: {error}"))?;
    Ok(identity(&bytes))
}

pub(crate) fn identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

pub(crate) fn path_hash(repository_path: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_PATH_HASH_DOMAIN);
    hasher.update(repository_path.as_bytes());
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub(crate) fn symbol_name_hash(raw_name: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SYMBOL_NAME_HASH_DOMAIN);
    hasher.update(raw_name.as_bytes());
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub(crate) fn display_name(raw_name: &str) -> String {
    match rustc_demangle::try_demangle(raw_name) {
        Ok(demangled) => format!("{demangled:#}"),
        Err(_) => raw_name.to_owned(),
    }
}

pub(crate) fn parse_requirement_refs(text: &str) -> Result<RequirementRefSet, String> {
    serde_json::from_str(text).map_err(|error| format!("parse binary debug-map requirement references: {error}"))
}

pub(crate) fn validate_requirement_refs(references: &RequirementRefSet) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if references.schema_version != REQUIREMENT_REFS_SCHEMA {
        diagnostics.push(format!(
            "unsupported requirement reference schema `{}`; expected `{REQUIREMENT_REFS_SCHEMA}`",
            references.schema_version
        ));
    }
    if references.repository.trim().is_empty() {
        diagnostics.push("requirement reference repository must not be empty".to_owned());
    }
    if !is_revision_identity(&references.revision) {
        diagnostics.push("requirement reference revision must be a Git commit or BLAKE3 identity".to_owned());
    }
    if !is_b3_identity(&references.registry_identity) {
        diagnostics.push("requirement reference registry identity must be a BLAKE3 identity".to_owned());
    }
    let mut function_addresses = std::collections::BTreeSet::new();
    for entry in &references.references {
        if !is_requirement_id(&entry.requirement_id) {
            diagnostics.push(format!("malformed requirement id `{}`", entry.requirement_id));
        }
        if entry.function_address.trim().is_empty() {
            diagnostics.push("requirement reference missing function address".to_owned());
        }
        if !function_addresses.insert(entry.function_address.clone()) {
            diagnostics
                .push(format!("duplicate requirement reference for function address `{}`", entry.function_address));
        }
        for legacy in &entry.legacy_requirement_ids {
            if legacy.trim().is_empty() {
                diagnostics.push("legacy requirement id must not be empty".to_owned());
            }
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn evaluate_envelope(text: &str) -> BinaryDebugMapEvaluation {
    let parsed = serde_json::from_str::<BinaryDebugMapEnvelope>(text);
    let mut diagnostics = Vec::new();
    let details = match parsed {
        Ok(envelope) => {
            if envelope.schema_version != ENVELOPE_SCHEMA {
                diagnostics.push(format!(
                    "unsupported binary debug-map envelope schema `{}`; expected `{ENVELOPE_SCHEMA}`",
                    envelope.schema_version
                ));
            }
            validate_receipt_details(&envelope.details, &envelope.non_claims, &mut diagnostics);
            Some(envelope.details)
        }
        Err(error) => {
            diagnostics.push(format!("malformed binary debug-map envelope: {error}"));
            None
        }
    };
    diagnostics.sort();
    diagnostics.dedup();
    let status = status_for(details.as_ref(), &diagnostics);
    let non_claims = details.as_ref().map_or_else(Vec::new, |details| details.non_claims.clone());
    BinaryDebugMapEvaluation {
        details,
        non_claims,
        diagnostics,
        status,
    }
}

pub(crate) fn status_for(
    details: Option<&BinaryDebugMapReceiptDetails>,
    diagnostics: &[String],
) -> BinaryDebugMapStatus {
    if !diagnostics.is_empty() {
        return BinaryDebugMapStatus::Failed;
    }
    let Some(details) = details else {
        return BinaryDebugMapStatus::Failed;
    };
    if details.findings.iter().any(|finding| finding.severity == SEVERITY_ERROR) {
        return BinaryDebugMapStatus::Failed;
    }
    if details.truncation.truncated {
        return BinaryDebugMapStatus::Failed;
    }
    if details.findings.iter().any(|finding| finding.severity == SEVERITY_MANUAL_REVIEW) {
        return BinaryDebugMapStatus::ManualReview;
    }
    BinaryDebugMapStatus::Passed
}

pub(crate) fn validate_receipt_details(
    details: &BinaryDebugMapReceiptDetails,
    non_claims: &[String],
    diagnostics: &mut Vec<String>,
) {
    if details.schema_version != RECEIPT_SCHEMA {
        diagnostics.push(format!(
            "unsupported binary debug-map receipt schema `{}`; expected `{RECEIPT_SCHEMA}`",
            details.schema_version
        ));
    }
    if details.profile_id != PROFILE_ID {
        diagnostics.push(format!("binary debug-map receipt uses unsupported profile `{}`", details.profile_id));
    }
    for (field, value) in [
        ("profile_identity", &details.profile_identity),
        ("registry_identity", &details.registry_identity),
        ("binary_identity", &details.binary_identity),
    ] {
        if !is_b3_identity(value) {
            diagnostics.push(format!("binary debug-map receipt `{field}` must be a BLAKE3 identity"));
        }
    }
    if details.object_format != OBJECT_FORMAT_ELF {
        diagnostics
            .push(format!("binary debug-map receipt records unsupported object format `{}`", details.object_format));
    }
    if details.debug_format != DEBUG_FORMAT_DWARF {
        diagnostics
            .push(format!("binary debug-map receipt records unsupported debug format `{}`", details.debug_format));
    }
    if !matches!(details.debug_source.as_str(), DEBUG_SOURCE_INLINE | DEBUG_SOURCE_SPLIT) {
        diagnostics
            .push(format!("binary debug-map receipt records unsupported debug source `{}`", details.debug_source));
    }
    if let Some(companion) = &details.debug_companion {
        if !is_b3_identity(&companion.identity) {
            diagnostics.push("binary debug-map companion identity must be a BLAKE3 identity".to_owned());
        }
        if companion.role != COMPANION_ROLE {
            diagnostics.push(format!("binary debug-map companion role `{}` is not `{COMPANION_ROLE}`", companion.role));
        }
    }
    if let Some(build_identity) = &details.build_identity
        && build_identity.trim().is_empty()
    {
        diagnostics.push("binary debug-map build identity must not be empty".to_owned());
    }
    if details.verification_role != RECORDED_ONLY_ROLE {
        diagnostics.push(format!(
            "binary debug-map receipt must remain `{RECORDED_ONLY_ROLE}`, got `{}`",
            details.verification_role
        ));
    }
    diagnostics.extend(validate_handoff_role(&details.verification_role));
    if !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
        && !non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("binary debug-map receipt missing the required linkage non-claim".to_owned());
    }
    if details.non_claims.iter().any(|claim| contains_forbidden_claim(claim)) {
        diagnostics.push("binary debug-map receipt contains a promoted claim".to_owned());
    }
    if let Some(registry) = &details.requirement_registry_identity
        && !is_b3_identity(registry)
    {
        diagnostics.push("binary debug-map requirement registry identity must be a BLAKE3 identity".to_owned());
    }
    for (field, value) in [
        ("function_address_identity", details.function_address_identity.as_deref()),
        ("requirement_refs_identity", details.requirement_refs_identity.as_deref()),
    ] {
        if let Some(value) = value
            && !is_b3_identity(value)
        {
            diagnostics.push(format!("binary debug-map receipt `{field}` must be a BLAKE3 identity"));
        }
    }
    validate_rows(details, diagnostics);
    if details.truncation.truncated && details.truncation.limit.is_none() {
        diagnostics.push("binary debug-map truncation state must name the exceeded limit".to_owned());
    }
}

fn validate_rows(details: &BinaryDebugMapReceiptDetails, diagnostics: &mut Vec<String>) {
    if u64::try_from(details.rows.len()).unwrap_or(u64::MAX) > details.limits.max_rows {
        diagnostics.push("binary debug-map receipt exceeds its row limit".to_owned());
    }
    let mut indexes = std::collections::BTreeSet::new();
    for row in &details.rows {
        if !indexes.insert(row.index) {
            diagnostics.push(format!("duplicate binary debug-map row index {}", row.index));
        }
        if !is_b3_identity(&row.raw_symbol_name_hash) {
            diagnostics.push(format!("row {} symbol-name hash must be a BLAKE3 identity", row.index));
        }
        if !is_b3_identity(&row.path_hash) {
            diagnostics.push(format!("row {} path hash must be a BLAKE3 identity", row.index));
        }
        if row.demangled_name.trim().is_empty() {
            diagnostics.push(format!("row {} demangled display name must not be empty", row.index));
        }
        if !is_safe_relative_path(&row.repository_path) {
            diagnostics.push(format!("row {} repository path is not canonical and repository-relative", row.index));
        }
        if row.path_hash != path_hash(&row.repository_path) {
            diagnostics.push(format!("row {} path hash does not bind its repository path", row.index));
        }
        if row.address_start >= row.address_end {
            diagnostics.push(format!("row {} has an empty or inverted address range", row.index));
        }
        if row.line_start == 0 || row.line_end < row.line_start {
            diagnostics.push(format!("row {} has an invalid source line span", row.index));
        }
        if row.column_end < row.column_start {
            diagnostics.push(format!("row {} has an invalid source column span", row.index));
        }
        if let Some(source_identity) = &row.source_identity
            && source_identity.trim().is_empty()
        {
            diagnostics.push(format!("row {} source identity must not be empty", row.index));
        }
        if let Some(function_address) = &row.function_address
            && function_address.trim().is_empty()
        {
            diagnostics.push(format!("row {} function address must not be empty", row.index));
        }
        if let Some(requirement) = &row.requirement {
            validate_requirement_reference(requirement, row.index, diagnostics);
            if let Some(registry) = &details.requirement_registry_identity
                && requirement.registry_identity != *registry
            {
                diagnostics.push(format!(
                    "row {} requirement registry identity does not match the receipt registry",
                    row.index
                ));
            }
        }
        for legacy in &row.legacy_requirement_ids {
            if legacy.trim().is_empty() {
                diagnostics.push(format!("row {} legacy requirement id must not be empty", row.index));
            }
        }
    }
}

fn validate_requirement_reference(reference: &RequirementReference, row_index: u64, diagnostics: &mut Vec<String>) {
    if reference.repository.trim().is_empty() {
        diagnostics.push(format!("row {row_index} requirement reference repository must not be empty"));
    }
    if !is_revision_identity(&reference.revision) {
        diagnostics.push(format!("row {row_index} requirement revision must be a Git commit or BLAKE3 identity"));
    }
    if !is_b3_identity(&reference.registry_identity) {
        diagnostics.push(format!("row {row_index} requirement registry identity must be a BLAKE3 identity"));
    }
    if !is_requirement_id(&reference.requirement_id) {
        diagnostics.push(format!("row {row_index} requirement id is malformed"));
    }
    if !linkage::is_typed_requirement_reference(reference) {
        diagnostics.push(format!("row {row_index} requirement reference is not a typed reference"));
    }
}

/// Validates a downstream handoff role. Only recorded linkage roles are accepted;
/// formal-proof, semantic-equivalence, requirement-satisfaction, behavioral, and
/// release roles are rejected without changing the original evidence identity.
pub(crate) fn validate_handoff_role(role: &str) -> Vec<String> {
    if role == RECORDED_ONLY_ROLE {
        return Vec::new();
    }
    vec![format!(
        "binary debug-map evidence role `{role}` promotes recorded linkage; only `{RECORDED_ONLY_ROLE}` is admitted"
    )]
}

pub(crate) fn contains_forbidden_claim(text: &str) -> bool {
    let lowered = text.to_lowercase();
    FORBIDDEN_CLAIM_PHRASES.iter().any(|phrase| lowered.contains(phrase))
}

pub(crate) fn is_b3_identity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(BLAKE3_PREFIX) else {
        return false;
    };
    hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn is_revision_identity(value: &str) -> bool {
    if is_b3_identity(value) {
        return true;
    }
    value.len() == GIT_REVISION_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn is_requirement_id(value: &str) -> bool {
    value.starts_with(REQUIREMENT_ID_PREFIX)
        && value.ends_with(REQUIREMENT_ID_SUFFIX)
        && value.len() > REQUIREMENT_ID_PREFIX.len() + REQUIREMENT_ID_SUFFIX.len()
        && !value.contains(char::is_whitespace)
}

pub(crate) fn is_safe_relative_path(value: &str) -> bool {
    if value.is_empty() || value.starts_with('/') || value.contains('\\') || value.contains('\0') {
        return false;
    }
    value.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}

pub(crate) fn is_path_prefix(prefix: &str, path: &str) -> bool {
    path == prefix || (path.starts_with(prefix) && path.as_bytes().get(prefix.len()) == Some(&b'/'))
}

#[cfg(test)]
pub(crate) fn profile_for_tests() -> Profile {
    let registry = checked_in_registry().unwrap_or_else(|error| panic!("checked-in profile: {error}"));
    selected_profile(&registry, PROFILE_ID)
        .unwrap_or_else(|error| panic!("checked-in profile id: {error}"))
        .clone()
}
