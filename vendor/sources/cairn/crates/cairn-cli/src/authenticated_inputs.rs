use std::fs;
use std::path::{Path, PathBuf};

use cairn_core::{
    AuthenticatedEvidencePolicy, AuthenticatedEvidenceReport, MeasuredEvidenceInput,
    PolicyExportObservation,
};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::Deserialize;
use serde_json::{json, Value};

const ED25519_PUBLIC_KEY_BYTES: usize = 32;
const ED25519_SIGNATURE_BYTES: usize = 64;
const HEX_PAIR_BYTES: usize = 2;
const NIBBLE_BITS: u8 = 4;
const HEX_ALPHA_OFFSET: u8 = 10;
#[cfg(test)]
const HEX_ALPHABET_LENGTH: usize = 16;
#[cfg(test)]
const LOW_NIBBLE_MASK: u8 = 0x0f;

#[derive(Debug, Clone)]
pub(crate) struct AuthenticationArtifact {
    pub(crate) role: &'static str,
    pub(crate) path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct AuthenticationComponent {
    pub(crate) check: Value,
    pub(crate) value: Value,
    pub(crate) report: Option<AuthenticatedEvidenceReport>,
}

mod dual_run;

#[derive(Debug, Deserialize)]
struct AuthenticationManifest {
    schema_version: String,
    evidence_kind: String,
    canonicalization_profile: String,
    supplied_receipt_hash: String,
    measurements: Vec<DeclaredMeasurement>,
    declared_policy_export_blake3: String,
    required_parent_hashes: Vec<String>,
    valence_integrity_receipt_hash: String,
    signature: DeclaredSignature,
}

#[derive(Debug, Deserialize)]
struct DeclaredMeasurement {
    role: String,
    path: String,
    blake3: String,
}

#[derive(Debug, Deserialize)]
struct DeclaredSignature {
    algorithm: String,
    producer: String,
    key_id: String,
    purpose: String,
    signature_hex: String,
    #[serde(default)]
    artifact_auth_signature_hex: Option<String>,
}

// r[impl cairn.stack_provenance_gates.authenticated_inputs.measurement]
// r[impl cairn.stack_provenance_gates.authenticated_inputs.receipt_identity]
// r[impl cairn.stack_provenance_gates.authenticated_inputs.signature_policy]
pub(crate) fn authentication_component(
    root: &Path,
    policy_value: Option<&Value>,
    evidence_kind: &str,
    authentication_path: Option<PathBuf>,
    artifacts: Vec<AuthenticationArtifact>,
    primary_receipt_role: &str,
) -> AuthenticationComponent {
    let Some(authentication_path) = authentication_path else {
        return skipped_component(evidence_kind);
    };
    let result = dual_run::build_authenticated_dual_run(
        root,
        policy_value,
        evidence_kind,
        &authentication_path,
        &artifacts,
        primary_receipt_role,
    );
    match result {
        Ok(run) => dual_run::successful_component(evidence_kind, authentication_path, run),
        Err(error) => AuthenticationComponent {
            check: json!({
                "name": format!("{evidence_kind}_authentication"),
                "passed": false,
                "source": "measured local bytes and configured Ed25519 trust policy",
                "path": authentication_path,
                "issue": error,
            }),
            value: json!({
                "evidence_kind": evidence_kind,
                "passed": false,
                "shell_error": error,
            }),
            report: None,
        },
    }
}

fn read_manifest(path: &Path) -> Result<AuthenticationManifest, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn measure_artifacts(
    artifacts: &[AuthenticationArtifact],
    declared: &[DeclaredMeasurement],
) -> Result<Vec<MeasuredEvidenceInput>, String> {
    if artifacts.len() != declared.len() {
        return Err(format!(
            "authentication manifest declares {} measurements for {} supplied artifacts",
            declared.len(),
            artifacts.len()
        ));
    }
    let mut measurements = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let path = artifact
            .path
            .as_ref()
            .ok_or_else(|| format!("missing evidence path for role {}", artifact.role))?;
        let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let actual_path = path.display().to_string();
        let declaration = declared
            .iter()
            .find(|entry| entry.role == artifact.role && entry.path == actual_path);
        measurements.push(MeasuredEvidenceInput {
            role: artifact.role.to_string(),
            path: actual_path,
            declared_blake3: declaration
                .map(|entry| entry.blake3.clone())
                .unwrap_or_default(),
            measured_blake3: blake3::hash(&bytes).to_hex().to_string(),
        });
    }
    Ok(measurements)
}

fn recompute_receipt_identity(path: &Path) -> Result<(String, String, String), String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut value: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse receipt {}: {error}", path.display()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| format!("receipt {} is not a JSON object", path.display()))?;
    let supplied = object
        .remove("receipt_hash")
        .and_then(|hash| hash.as_str().map(ToOwned::to_owned))
        .ok_or_else(|| format!("receipt {} has no receipt_hash self-hash", path.display()))?;
    let canonical_receipt = cairn_core::canonical_json(&value);
    let receipt_hash = blake3::hash(canonical_receipt.as_bytes())
        .to_hex()
        .to_string();
    Ok((supplied, receipt_hash, canonical_receipt))
}

fn measure_policy_export(
    root: &Path,
    policy: &AuthenticatedEvidencePolicy,
    evidence_kind: &str,
    manifest: &AuthenticationManifest,
) -> Result<PolicyExportObservation, String> {
    let export = policy
        .policy_exports
        .iter()
        .find(|export| export.evidence_kind == evidence_kind)
        .ok_or_else(|| format!("no configured policy export for {evidence_kind}"))?;
    let configured_path = PathBuf::from(&export.path);
    let path = if configured_path.is_absolute() {
        configured_path
    } else {
        root.join(configured_path)
    };
    let bytes = fs::read(&path)
        .map_err(|error| format!("read policy export {}: {error}", path.display()))?;
    Ok(PolicyExportObservation {
        path: export.path.clone(),
        declared_blake3: manifest.declared_policy_export_blake3.clone(),
        measured_blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn derive_required_parent_hashes(
    measurements: &[MeasuredEvidenceInput],
    primary_receipt_role: &str,
    policy_export: &PolicyExportObservation,
) -> Vec<String> {
    let mut parent_hashes = measurements
        .iter()
        .filter(|measurement| measurement.role != primary_receipt_role)
        .map(|measurement| measurement.measured_blake3.clone())
        .collect::<Vec<_>>();
    parent_hashes.push(policy_export.measured_blake3.clone());
    parent_hashes.sort();
    parent_hashes.dedup();
    parent_hashes
}

fn measure_valence_integrity_receipt(
    root: &Path,
    policy: &AuthenticatedEvidencePolicy,
    declared_blake3: &str,
) -> Result<MeasuredEvidenceInput, String> {
    let configured_path = PathBuf::from(&policy.accepted_valence_integrity_receipt_path);
    let path = if configured_path.is_absolute() {
        configured_path
    } else {
        root.join(configured_path)
    };
    let bytes = fs::read(&path)
        .map_err(|error| format!("read Valence integrity receipt {}: {error}", path.display()))?;
    Ok(MeasuredEvidenceInput {
        role: "valence_integrity_receipt".to_string(),
        path: policy.accepted_valence_integrity_receipt_path.clone(),
        declared_blake3: declared_blake3.to_string(),
        measured_blake3: blake3::hash(&bytes).to_hex().to_string(),
    })
}

fn verify_signature(public_key_hex: Option<&str>, signature_hex: &str, message: &[u8]) -> bool {
    let Some(public_key_hex) = public_key_hex else {
        return false;
    };
    let Ok(public_key_bytes) = decode_fixed_hex::<ED25519_PUBLIC_KEY_BYTES>(public_key_hex) else {
        return false;
    };
    let Ok(signature_bytes) = decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(signature_hex) else {
        return false;
    };
    let Ok(public_key) = VerifyingKey::from_bytes(&public_key_bytes) else {
        return false;
    };
    let signature = Signature::from_bytes(&signature_bytes);
    public_key.verify(message, &signature).is_ok()
}

fn decode_fixed_hex<const SIZE: usize>(value: &str) -> Result<[u8; SIZE], ()> {
    let expected_length = SIZE.checked_mul(HEX_PAIR_BYTES).ok_or(())?;
    if value.len() != expected_length {
        return Err(());
    }
    let mut bytes = [0_u8; SIZE];
    for (index, pair) in value.as_bytes().chunks_exact(HEX_PAIR_BYTES).enumerate() {
        let high = decode_nibble(pair[0]).ok_or(())?;
        let low = decode_nibble(pair[1]).ok_or(())?;
        bytes[index] = (high << NIBBLE_BITS) | low;
    }
    Ok(bytes)
}

fn decode_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + HEX_ALPHA_OFFSET),
        _ => None,
    }
}

fn skipped_component(evidence_kind: &str) -> AuthenticationComponent {
    AuthenticationComponent {
        check: json!({
            "name": format!("{evidence_kind}_authentication"),
            "passed": true,
            "skipped": true,
            "source": "authentication manifest not supplied",
        }),
        value: json!({
            "evidence_kind": evidence_kind,
            "passed": true,
            "skipped": true,
        }),
        report: None,
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
