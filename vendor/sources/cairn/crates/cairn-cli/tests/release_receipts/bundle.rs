use super::support::*;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;

#[test]
fn agent_evidence_bundle_verification_links_agent_replay_and_release_evidence(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.agent_evidence_bundle_verification.cli]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.validation]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.receipt]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.secret_safe]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.tests]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.archive]
    let dir = temp_project("agent-evidence-bundle")?;
    let replay_path = dir.join("agent-replay.json");
    let verify_path = dir.join("agent-replay-verify.json");
    let release_path = dir.join("release-readiness.json");
    let bundle_path = dir.join("bundle.json");

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
        &verify_path,
        serde_json::to_string_pretty(&replay_verify)? + "\n",
    )?;
    fs::write(
        &release_path,
        serde_json::to_string_pretty(&json!({
            "valid": true,
            "receipt_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }))? + "\n",
    )?;
    let agent_path = dir.join("agent-receipt.json");
    let mut agent_receipt: Value = serde_json::from_str(&fs::read_to_string(
        repo_root().join("fixtures/agent-receipts/positive.json"),
    )?)?;
    agent_receipt["replay_evidence"]["replay_receipt_hash"] =
        replay_verify["replay_receipt_hash"].clone();
    agent_receipt["replay_evidence"]["replay_policy_hash"] =
        replay_verify["replay_policy_hash"].clone();
    agent_receipt["replay_evidence"]["replay_environment_attestation_hash"] =
        replay_verify["environment_attestation_hash"].clone();
    agent_receipt["receipt_hash"] = Value::Null;
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
        "replay_verification_path": verify_path,
        "agent_receipt_hash": agent_validation["agent_receipt_hash"],
        "replay_receipt_hash": replay_verify["replay_receipt_hash"],
        "replay_verification_receipt_hash": replay_verify["receipt_hash"],
    });
    fs::write(&bundle_path, serde_json::to_string_pretty(&bundle)? + "\n")?;

    let verified = run_json_strings(&[
        "agent-receipt".into(),
        "verify-bundle".into(),
        bundle_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(verified["valid"], true);
    assert_eq!(verified["verdict"], "pass");
    assert_eq!(verified["required_replay_group"], "agent");
    assert_eq!(verified["issues"].as_array().unwrap().len(), 0);
    assert_eq!(
        verified["agent_receipt_hash"],
        agent_validation["agent_receipt_hash"]
    );
    assert_eq!(
        verified["replay_receipt_hash"],
        replay_verify["replay_receipt_hash"]
    );
    assert_eq!(
        verified["environment_attestation_hash"],
        replay_verify["environment_attestation_hash"]
    );
    assert_hash_field(&verified, "receipt_hash")?;

    let mut tampered_verify = replay_verify.clone();
    tampered_verify["replay_receipt_hash"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    let tampered_verify_path = dir.join("tampered-verify.json");
    fs::write(
        &tampered_verify_path,
        serde_json::to_string_pretty(&tampered_verify)? + "\n",
    )?;
    let tampered_bundle = json!({
        "agent_receipt_path": agent_path,
        "replay_receipt_path": replay_path,
        "replay_verification_path": tampered_verify_path,
        "release_readiness_path": release_path,
        "release_readiness_receipt_hash": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
    });
    let tampered_bundle_path = dir.join("tampered-bundle.json");
    fs::write(
        &tampered_bundle_path,
        serde_json::to_string_pretty(&tampered_bundle)? + "\n",
    )?;
    let failure = run_json_failure_strings(&[
        "agent-receipt".into(),
        "verify-bundle".into(),
        tampered_bundle_path.display().to_string(),
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
    assert!(issue_codes.contains(&"replay_verification.replay_receipt_hash_mismatch"));
    assert!(issue_codes.contains(&"bundle.expected_hash_mismatch"));
    let failure_text = serde_json::to_string(&failure)?;
    assert!(
        !failure_text.contains(&dir.display().to_string()),
        "{failure_text}"
    );
    assert!(!failure_text.contains("SECRET"), "{failure_text}");
    Ok(())
}
