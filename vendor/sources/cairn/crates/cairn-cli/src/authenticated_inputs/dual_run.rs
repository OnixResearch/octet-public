use std::path::{Path, PathBuf};

use cairn_core::{
    authenticated_evidence_policy_from_value, authenticated_signature_message,
    compare_cairn_artifact_auth, evaluate_authenticated_evidence, evaluate_cairn_artifact_auth,
    map_cairn_artifact_auth, AuthenticatedEvidenceInput, AuthenticatedEvidenceIssue,
    AuthenticatedEvidencePolicy, AuthenticatedEvidenceReport, CairnArtifactAuthCompatibilityReport,
    CairnArtifactAuthReport, CanonicalReceiptObservation, SignatureVerificationObservation,
    AUTHENTICATED_EVIDENCE_SCHEMA_V1,
};
use serde_json::{json, Value};

use super::{
    decode_fixed_hex, derive_required_parent_hashes, measure_artifacts, measure_policy_export,
    measure_valence_integrity_receipt, read_manifest, recompute_receipt_identity, verify_signature,
    AuthenticationArtifact, AuthenticationComponent, ED25519_SIGNATURE_BYTES,
};

pub(super) struct AuthenticationDualRun {
    legacy: AuthenticatedEvidenceReport,
    standalone: Option<CairnArtifactAuthReport>,
    compatibility: CairnArtifactAuthCompatibilityReport,
}

struct StandaloneRun {
    statement_blake3: Option<String>,
    report: Option<CairnArtifactAuthReport>,
    blockers: Vec<AuthenticatedEvidenceIssue>,
}

pub(super) fn build_authenticated_dual_run(
    root: &Path,
    policy_value: Option<&Value>,
    evidence_kind: &str,
    authentication_path: &Path,
    artifacts: &[AuthenticationArtifact],
    primary_receipt_role: &str,
) -> Result<AuthenticationDualRun, String> {
    let policy_value = policy_value.ok_or_else(|| "policy could not be loaded".to_string())?;
    let policy = authenticated_evidence_policy_from_value(policy_value)?;
    let manifest = read_manifest(authentication_path)?;
    if manifest.schema_version != AUTHENTICATED_EVIDENCE_SCHEMA_V1 {
        return Err(format!(
            "authentication manifest schema {} is unsupported",
            manifest.schema_version
        ));
    }
    if manifest.evidence_kind != evidence_kind {
        return Err(format!(
            "authentication manifest kind {} does not match {evidence_kind}",
            manifest.evidence_kind
        ));
    }
    let mut measurements = measure_artifacts(artifacts, &manifest.measurements)?;
    let valence_integrity_measurement =
        measure_valence_integrity_receipt(root, &policy, &manifest.valence_integrity_receipt_hash)?;
    let valence_integrity_receipt_hash = valence_integrity_measurement.measured_blake3.clone();
    measurements.push(valence_integrity_measurement);
    let receipt_path = artifacts
        .iter()
        .find(|artifact| artifact.role == primary_receipt_role)
        .and_then(|artifact| artifact.path.as_ref())
        .ok_or_else(|| format!("missing primary receipt role {primary_receipt_role}"))?;
    let (embedded_receipt_hash, recomputed_receipt_hash, canonical_receipt) =
        recompute_receipt_identity(receipt_path)?;
    if embedded_receipt_hash != manifest.supplied_receipt_hash {
        return Err(format!(
            "authentication manifest supplied receipt hash {} does not match embedded receipt hash {}",
            manifest.supplied_receipt_hash, embedded_receipt_hash
        ));
    }
    let policy_export = measure_policy_export(root, &policy, evidence_kind, &manifest)?;
    let required_parent_hashes =
        derive_required_parent_hashes(&measurements, primary_receipt_role, &policy_export);
    let trusted_key = policy.trusted_producers.iter().find(|producer| {
        producer.producer == manifest.signature.producer
            && producer.key_id == manifest.signature.key_id
    });
    let public_key_blake3 = trusted_key
        .map(|producer| {
            blake3::hash(producer.public_key_hex.as_bytes())
                .to_hex()
                .to_string()
        })
        .unwrap_or_default();
    let mut input = AuthenticatedEvidenceInput {
        schema_version: manifest.schema_version.clone(),
        evidence_kind: manifest.evidence_kind.clone(),
        measurements,
        receipt: CanonicalReceiptObservation {
            profile: manifest.canonicalization_profile.clone(),
            supplied_receipt_hash: embedded_receipt_hash,
            recomputed_receipt_hash: recomputed_receipt_hash.clone(),
        },
        policy_export,
        signature: SignatureVerificationObservation {
            algorithm: manifest.signature.algorithm.clone(),
            verified: false,
            producer: manifest.signature.producer.clone(),
            key_id: manifest.signature.key_id.clone(),
            purpose: manifest.signature.purpose.clone(),
            subject_hash: recomputed_receipt_hash,
            parent_hashes: manifest.required_parent_hashes.clone(),
            public_key_blake3,
        },
        required_parent_hashes,
        valence_integrity_receipt_hash,
    };
    let message = authenticated_signature_message(&input, &canonical_receipt);
    let legacy_preimage_blake3 = blake3::hash(message.as_bytes()).to_hex().to_string();
    input.signature.verified = verify_signature(
        trusted_key.map(|producer| producer.public_key_hex.as_str()),
        &manifest.signature.signature_hex,
        message.as_bytes(),
    );
    let legacy = evaluate_authenticated_evidence(&policy, &input);
    let standalone = evaluate_standalone(
        &policy,
        &input,
        manifest.signature.artifact_auth_signature_hex.as_deref(),
    );
    let compatibility = compare_cairn_artifact_auth(
        &legacy_preimage_blake3,
        standalone.statement_blake3.as_deref(),
        &legacy,
        standalone.report.as_ref(),
        &standalone.blockers,
    );
    Ok(AuthenticationDualRun {
        legacy,
        standalone: standalone.report,
        compatibility,
    })
}

pub(super) fn successful_component(
    evidence_kind: &str,
    authentication_path: PathBuf,
    run: AuthenticationDualRun,
) -> AuthenticationComponent {
    let AuthenticationDualRun {
        legacy,
        standalone,
        compatibility,
    } = run;
    let is_case_explained = compatibility.case_explained;
    let mut value = serde_json::to_value(&legacy).unwrap_or_else(|error| {
        json!({
            "passed": false,
            "shell_error": error.to_string(),
        })
    });
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "artifact_auth_compatibility".to_string(),
            json!(compatibility),
        );
        object.insert("artifact_auth_standalone".to_string(), json!(standalone));
    }
    AuthenticationComponent {
        check: json!({
            "name": format!("{evidence_kind}_authentication"),
            "passed": legacy.passed,
            "source": "dual-run measured bytes; legacy Cairn report remains authoritative",
            "path": authentication_path,
            "authentication_hash": legacy.authentication_hash,
            "artifact_auth_case_explained": is_case_explained,
        }),
        value,
        report: Some(legacy),
    }
}

fn evaluate_standalone(
    policy: &AuthenticatedEvidencePolicy,
    input: &AuthenticatedEvidenceInput,
    signature_hex: Option<&str>,
) -> StandaloneRun {
    let mapped = match map_cairn_artifact_auth(policy, input) {
        Ok(mapped) => mapped,
        Err(blockers) => {
            return StandaloneRun {
                statement_blake3: None,
                report: None,
                blockers,
            };
        }
    };
    let mut blockers = Vec::new();
    let signature_bytes = decode_standalone_signature(signature_hex, &mut blockers);
    let cryptographic = artifact_auth_ed25519::verify_statement(
        &mapped.statement,
        &mapped.public_key_bytes,
        signature_bytes.as_deref().unwrap_or_default(),
    );
    let report = evaluate_cairn_artifact_auth(input, &mapped, cryptographic);
    StandaloneRun {
        statement_blake3: report.standalone_statement_blake3.clone(),
        report: Some(report),
        blockers,
    }
}

fn decode_standalone_signature(
    signature_hex: Option<&str>,
    blockers: &mut Vec<AuthenticatedEvidenceIssue>,
) -> Option<Vec<u8>> {
    let Some(signature_hex) = signature_hex else {
        blockers.push(issue(
            "artifact_auth.dual_run.signature_missing",
            "manifest.signature.artifact_auth_signature_hex",
            "a dedicated signature over artifact-auth statement bytes",
            "absent",
        ));
        return None;
    };
    match decode_fixed_hex::<ED25519_SIGNATURE_BYTES>(signature_hex) {
        Ok(bytes) => Some(bytes.to_vec()),
        Err(()) => {
            blockers.push(issue(
                "artifact_auth.dual_run.signature_malformed",
                "manifest.signature.artifact_auth_signature_hex",
                "an Ed25519 signature as lowercase hexadecimal bytes",
                "malformed signature encoding",
            ));
            None
        }
    }
}

fn issue(code: &str, field_path: &str, expected: &str, actual: &str) -> AuthenticatedEvidenceIssue {
    AuthenticatedEvidenceIssue {
        code: code.to_string(),
        field_path: field_path.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}
