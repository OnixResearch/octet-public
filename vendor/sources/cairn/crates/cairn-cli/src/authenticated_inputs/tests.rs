use std::error::Error;
use std::fs;

use cairn_core::{
    authenticated_signature_message, map_cairn_artifact_auth, AuthenticatedEvidenceInput,
    AuthenticatedEvidencePolicy, AuthenticatedPolicyExport, AuthenticatedTrustedProducer,
    CanonicalReceiptObservation, MeasuredEvidenceInput, PolicyExportObservation,
    SignatureVerificationObservation, AUTHENTICATED_EVIDENCE_CANONICALIZATION_CORRECTED_V1,
    AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE, AUTHENTICATED_EVIDENCE_SCHEMA_V1,
    AUTHENTICATED_EVIDENCE_SIGNATURE_ALGORITHM_ED25519,
};
use ed25519_dalek::{Signer, SigningKey};
use serde_json::json;

use super::test_support::{encode_hex, file_hash, measured, write_manifest};
use super::{authentication_component, AuthenticationArtifact, ED25519_PUBLIC_KEY_BYTES};

const TEST_SEED_BYTE: u8 = 7;
const TEST_PURPOSE: &str = "onix.stack-provenance.release-evidence.v1";
const TEST_POLICY_PATH: &str = "policy-export.json";
const TEST_VALENCE_PATH: &str = "valence-integrity.json";

#[test]
// r[verify cairn.stack_provenance_gates.authenticated_inputs.final_validation]
// r[verify cairn.stack_provenance_gates.authenticated_inputs.fixtures.positive]
// r[verify cairn.stack_provenance_gates.authenticated_inputs.fixtures.negative]
// r[verify cairn.artifact_auth.repository_extraction.cutover.positive]
// r[verify cairn.artifact_auth.repository_extraction.cutover.negative]
fn shell_dual_runs_baseline_and_tamper_while_preserving_legacy_rollback(
) -> Result<(), Box<dyn Error>> {
    let root =
        std::env::temp_dir().join(format!("cairn-authenticated-inputs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root)?;
    let receipt_path = root.join("receipt.json");
    let graph_path = root.join("graph.json");
    let policy_export_path = root.join(TEST_POLICY_PATH);
    let valence_path = root.join(TEST_VALENCE_PATH);

    let receipt_body = json!({"claim": "bounded"});
    let receipt_hash = cairn_core::canonical_json_hash(&receipt_body);
    let mut receipt = receipt_body;
    receipt["receipt_hash"] = json!(receipt_hash);
    fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;
    fs::write(&graph_path, b"{\"edges\":[]}")?;
    fs::write(&policy_export_path, b"{\"reviewed\":true}")?;
    fs::write(&valence_path, b"{\"profile\":\"corrected\"}")?;

    let policy_export_hash = file_hash(&policy_export_path)?;
    let valence_hash = file_hash(&valence_path)?;
    let signing_key = SigningKey::from_bytes(&[TEST_SEED_BYTE; ED25519_PUBLIC_KEY_BYTES]);
    let public_key_hex = encode_hex(signing_key.verifying_key().as_bytes());
    let policy = AuthenticatedEvidencePolicy {
        schema_version: AUTHENTICATED_EVIDENCE_SCHEMA_V1.to_string(),
        release_profile: "onix-release".to_string(),
        accepted_canonicalization_profiles: vec![
            AUTHENTICATED_EVIDENCE_CANONICALIZATION_CORRECTED_V1.to_string(),
        ],
        accepted_valence_integrity_receipt_path: TEST_VALENCE_PATH.to_string(),
        accepted_valence_integrity_receipt_hash: valence_hash.clone(),
        policy_exports: vec![AuthenticatedPolicyExport {
            evidence_kind: AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE.to_string(),
            path: TEST_POLICY_PATH.to_string(),
            digest_blake3: policy_export_hash.clone(),
            signature_purpose: TEST_PURPOSE.to_string(),
        }],
        trusted_producers: vec![AuthenticatedTrustedProducer {
            producer: "test-producer".to_string(),
            key_id: "test-key".to_string(),
            algorithm: AUTHENTICATED_EVIDENCE_SIGNATURE_ALGORITHM_ED25519.to_string(),
            public_key_hex: public_key_hex.clone(),
            allowed_purposes: vec![TEST_PURPOSE.to_string()],
            revoked: false,
        }],
    };
    let policy_value = json!({"authenticated_evidence_policy": policy});
    let receipt_measurement = measured(&receipt_path, "receipt")?;
    let graph_measurement = measured(&graph_path, "graph")?;
    let valence_measurement = MeasuredEvidenceInput {
        role: "valence_integrity_receipt".to_string(),
        path: TEST_VALENCE_PATH.to_string(),
        declared_blake3: valence_hash.clone(),
        measured_blake3: valence_hash.clone(),
    };
    let mut parent_hashes = vec![
        graph_measurement.measured_blake3.clone(),
        policy_export_hash.clone(),
        valence_hash.clone(),
    ];
    parent_hashes.sort();
    let message_input = AuthenticatedEvidenceInput {
        schema_version: AUTHENTICATED_EVIDENCE_SCHEMA_V1.to_string(),
        evidence_kind: AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE.to_string(),
        measurements: vec![
            receipt_measurement.clone(),
            graph_measurement.clone(),
            valence_measurement,
        ],
        receipt: CanonicalReceiptObservation {
            profile: AUTHENTICATED_EVIDENCE_CANONICALIZATION_CORRECTED_V1.to_string(),
            supplied_receipt_hash: receipt_hash.clone(),
            recomputed_receipt_hash: receipt_hash.clone(),
        },
        policy_export: PolicyExportObservation {
            path: TEST_POLICY_PATH.to_string(),
            declared_blake3: policy_export_hash.clone(),
            measured_blake3: policy_export_hash.clone(),
        },
        signature: SignatureVerificationObservation {
            algorithm: AUTHENTICATED_EVIDENCE_SIGNATURE_ALGORITHM_ED25519.to_string(),
            verified: false,
            producer: "test-producer".to_string(),
            key_id: "test-key".to_string(),
            purpose: TEST_PURPOSE.to_string(),
            subject_hash: receipt_hash.clone(),
            parent_hashes: parent_hashes.clone(),
            public_key_blake3: blake3::hash(public_key_hex.as_bytes()).to_hex().to_string(),
        },
        required_parent_hashes: parent_hashes.clone(),
        valence_integrity_receipt_hash: valence_hash.clone(),
    };
    let signature = signing_key.sign(
        authenticated_signature_message(
            &message_input,
            &cairn_core::canonical_json(&json!({"claim": "bounded"})),
        )
        .as_bytes(),
    );
    let mapped = map_cairn_artifact_auth(&policy, &message_input)
        .map_err(|issues| format!("map standalone test statement: {issues:?}"))?;
    let standalone_preimage = artifact_auth_core::canonical_statement_bytes(&mapped.statement)
        .map_err(|issues| format!("canonicalize standalone test statement: {issues:?}"))?;
    let standalone_signature = signing_key.sign(&standalone_preimage);
    let manifest_path = root.join("authentication.json");
    let manifest_value = json!({
        "schema_version": AUTHENTICATED_EVIDENCE_SCHEMA_V1,
        "evidence_kind": AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE,
        "canonicalization_profile": AUTHENTICATED_EVIDENCE_CANONICALIZATION_CORRECTED_V1,
        "supplied_receipt_hash": receipt_hash,
        "measurements": [
            {
                "role": "receipt",
                "path": receipt_path.display().to_string(),
                "blake3": receipt_measurement.declared_blake3,
            },
            {
                "role": "graph",
                "path": graph_path.display().to_string(),
                "blake3": graph_measurement.declared_blake3,
            },
        ],
        "declared_policy_export_blake3": policy_export_hash,
        "required_parent_hashes": parent_hashes,
        "valence_integrity_receipt_hash": valence_hash,
        "signature": {
            "algorithm": AUTHENTICATED_EVIDENCE_SIGNATURE_ALGORITHM_ED25519,
            "producer": "test-producer",
            "key_id": "test-key",
            "purpose": TEST_PURPOSE,
            "signature_hex": encode_hex(&signature.to_bytes()),
            "artifact_auth_signature_hex": encode_hex(&standalone_signature.to_bytes()),
        },
    });
    write_manifest(&manifest_path, &manifest_value)?;
    let artifacts = || {
        vec![
            AuthenticationArtifact {
                role: "receipt",
                path: Some(receipt_path.clone()),
            },
            AuthenticationArtifact {
                role: "graph",
                path: Some(graph_path.clone()),
            },
        ]
    };
    let passing = authentication_component(
        &root,
        Some(&policy_value),
        AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE,
        Some(manifest_path.clone()),
        artifacts(),
        "receipt",
    );
    assert_eq!(
        passing.report.as_ref().map(|report| report.passed),
        Some(true)
    );
    assert_eq!(
        passing.value["artifact_auth_standalone"]["decision_passed"],
        true
    );
    assert_eq!(
        passing.value["artifact_auth_compatibility"]["case_explained"],
        true
    );
    assert_eq!(
        passing.value["artifact_auth_compatibility"]["legacy_authoritative"],
        true
    );
    assert_eq!(
        passing.value["artifact_auth_compatibility"]["standalone_authority_admitted"],
        false
    );

    let mut missing_signature_manifest = manifest_value.clone();
    missing_signature_manifest["signature"]
        .as_object_mut()
        .ok_or("signature fixture is not an object")?
        .remove("artifact_auth_signature_hex");
    write_manifest(&manifest_path, &missing_signature_manifest)?;
    let missing_signature = authentication_component(
        &root,
        Some(&policy_value),
        AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE,
        Some(manifest_path.clone()),
        artifacts(),
        "receipt",
    );
    assert_eq!(
        missing_signature
            .report
            .as_ref()
            .map(|report| report.passed),
        Some(true)
    );
    assert_eq!(missing_signature.check["passed"], true);
    assert_eq!(
        missing_signature.value["artifact_auth_compatibility"]["case_explained"],
        false
    );
    assert!(compatibility_blockers(&missing_signature)
        .contains(&"artifact_auth.compatibility.standalone.blocker"));

    let mut wrong_signature_manifest = manifest_value.clone();
    wrong_signature_manifest["signature"]["artifact_auth_signature_hex"] =
        wrong_signature_manifest["signature"]["signature_hex"].clone();
    write_manifest(&manifest_path, &wrong_signature_manifest)?;
    let wrong_signature = authentication_component(
        &root,
        Some(&policy_value),
        AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE,
        Some(manifest_path.clone()),
        artifacts(),
        "receipt",
    );
    assert_eq!(
        wrong_signature.report.as_ref().map(|report| report.passed),
        Some(true)
    );
    assert_eq!(
        wrong_signature.value["artifact_auth_standalone"]["decision_passed"],
        false
    );
    assert_eq!(wrong_signature.check["passed"], true);
    assert_eq!(
        wrong_signature.value["artifact_auth_compatibility"]["case_explained"],
        false
    );
    assert!(compatibility_blockers(&wrong_signature)
        .contains(&"artifact_auth.compatibility.decision.outcome"));

    write_manifest(&manifest_path, &manifest_value)?;
    fs::write(&graph_path, b"{\"edges\":[\"tampered\"]}")?;
    let failing = authentication_component(
        &root,
        Some(&policy_value),
        AUTHENTICATED_EVIDENCE_KIND_STACK_PROVENANCE,
        Some(manifest_path),
        artifacts(),
        "receipt",
    );
    let report = failing.report.as_ref().ok_or("missing failure report")?;
    assert!(!report.passed);
    assert!(report
        .issues
        .iter()
        .any(|issue| issue.code == "authenticated_inputs.measured_digest_mismatch"));
    assert_eq!(
        failing.value["artifact_auth_standalone"]["decision_passed"],
        false
    );
    assert_eq!(
        failing.value["artifact_auth_compatibility"]["case_explained"],
        true
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

fn compatibility_blockers(component: &super::AuthenticationComponent) -> Vec<&str> {
    component.value["artifact_auth_compatibility"]["blockers"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect()
}
