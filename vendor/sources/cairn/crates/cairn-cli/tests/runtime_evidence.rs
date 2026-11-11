use std::{error::Error, process::Command};

#[allow(dead_code)]
#[path = "release_receipts/support.rs"]
mod support;

use support::{
    assert_hash_field, cairn, repo_root, run_json, run_json_failure_command, run_json_strings,
    temp_project,
};

fn run_failure(args: &[&str]) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut command: Command = cairn();
    command.args(args);
    run_json_failure_command(&mut command)
}

fn run_failure_strings(args: &[String]) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut command: Command = cairn();
    command.args(args);
    run_json_failure_command(&mut command)
}

#[test]
fn runtime_evidence_verify_accepts_witnessed_chain() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.runtime_evidence.cli.valid]
    // r[verify cairn.runtime_evidence.tests.coverage]
    // r[verify cairn.runtime_evidence.gates]
    // r[verify cairn.runtime_evidence.gates.focused]
    let receipt = run_json(&[
        "runtime-evidence",
        "verify",
        "--chain",
        "fixtures/runtime-evidence/positive/chain.json",
        "--checkpoint",
        "fixtures/runtime-evidence/positive/checkpoint.json",
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(receipt["subject"], "repo:cairn");
    assert_eq!(receipt["record_count"], 2);
    assert_eq!(receipt["completeness"]["verdict"], "witnessed_prefix");
    assert_eq!(receipt["capture_origin_summary"]["boundary_captured"], 1);
    assert_eq!(receipt["capture_origin_summary"]["ingested_telemetry"], 1);
    assert_hash_field(&receipt, "chain_root_hash")?;
    assert_hash_field(&receipt, "head_hash")?;
    assert_hash_field(&receipt, "checkpoint_hash")?;
    assert_hash_field(&receipt, "receipt_hash")
}

#[test]
fn runtime_evidence_verify_fails_path_safely() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.runtime_evidence.cli.path_safe]
    // r[verify cairn.runtime_evidence.docs]
    // r[verify cairn.runtime_evidence.docs.claims]
    // r[verify cairn.runtime_evidence.docs.compatibility]
    let receipt = run_failure(&[
        "runtime-evidence",
        "verify",
        "--chain",
        "fixtures/runtime-evidence/negative/secret-summary.json",
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "fail");
    let text = serde_json::to_string(&receipt)?;
    assert!(!text.contains("SECRET=do-not-leak"));
    assert!(!text.contains(&repo_root().display().to_string()));
    assert!(receipt["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .any(|issue| issue["code"] == "runtime_evidence.summary_secret_like"));
    Ok(())
}

#[test]
fn runtime_evidence_verify_fails_wrong_checkpoint_subject() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.runtime_evidence.external_witness.subject_mismatch]
    let receipt = run_failure(&[
        "runtime-evidence",
        "verify",
        "--chain",
        "fixtures/runtime-evidence/positive/chain.json",
        "--checkpoint",
        "fixtures/runtime-evidence/negative/wrong-subject-checkpoint.json",
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["completeness"]["verdict"], "subject_mismatch");
    Ok(())
}

#[test]
fn evidence_export_links_runtime_roles() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.runtime_evidence.manifest_roles]
    // r[verify cairn.runtime_evidence.manifest_roles.bundle]
    // r[verify cairn.runtime_evidence.manifest_roles.manifest]
    let out = temp_project("runtime-evidence-export")?.join("export");
    let receipt = run_json_strings(&[
        "evidence".into(),
        "export".into(),
        "--root".into(),
        ".".into(),
        "--out".into(),
        out.display().to_string(),
        "--runtime-chain".into(),
        "fixtures/runtime-evidence/positive/chain.json".into(),
        "--witness-checkpoint".into(),
        "fixtures/runtime-evidence/positive/checkpoint.json".into(),
    ])?;
    assert_eq!(receipt["valid"], true);
    let roles = receipt["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .map(|artifact| artifact["role"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(roles.contains(&"runtime_evidence"));
    assert!(roles.contains(&"runtime_chain"));
    assert!(roles.contains(&"witness_checkpoint"));
    Ok(())
}

#[test]
fn invalid_runtime_export_does_not_weaken_existing_validators() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.runtime_evidence.manifest_roles.invalid]
    // r[verify cairn.runtime_evidence.archive]
    // r[verify cairn.runtime_evidence.archive.ready]
    let out = temp_project("runtime-evidence-invalid-export")?.join("export");
    let receipt = run_failure_strings(&[
        "evidence".into(),
        "export".into(),
        "--root".into(),
        ".".into(),
        "--out".into(),
        out.display().to_string(),
        "--runtime-chain".into(),
        "fixtures/runtime-evidence/negative/secret-summary.json".into(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert!(receipt["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .any(|issue| issue["code"] == "export.bundle_verification_failed"
            || issue["code"] == "export.manifest_verification_failed"));
    Ok(())
}
