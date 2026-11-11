use std::{error::Error, fs};

use cairn_core::{
    canonical_json_hash, positive_release_run_ledger_fixture,
    positive_state_machine_receipt_fixture, release_run_identity, release_run_ledger_hash,
    state_machine_receipt_hash, ReleaseRunLedger,
};
use serde_json::Value;

#[allow(dead_code)]
#[path = "release_receipts/support.rs"]
mod support;

use support::{cairn, policy_json, run_json_command, run_json_failure_command, temp_project};

const BLAKE3_HEX_LENGTH: usize = 64;

#[test]
fn state_machine_receipt_cli_accepts_bounded_witness() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_orchestration.state_machine_receipts.cli]
    let temp = temp_project("state-machine-receipt-positive")?;
    let input = temp.join("receipt.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&positive_state_machine_receipt_fixture())?,
    )?;
    let mut command = cairn();
    command.args([
        "state-machine-receipt",
        "validate",
        "--input",
        &input.display().to_string(),
    ]);
    let receipt = run_json_command(&mut command)?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert!(receipt["boundary"]
        .as_str()
        .is_some_and(|text| text.contains("bounded witnesses")));
    Ok(())
}

#[test]
fn state_machine_receipt_cli_rejects_unknown_checker() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_orchestration.state_machine_receipts.fixtures.negative]
    let temp = temp_project("state-machine-receipt-negative")?;
    let input = temp.join("receipt.json");
    let mut value = positive_state_machine_receipt_fixture();
    value["checker"]["id"] = Value::String("unknown".to_string());
    value["receipt_hash"] = Value::String(state_machine_receipt_hash(&value));
    fs::write(&input, serde_json::to_vec_pretty(&value)?)?;
    let mut command = cairn();
    command.args([
        "state-machine-receipt",
        "validate",
        "--input",
        &input.display().to_string(),
    ]);
    let receipt = run_json_failure_command(&mut command)?;
    assert_eq!(receipt["valid"], false);
    assert!(receipt["issues"].as_array().is_some_and(|issues| issues
        .iter()
        .any(|issue| issue["code"] == "state_machine_receipt.checker_unknown")));
    Ok(())
}

#[test]
fn release_run_status_reports_complete_local_ledger() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.release_evidence.durable_run_ledger.shell]
    let policy_hash = canonical_json_hash(&policy_json()?);
    let temp = temp_project("release-run-positive")?;
    let input = temp.join("ledger.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&positive_release_run_ledger_fixture(&policy_hash))?,
    )?;
    let mut command = cairn();
    command.args([
        "release-run",
        "status",
        "--input",
        &input.display().to_string(),
    ]);
    let receipt = run_json_command(&mut command)?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["projection"]["terminal_verdict"], "complete");
    assert_eq!(receipt["projection"]["disposition"], "complete");
    Ok(())
}

#[test]
fn release_run_validate_rejects_stale_policy() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.release_evidence.durable_run_ledger.fixtures.negative]
    let current_policy_hash = canonical_json_hash(&policy_json()?);
    let stale_policy_hash = "e".repeat(BLAKE3_HEX_LENGTH);
    assert_ne!(current_policy_hash, stale_policy_hash);
    let temp = temp_project("release-run-negative")?;
    let input = temp.join("ledger.json");
    let mut value = positive_release_run_ledger_fixture(&stale_policy_hash);
    let mut ledger: ReleaseRunLedger = serde_json::from_value(value.clone())?;
    ledger.run_id = release_run_identity(&ledger);
    for artifact in &mut ledger.artifacts {
        artifact.run_id.clone_from(&ledger.run_id);
    }
    value = serde_json::to_value(ledger)?;
    value["ledger_hash"] = Value::String(release_run_ledger_hash(&value));
    fs::write(&input, serde_json::to_vec_pretty(&value)?)?;
    let mut command = cairn();
    command.args([
        "release-run",
        "validate",
        "--input",
        &input.display().to_string(),
    ]);
    let receipt = run_json_failure_command(&mut command)?;
    assert_eq!(receipt["valid"], false);
    assert!(receipt["projection"]["issues"]
        .as_array()
        .is_some_and(|issues| issues
            .iter()
            .any(|issue| issue["code"] == "release_run_ledger.policy_stale")));
    Ok(())
}
