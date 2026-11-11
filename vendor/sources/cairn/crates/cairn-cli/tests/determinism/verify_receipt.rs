use std::{error::Error, fs};

use serde_json::{json, Value};

use super::support::{
    assert_hash_field, cairn, run_json_failure_strings, run_json_strings, temp_project,
};

#[test]
fn determinism_verify_receipt_validates_saved_replay_evidence() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.replay_receipt_verification.contract]
    // r[verify cairn.supersede.replay_receipt_verification.cli]
    // r[verify cairn.supersede.replay_receipt_verification.validation]
    // r[verify cairn.supersede.replay_receipt_verification.receipt]
    // r[verify cairn.supersede.replay_receipt_verification.failures]
    // r[verify cairn.supersede.replay_receipt_verification.secret_safe]
    let artifact = temp_project("verify-replay-receipt")?.join("receipt.json");
    let mut replay = cairn();
    replay.args([
        "determinism",
        "replay",
        "--root",
        ".",
        "--group",
        "smoke",
        "--format",
        "summary",
        "--out",
        artifact.to_str().unwrap(),
    ]);
    let output = replay.output()?;
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let verified = run_json_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
        artifact.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    let replay_receipt: Value = serde_json::from_slice(&fs::read(&artifact)?)?;
    assert_eq!(verified["valid"], true);
    assert_eq!(verified["verdict"], "pass");
    assert_eq!(verified["issues"].as_array().unwrap().len(), 0);
    assert_eq!(
        verified["replay_receipt_hash"],
        replay_receipt["receipt_hash"]
    );
    assert_eq!(
        verified["replay_policy_hash"],
        replay_receipt["environment_attestation"]["generated_policy_hash"]
    );
    assert_eq!(verified["selected_cases"], json!(["status", "validate"]));
    assert_eq!(verified["selected_groups"], json!(["smoke"]));
    assert_eq!(
        verified["environment_attestation_hash"],
        replay_receipt["environment_attestation"]["attestation_hash"]
    );
    assert_hash_field(&verified, "receipt_hash")?;

    let mut tampered = replay_receipt.clone();
    tampered["selected_cases"] = json!(["status", "unknown_case"]);
    let tampered_path = artifact.with_file_name("tampered.json");
    fs::write(
        &tampered_path,
        serde_json::to_string_pretty(&tampered)? + "\n",
    )?;
    let failure = run_json_failure_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
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
    assert!(issue_codes.contains(&"selected_cases.invalid"));
    assert!(issue_codes.contains(&"receipt_hash.mismatch"));
    assert!(!serde_json::to_string(&failure)?.contains("SECRET"));

    let malformed_path = artifact.with_file_name("malformed.json");
    fs::write(&malformed_path, "{not json\n")?;
    let malformed = run_json_failure_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
        malformed_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(malformed["valid"], false);
    assert_eq!(malformed["issues"][0]["code"], "receipt.malformed_json");
    Ok(())
}
