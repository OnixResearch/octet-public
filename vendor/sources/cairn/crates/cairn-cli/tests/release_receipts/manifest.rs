use super::support::*;
use serde_json::{json, Value};
use std::{error::Error, fs, path::PathBuf};

#[test]
fn signed_evidence_manifest_verifies_indexed_evidence_artifacts() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.signed_evidence_manifest.verify]
    // r[verify cairn.supersede.signed_evidence_manifest.chain]
    // r[verify cairn.supersede.signed_evidence_manifest.metadata]
    // r[verify cairn.supersede.signed_evidence_manifest.schema]
    // r[verify cairn.supersede.signed_evidence_manifest.tests]
    // r[verify cairn.supersede.signed_evidence_manifest.docs]
    // r[verify cairn.supersede.signed_evidence_manifest.gates]
    // r[verify cairn.supersede.signed_evidence_manifest.archive]
    // r[verify cairn.supersede.evidence_manifest_generate.cli]
    // r[verify cairn.supersede.evidence_manifest_generate.canonical]
    // r[verify cairn.supersede.evidence_manifest_generate.metadata]
    // r[verify cairn.supersede.evidence_manifest_generate.failures]
    // r[verify cairn.supersede.evidence_manifest_generate.docs]
    // r[verify cairn.supersede.evidence_manifest_generate.tests]
    // r[verify cairn.supersede.evidence_manifest_generate.gates]
    // r[verify cairn.supersede.evidence_manifest_generate.archive]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.bundle_manifest]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.failures]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.docs]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.archive]
    let dir = temp_project("signed-evidence-manifest")?;
    let replay_path = dir.join("agent-replay.json");
    let replay_verify_path = dir.join("agent-replay-verify.json");
    let release_path = dir.join("release-readiness.json");
    let coverage_path = dir.join("coverage-audit.json");
    let agent_path = dir.join("agent-receipt.json");
    let bundle_path = dir.join("bundle.json");
    let manifest_path = dir.join("manifest.json");

    let mut replay = cairn();
    replay.args([
        "determinism",
        "replay",
        "--root",
        ".",
        "--group",
        "agent",
        "--format",
        "summary",
        "--out",
        replay_path.to_str().unwrap(),
    ]);
    let output = replay.output()?;
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let replay_verify = run_json_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
        replay_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    fs::write(
        &replay_verify_path,
        serde_json::to_string_pretty(&replay_verify)? + "\n",
    )?;
    fs::write(
        &release_path,
        serde_json::to_string_pretty(&json!({
            "valid": true,
            "receipt_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        }))? + "\n",
    )?;
    let coverage = run_json_strings(&[
        "determinism".into(),
        "coverage-audit".into(),
        "--root".into(),
        ".".into(),
    ])?;
    fs::write(
        &coverage_path,
        serde_json::to_string_pretty(&coverage)? + "\n",
    )?;

    let mut agent_receipt: Value = serde_json::from_str(&fs::read_to_string(
        repo_root().join("fixtures/agent-receipts/positive.json"),
    )?)?;
    agent_receipt["replay_evidence"]["replay_receipt_hash"] =
        replay_verify["replay_receipt_hash"].clone();
    agent_receipt["replay_evidence"]["replay_policy_hash"] =
        replay_verify["replay_policy_hash"].clone();
    agent_receipt["replay_evidence"]["replay_environment_attestation_hash"] =
        replay_verify["environment_attestation_hash"].clone();
    agent_receipt
        .as_object_mut()
        .unwrap()
        .remove("receipt_hash");
    fs::write(
        &agent_path,
        serde_json::to_string_pretty(&agent_receipt)? + "\n",
    )?;
    let initial_agent_validation = run_json_strings(&[
        "agent-receipt".into(),
        "validate".into(),
        agent_path.display().to_string(),
    ])?;
    agent_receipt["receipt_hash"] = initial_agent_validation["agent_receipt_hash"].clone();
    fs::write(
        &agent_path,
        serde_json::to_string_pretty(&agent_receipt)? + "\n",
    )?;
    let agent_validation = run_json_strings(&[
        "agent-receipt".into(),
        "validate".into(),
        agent_path.display().to_string(),
    ])?;
    assert_eq!(agent_validation["valid"], true);

    let bundle = json!({
        "agent_receipt_path": agent_path,
        "replay_receipt_path": replay_path,
        "replay_verification_path": replay_verify_path,
        "release_readiness_path": release_path,
        "coverage_audit_path": coverage_path,
        "agent_receipt_hash": agent_validation["agent_receipt_hash"],
        "replay_receipt_hash": replay_verify["replay_receipt_hash"],
        "replay_verification_receipt_hash": replay_verify["receipt_hash"],
        "release_readiness_receipt_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "coverage_audit_receipt_hash": coverage["receipt_hash"],
    });
    fs::write(&bundle_path, serde_json::to_string_pretty(&bundle)? + "\n")?;
    let bundle_verify = run_json_strings(&[
        "agent-receipt".into(),
        "verify-bundle".into(),
        bundle_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(bundle_verify["valid"], true);
    assert_eq!(
        bundle_verify["coverage_audit_receipt_hash"],
        coverage["receipt_hash"]
    );
    let mut bundle_with_hash = bundle.clone();
    bundle_with_hash["receipt_hash"] = bundle_verify["receipt_hash"].clone();
    fs::write(
        &bundle_path,
        serde_json::to_string_pretty(&bundle_with_hash)? + "\n",
    )?;

    let manifest = json!({
        "artifacts": [
            {"role": "agent_evidence_bundle", "path": bundle_path, "hash": bundle_verify["receipt_hash"], "schema": "bundle-v1"},
            {"role": "agent_receipt", "path": agent_path, "hash": agent_validation["agent_receipt_hash"], "schema": "agent-receipt-v1"},
            {"role": "replay_receipt", "path": replay_path, "hash": replay_verify["replay_receipt_hash"], "schema": "determinism-replay-v1"},
            {"role": "replay_verification", "path": replay_verify_path, "hash": replay_verify["receipt_hash"], "schema": "determinism-verify-receipt-v1"},
            {"role": "release_readiness", "path": release_path, "hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "schema": "release-readiness-v1"},
            {"role": "coverage_audit", "path": coverage_path, "hash": coverage["receipt_hash"], "schema": "determinism-coverage-audit-v1"}
        ],
        "signer": {"kind": "ci", "id": "onix-ci"},
        "anchor": {"kind": "local-artifact-index", "id": "run-123"}
    });
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest)? + "\n",
    )?;
    let verified = run_json_strings(&[
        "evidence-manifest".into(),
        "verify".into(),
        manifest_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(verified["valid"], true);
    assert_eq!(verified["verdict"], "pass");
    assert_eq!(verified["artifacts"].as_array().unwrap().len(), 6);
    assert_eq!(
        verified["coverage_audit_receipt_hash"],
        coverage["receipt_hash"]
    );
    assert_hash_field(&verified, "manifest_hash")?;
    assert_hash_field(&verified, "signer_metadata_hash")?;
    assert_hash_field(&verified, "anchor_metadata_hash")?;
    assert_hash_field(&verified, "receipt_hash")?;

    let generated_manifest_path = dir.join("generated-manifest.json");
    let requirement_registry_path = dir.join("requirement-registry.json");
    let signer_path = dir.join("signer.json");
    let anchor_path = dir.join("anchor.json");
    let requirement_registry_export = run_json_strings(&[
        "requirement-registry".into(),
        "export".into(),
        "--root".into(),
        repo_root()
            .join("fixtures/content-bound-requirement-registry/current")
            .display()
            .to_string(),
        "--repository-id".into(),
        "OnixResearch/example".into(),
        "--revision-kind".into(),
        "opaque".into(),
        "--revision".into(),
        "reviewed-manifest-fixture".into(),
        "--out".into(),
        requirement_registry_path.display().to_string(),
    ])?;
    assert_eq!(requirement_registry_export["valid"], true);
    fs::write(&signer_path, "{\"id\":\"onix-ci\",\"kind\":\"ci\"}\n")?;
    fs::write(
        &anchor_path,
        "{\"id\":\"run-123\",\"kind\":\"local-artifact-index\"}\n",
    )?;
    let generated = run_json_strings(&[
        "evidence-manifest".into(),
        "generate".into(),
        "--bundle".into(),
        bundle_path.display().to_string(),
        "--agent-receipt".into(),
        agent_path.display().to_string(),
        "--replay".into(),
        replay_path.display().to_string(),
        "--replay-verification".into(),
        replay_verify_path.display().to_string(),
        "--release-readiness".into(),
        release_path.display().to_string(),
        "--coverage-audit".into(),
        coverage_path.display().to_string(),
        "--requirement-registry".into(),
        requirement_registry_path.display().to_string(),
        "--signer-metadata".into(),
        signer_path.display().to_string(),
        "--anchor-metadata".into(),
        anchor_path.display().to_string(),
        "--out".into(),
        generated_manifest_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(generated["valid"], true);
    assert_hash_field(&generated, "manifest_hash")?;
    let generated_manifest: Value =
        serde_json::from_str(&fs::read_to_string(&generated_manifest_path)?)?;
    let generated_roles = generated_manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["role"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        generated_roles,
        vec![
            "agent_evidence_bundle",
            "agent_receipt",
            "replay_receipt",
            "replay_verification",
            "release_readiness",
            "coverage_audit",
            "requirement_registry"
        ]
    );
    let requirement_registry_entry = generated_manifest["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["role"] == "requirement_registry")
        .expect("requirement registry manifest entry");
    let requirement_registry: Value =
        serde_json::from_str(&fs::read_to_string(&requirement_registry_path)?)?;
    assert_eq!(
        requirement_registry_entry["hash"],
        blake3::hash(&fs::read(&requirement_registry_path)?)
            .to_hex()
            .to_string()
    );
    assert_eq!(
        requirement_registry_entry["receipt_hash"],
        requirement_registry["receipt"]["receipt_blake3"]
    );
    assert_eq!(generated_manifest["signer"]["kind"], "ci");
    let generated_verified = run_json_strings(&[
        "evidence-manifest".into(),
        "verify".into(),
        generated_manifest_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(generated_verified["valid"], true);
    assert_eq!(
        generated_verified["coverage_audit_receipt_hash"],
        coverage["receipt_hash"]
    );
    assert_eq!(
        generated_verified["manifest_hash"],
        generated["manifest_hash"]
    );
    assert!(generated_verified["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|artifact| artifact["role"] == "requirement_registry" && artifact["valid"] == true));

    let mut wrong_registry_receipt_manifest = generated_manifest.clone();
    let wrong_registry_entry = wrong_registry_receipt_manifest["artifacts"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["role"] == "requirement_registry")
        .expect("mutable requirement registry manifest entry");
    wrong_registry_entry["receipt_hash"] = requirement_registry["registry_blake3"].clone();
    let wrong_registry_receipt_path = dir.join("wrong-registry-receipt-manifest.json");
    fs::write(
        &wrong_registry_receipt_path,
        serde_json::to_string_pretty(&wrong_registry_receipt_manifest)? + "\n",
    )?;
    let wrong_registry_receipt = run_json_failure_strings(&[
        "evidence-manifest".into(),
        "verify".into(),
        wrong_registry_receipt_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert!(wrong_registry_receipt["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["code"] == "artifact.requirement_registry_receipt_mismatch"));

    let missing_generation = run_json_failure_strings(&[
        "evidence-manifest".into(),
        "generate".into(),
        "--bundle".into(),
        PathBuf::from("/tmp/SECRET-missing-bundle.json")
            .display()
            .to_string(),
        "--out".into(),
        "-".into(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(missing_generation["valid"], false);
    let generation_text = serde_json::to_string(&missing_generation)?;
    assert!(
        !generation_text.contains(&dir.display().to_string()),
        "{generation_text}"
    );
    assert!(!generation_text.contains("SECRET"), "{generation_text}");

    fs::write(&signer_path, "[\"not-object\"]\n")?;
    let invalid_metadata = run_json_failure_strings(&[
        "evidence-manifest".into(),
        "generate".into(),
        "--bundle".into(),
        bundle_path.display().to_string(),
        "--agent-receipt".into(),
        agent_path.display().to_string(),
        "--replay".into(),
        replay_path.display().to_string(),
        "--replay-verification".into(),
        replay_verify_path.display().to_string(),
        "--signer-metadata".into(),
        signer_path.display().to_string(),
        "--out".into(),
        "-".into(),
        "--root".into(),
        ".".into(),
    ])?;
    let invalid_codes = invalid_metadata["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(invalid_codes.contains(&"generate.metadata_not_object"));

    let mut tampered = manifest.clone();
    tampered["artifacts"][0]["hash"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    tampered["signer"] = json!("SECRET=/tmp/private-token");
    let tampered_path = dir.join("tampered-manifest.json");
    fs::write(
        &tampered_path,
        serde_json::to_string_pretty(&tampered)? + "\n",
    )?;
    let failure = run_json_failure_strings(&[
        "evidence-manifest".into(),
        "verify".into(),
        tampered_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(failure["valid"], false);
    let issue_codes = failure["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(issue_codes.contains(&"artifact.hash_mismatch"));
    assert!(issue_codes.contains(&"manifest.metadata_invalid"));
    let failure_text = serde_json::to_string(&failure)?;
    assert!(
        !failure_text.contains(&dir.display().to_string()),
        "{failure_text}"
    );
    assert!(!failure_text.contains("SECRET"), "{failure_text}");
    Ok(())
}
