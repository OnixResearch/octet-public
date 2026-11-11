use std::{error::Error, fs, path::PathBuf, process::Command};

use serde_json::json;
use serde_json::Value;

// r[verify cairn.developer_loop_profiles.cli]
// r[verify cairn.developer_loop_profiles.cli.inspect]
// r[verify cairn.developer_loop_profiles.cli.no_execution]
// r[verify cairn.developer_loop_profiles.receipt]
// r[verify cairn.developer_loop_profiles.receipt.current]
// r[verify cairn.developer_loop_profiles.receipt.stale]
// r[verify cairn.developer_loop_profiles.full_gate]
// r[verify cairn.developer_loop_profiles.full_gate.separate]
// r[verify cairn.developer_loop_profiles.dogfood]
// r[verify cairn.developer_loop_profiles.dogfood.current]
// r[verify cairn.developer_loop_profiles.verification]
// r[verify cairn.developer_loop_profiles.docs]

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy_path() -> PathBuf {
    repo_root().join("cairn-policy/generated/cairn-policy.json")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(repo_root())
        .env_remove("CAIRN_GLOBAL_REGISTRY");
    command
}

fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn local_receipt(profile: &str, policy_hash: &str, subject_hash: &str, duration: u64) -> Value {
    make_receipt(profile, policy_hash, subject_hash, duration)
}

fn make_receipt(profile: &str, policy_hash: &str, subject_hash: &str, duration: u64) -> Value {
    let mut value = json!({
        "schema": "cairn.developer-loop-receipt.v1",
        "profile": profile,
        "policy_hash": policy_hash,
        "subject_hash": subject_hash,
        "target": "x86_64-linux",
        "toolchain": "nix develop",
        "measurement_class": "wall",
        "observed_duration_seconds": duration,
        "result_statuses": [
            {"check_id": "fmt", "status": "passed", "command_identity": "cairn-developer-loop.fmt.check"},
            {"check_id": "validate", "status": "passed", "command_identity": "cairn-developer-loop.validate.check"},
            {"check_id": "traceability", "status": "passed", "command_identity": "cairn-developer-loop.traceability.check"}
        ],
        "cache": "miss",
        "cache_key_identity": "",
        "posture_states": ["declared"],
        "full_gate_status": "outstanding",
        "enforcement_evidence": []
    });
    let hash = blake3::hash(serde_json::to_vec(&value).expect("serialize").as_slice());
    let receipt_hash = hash.to_hex().to_string();
    value["receipt_hash"] = json!(receipt_hash);
    value
}

#[test]
fn lists_plans_and_verifies_current_receipt_without_execution() -> Result<(), Box<dyn Error>> {
    let policy = policy_path().to_string_lossy().to_string();
    let list = run_json(&["developer-loop", "list", "--policy", &policy])?;
    assert_eq!(list["valid"], true);
    assert_eq!(list["profiles"].as_array().map(Vec::len), Some(1));
    assert_eq!(list["profiles"][0]["id"], "cairn-local");

    let plan = run_json(&["developer-loop", "plan", "cairn-local", "--policy", &policy])?;
    assert_eq!(plan["valid"], true);
    assert_eq!(plan["dry_run"], true);
    assert_eq!(plan["plan"]["steps"].as_array().map(Vec::len), Some(3));
    assert!(!plan["plan"]["plan_hash"].as_str().unwrap_or("").is_empty());

    // verify a passing receipt bound to current subject and policy
    let subject_hash = "c".repeat(64);
    let policy_hash = policy_hash_of(&policy)?;
    let receipt = local_receipt("cairn-local", &policy_hash, &subject_hash, 10);
    let receipt_path = repo_root().join("cairn-test-developer-loop-receipt.json");
    fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;
    let receipt_string = "cairn-test-developer-loop-receipt.json".to_string();
    let verify = run_json(&[
        "developer-loop",
        "verify",
        "cairn-local",
        "--receipt",
        &receipt_string,
        "--subject-hash",
        &subject_hash,
        "--policy",
        &policy,
    ])?;
    assert_eq!(verify["valid"], true, "{:#}", verify["issues"]);
    assert_eq!(verify["full_gate_status"], "outstanding");
    fs::remove_file(&receipt_path)?;
    Ok(())
}

#[test]
fn stale_subject_fails_closed_without_execution() -> Result<(), Box<dyn Error>> {
    let policy = policy_path().to_string_lossy().to_string();
    let policy_hash = policy_hash_of(&policy)?;
    let receipt = local_receipt("cairn-local", &policy_hash, &("d".repeat(64)), 10);
    let receipt_path = repo_root().join("cairn-test-developer-loop-receipt.json");
    fs::write(&receipt_path, serde_json::to_vec_pretty(&receipt)?)?;
    let receipt_string = "cairn-test-developer-loop-receipt.json".to_string();
    let verify = run_json(&[
        "developer-loop",
        "verify",
        "cairn-local",
        "--receipt",
        &receipt_string,
        "--subject-hash",
        &("c".repeat(64)),
        "--policy",
        &policy,
    ])?;
    assert_eq!(verify["valid"], false);
    assert!(verify["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["code"] == "developer_loop.subject.stale"));
    fs::remove_file(&receipt_path)?;
    Ok(())
}

fn policy_hash_of(policy_path: &str) -> Result<String, Box<dyn Error>> {
    let value: Value = serde_json::from_slice(&fs::read(policy_path)?)?;
    Ok(blake3::hash(serde_json::to_vec(&value)?.as_slice())
        .to_hex()
        .to_string())
}
