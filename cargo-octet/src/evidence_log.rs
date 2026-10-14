use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const EVIDENCE_LOG_MANIFEST_ARTIFACT: &str = "evidence-log-manifest.json";
pub const EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION: &str = "octet-evidence-log-manifest/v1";
pub const RETENTION_CONTENT_ADDRESSED_RAW: &str = "content_addressed_raw";
pub const RETENTION_COMPRESSED_CONTENT_ADDRESSED_RAW: &str = "compressed_content_addressed_raw";
pub const RETENTION_INLINE_SUMMARY_EXTERNAL_RAW: &str = "inline_summary_external_raw";

const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_ARTIFACT_LOCATOR: &str = "artifact_locator";
const FIELD_BLAKE3: &str = "blake3";
const FIELD_BYTES: &str = "bytes";
const FIELD_REPLAY_COMMAND: &str = "replay_command";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_CAPTURE_SOURCE: &str = "capture_source";
const FIELD_RETENTION: &str = "retention";
const BLAKE3_HEX_LEN: usize = 64;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceLogManifest {
    pub schema_version: String,
    pub logs: Vec<EvidenceLogEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EvidenceLogEntry {
    pub artifact_locator: String,
    pub blake3: String,
    pub bytes: usize,
    pub replay_command: String,
    pub checked_scope: String,
    pub capture_source: String,
    pub retention: String,
}

impl EvidenceLogEntry {
    #[must_use]
    pub fn from_bytes(
        artifact_locator: &str,
        bytes: &[u8],
        replay_command: &str,
        checked_scope: &str,
        capture_source: &str,
        retention: &str,
    ) -> Self {
        Self {
            artifact_locator: artifact_locator.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(bytes),
            bytes: bytes.len(),
            replay_command: replay_command.to_owned(),
            checked_scope: checked_scope.to_owned(),
            capture_source: capture_source.to_owned(),
            retention: retention.to_owned(),
        }
    }
}

#[must_use]
pub fn manifest_from_entries(logs: Vec<EvidenceLogEntry>) -> EvidenceLogManifest {
    EvidenceLogManifest {
        schema_version: EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION.to_owned(),
        logs,
    }
}

#[must_use]
pub fn supported_retention_classifications() -> BTreeSet<&'static str> {
    [
        RETENTION_CONTENT_ADDRESSED_RAW,
        RETENTION_COMPRESSED_CONTENT_ADDRESSED_RAW,
        RETENTION_INLINE_SUMMARY_EXTERNAL_RAW,
    ]
    .into_iter()
    .collect()
}

#[must_use]
pub fn manifest_artifact_locators(manifest: &EvidenceLogManifest) -> Vec<String> {
    let mut locators = manifest.logs.iter().map(|entry| entry.artifact_locator.clone()).collect::<Vec<_>>();
    locators.sort();
    locators.dedup();
    locators
}

#[must_use]
pub fn validate_evidence_log_manifest(
    manifest: &EvidenceLogManifest,
    artifacts: &BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let mut diagnostics = manifest_shape_diagnostics(manifest);
    diagnostics.extend(entry_identity_diagnostics(manifest, artifacts));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn manifest_shape_diagnostics(manifest: &EvidenceLogManifest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if is_blank(&manifest.schema_version) {
        diagnostics.push(missing_manifest_field(FIELD_SCHEMA_VERSION));
    } else if manifest.schema_version != EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported evidence log manifest schema_version `{}`; expected `{}`",
            manifest.schema_version, EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.logs.is_empty() {
        diagnostics.push("evidence log manifest field `logs` must contain at least one entry".to_owned());
    }

    let supported = supported_retention_classifications();
    let mut locator_counts = BTreeMap::<String, usize>::new();
    for entry in &manifest.logs {
        *locator_counts.entry(entry.artifact_locator.clone()).or_default() += 1;
        diagnostics.extend(entry_shape_diagnostics(entry, &supported));
    }
    for (locator, count) in locator_counts {
        if count > 1 {
            diagnostics.push(format!(
                "evidence log entry `{locator}` field `{FIELD_ARTIFACT_LOCATOR}` is duplicated {count} times"
            ));
        }
    }
    diagnostics
}

fn entry_shape_diagnostics(entry: &EvidenceLogEntry, supported: &BTreeSet<&'static str>) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_blank_field(&mut diagnostics, entry, FIELD_ARTIFACT_LOCATOR, &entry.artifact_locator);
    if !is_normalized_relative_locator(&entry.artifact_locator) {
        diagnostics.push(format!(
            "evidence log entry `{}` field `{FIELD_ARTIFACT_LOCATOR}` must be a normalized relative path",
            entry.artifact_locator
        ));
    }
    push_blank_field(&mut diagnostics, entry, FIELD_BLAKE3, &entry.blake3);
    if !is_blake3_hex(&entry.blake3) {
        diagnostics.push(format!(
            "evidence log entry `{}` field `{FIELD_BLAKE3}` has malformed BLAKE3 digest `{}`",
            entry.artifact_locator, entry.blake3
        ));
    }
    push_blank_field(&mut diagnostics, entry, FIELD_REPLAY_COMMAND, &entry.replay_command);
    push_blank_field(&mut diagnostics, entry, FIELD_CHECKED_SCOPE, &entry.checked_scope);
    push_blank_field(&mut diagnostics, entry, FIELD_CAPTURE_SOURCE, &entry.capture_source);
    push_blank_field(&mut diagnostics, entry, FIELD_RETENTION, &entry.retention);
    if !supported.contains(entry.retention.as_str()) {
        diagnostics.push(format!(
            "evidence log entry `{}` field `{FIELD_RETENTION}` has unsupported classification `{}`",
            entry.artifact_locator, entry.retention
        ));
    }
    diagnostics
}

fn entry_identity_diagnostics(manifest: &EvidenceLogManifest, artifacts: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for entry in &manifest.logs {
        let Some(bytes) = artifacts.get(&entry.artifact_locator) else {
            diagnostics.push(format!("evidence log entry `{}` references missing artifact", entry.artifact_locator));
            continue;
        };
        let actual_blake3 = crate::evidence::artifact_blake3_hex(bytes);
        if entry.blake3 != actual_blake3 {
            diagnostics.push(format!(
                "evidence log entry `{}` field `{FIELD_BLAKE3}` mismatch; expected `{}`, got `{actual_blake3}`",
                entry.artifact_locator, entry.blake3
            ));
        }
        if entry.bytes != bytes.len() {
            diagnostics.push(format!(
                "evidence log entry `{}` field `{FIELD_BYTES}` mismatch; expected `{}`, got `{}`",
                entry.artifact_locator,
                entry.bytes,
                bytes.len()
            ));
        }
    }
    diagnostics
}

fn missing_manifest_field(field: &str) -> String {
    format!("evidence log manifest missing required field `{field}`")
}

fn push_blank_field(diagnostics: &mut Vec<String>, entry: &EvidenceLogEntry, field: &str, value: &str) {
    if is_blank(value) {
        diagnostics.push(format!("evidence log entry `{}` missing required field `{field}`", entry.artifact_locator));
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_normalized_relative_locator(value: &str) -> bool {
    !is_blank(value)
        && !value.starts_with('/')
        && !value.starts_with('\\')
        && !value.contains("\\")
        && !value.split('/').any(|component| matches!(component, "" | "." | ".."))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_LOCATOR: &str = "proofs/octet-rail-0.stdout";
    const LOG_BYTES: &[u8] = b"proof-result: pass fixture-verus\n";
    const REPLAY_COMMAND: &str = "cargo octet prove --registry policies.json --source src/lib.rs";
    const CHECKED_SCOPE: &str = "formal-proof:fixture-only:octet.verus_proof_rails.fixture#rail-0";
    const CAPTURE_SOURCE: &str = "octet.prove.stdout";

    #[test]
    fn complete_manifest_entry_with_matching_artifact_passes() {
        let manifest = valid_manifest();
        let artifacts = valid_artifacts();

        let diagnostics = validate_evidence_log_manifest(&manifest, &artifacts);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn manifest_artifact_locators_are_sorted_and_deduplicated() {
        let mut manifest = valid_manifest();
        manifest.logs.push(EvidenceLogEntry::from_bytes(
            "proofs/octet-rail-0.stderr",
            b"",
            REPLAY_COMMAND,
            CHECKED_SCOPE,
            "octet.prove.stderr",
            RETENTION_CONTENT_ADDRESSED_RAW,
        ));
        manifest.logs.push(manifest.logs[0].clone());

        let locators = manifest_artifact_locators(&manifest);

        assert_eq!(locators, vec!["proofs/octet-rail-0.stderr".to_owned(), ARTIFACT_LOCATOR.to_owned(),]);
    }

    #[test]
    fn missing_artifact_fails_closed() {
        let manifest = valid_manifest();
        let artifacts = BTreeMap::new();

        let diagnostics = validate_evidence_log_manifest(&manifest, &artifacts);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("references missing artifact")));
    }

    #[test]
    fn wrong_digest_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].blake3 = crate::evidence::artifact_blake3_hex(b"different log bytes");

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("field `blake3` mismatch")));
    }

    #[test]
    fn stale_byte_count_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].bytes += 1;

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("field `bytes` mismatch")));
    }

    #[test]
    fn missing_replay_command_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].replay_command.clear();

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("field `replay_command`")));
    }

    #[test]
    fn missing_checked_scope_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].checked_scope.clear();

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("field `checked_scope`")));
    }

    #[test]
    fn unsupported_retention_classification_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].retention = "keep_forever_unverified".to_owned();

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported classification")));
    }

    #[test]
    fn malformed_locator_digest_and_duplicate_entries_fail_closed() {
        let mut manifest = valid_manifest();
        manifest.logs[0].artifact_locator = "../proofs/log.txt".to_owned();
        manifest.logs[0].blake3 = "BADC0FFEE".to_owned();
        manifest.logs.push(manifest.logs[0].clone());

        let diagnostics = validate_evidence_log_manifest(&manifest, &valid_artifacts());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("normalized relative path")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed BLAKE3 digest")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicated")));
    }

    fn valid_manifest() -> EvidenceLogManifest {
        manifest_from_entries(vec![EvidenceLogEntry::from_bytes(
            ARTIFACT_LOCATOR,
            LOG_BYTES,
            REPLAY_COMMAND,
            CHECKED_SCOPE,
            CAPTURE_SOURCE,
            RETENTION_CONTENT_ADDRESSED_RAW,
        )])
    }

    fn valid_artifacts() -> BTreeMap<String, Vec<u8>> {
        BTreeMap::from([(ARTIFACT_LOCATOR.to_owned(), LOG_BYTES.to_vec())])
    }
}
