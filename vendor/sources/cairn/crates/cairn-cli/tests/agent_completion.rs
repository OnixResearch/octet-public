use std::{error::Error, fs, path::PathBuf, process::Command};

use serde_json::Value;

const SUBJECT_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(root());
    command
}

fn run_json(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn complete(contract: &str, results: &str) -> Result<Value, Box<dyn Error>> {
    run_json(cairn().args([
        "agent-receipt",
        "complete",
        "--contract",
        contract,
        "--results",
        results,
        "--subject-hash",
        SUBJECT_HASH,
    ]))
}

#[test]
// r[verify cairn.supersede.agent_run_receipts.verification_contract]
fn generated_policy_owns_authoritative_schema_and_statuses() -> Result<(), Box<dyn Error>> {
    let policy: Value = serde_json::from_slice(&fs::read(
        root().join("cairn-policy/generated/cairn-policy.json"),
    )?)?;
    assert_eq!(
        policy["agent_completion_policy"]["schema"],
        "cairn.agent-completion.v2"
    );
    assert_eq!(
        policy["agent_completion_policy"]["passing_status"],
        "passed"
    );
    assert_eq!(
        policy["agent_completion_policy"]["non_passing_statuses"],
        serde_json::json!([
            "failed",
            "blocked",
            "skipped",
            "deferred",
            "timed_out",
            "cancelled",
            "dry_run",
            "cleanup_failed",
            "infrastructure_error",
            "not_selected",
            "unknown"
        ])
    );
    Ok(())
}

#[test]
// r[verify cairn.supersede.agent_run_receipts.authoritative_verification.positive]
// r[verify cairn.supersede.agent_run_receipts.completion_authority.citation]
// r[verify cairn.supersede.agent_run_receipts.execution_boundary.no_execution]
fn cli_authoritative_completion_is_snapshot_bound_and_cross_root_stable(
) -> Result<(), Box<dyn Error>> {
    let first = complete(
        "fixtures/agent-completion/contract.json",
        "fixtures/agent-completion/results.json",
    )?;
    assert_eq!(first["valid"], true, "{first:#}");
    assert_eq!(first["checks_executed"], false);
    assert_eq!(
        first["completion"]["passed_checks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let temporary = std::env::temp_dir().join(format!(
        "cairn-agent-completion-cross-root-{}",
        std::process::id()
    ));
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(&temporary)?;
    let contract = temporary.join("contract.json");
    let results = temporary.join("results.json");
    fs::copy(
        root().join("fixtures/agent-completion/contract.json"),
        &contract,
    )?;
    fs::copy(
        root().join("fixtures/agent-completion/results.json"),
        &results,
    )?;
    let second = complete(
        contract.to_str().ok_or("contract path")?,
        results.to_str().ok_or("results path")?,
    )?;
    assert_eq!(
        first["completion_receipt_hash"],
        second["completion_receipt_hash"]
    );
    assert_eq!(
        first["completion"]["input_hash"],
        second["completion"]["input_hash"]
    );
    fs::remove_dir_all(temporary)?;
    Ok(())
}

#[test]
// r[verify cairn.supersede.agent_run_receipts.schema_migration.legacy]
// r[verify cairn.supersede.agent_run_receipts.verification_contract.self_authored]
fn legacy_success_receipt_is_non_authoritative() -> Result<(), Box<dyn Error>> {
    let receipt = run_json(cairn().args([
        "agent-receipt",
        "validate",
        "fixtures/agent-receipts/positive.json",
    ]))?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "INSPECTED");
    assert_eq!(receipt["classification"], "legacy_unscoped");
    assert_eq!(receipt["authoritative_completion"], false);
    assert!(receipt["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| { issue["code"] == "verification.missing_contract" }));
    Ok(())
}

#[test]
// r[verify cairn.supersede.agent_run_receipts.authoritative_verification.negative]
// r[verify cairn.supersede.agent_run_receipts.freshness.post_check_mutation]
fn stale_subject_fails_without_mutating_saved_evidence() -> Result<(), Box<dyn Error>> {
    let contract_path = root().join("fixtures/agent-completion/contract.json");
    let results_path = root().join("fixtures/agent-completion/results.json");
    let before_contract = fs::read(&contract_path)?;
    let before_results = fs::read(&results_path)?;
    let receipt = run_json(cairn().args([
        "agent-receipt",
        "complete",
        "--contract",
        contract_path.to_str().ok_or("contract path")?,
        "--results",
        results_path.to_str().ok_or("results path")?,
        "--subject-hash",
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
    ]))?;
    assert_eq!(receipt["valid"], false);
    assert!(receipt["completion"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["code"] == "verification_contract.subject_stale"));
    assert_eq!(fs::read(contract_path)?, before_contract);
    assert_eq!(fs::read(results_path)?, before_results);
    Ok(())
}

#[test]
// r[verify cairn.supersede.agent_run_receipts.schema_migration.parity]
fn mcp_and_cli_authoritative_surfaces_agree() -> Result<(), Box<dyn Error>> {
    let cli = complete(
        "fixtures/agent-completion/contract.json",
        "fixtures/agent-completion/results.json",
    )?;
    let contract: Value = serde_json::from_slice(&fs::read(
        root().join("fixtures/agent-completion/contract.json"),
    )?)?;
    let results: Value = serde_json::from_slice(&fs::read(
        root().join("fixtures/agent-completion/results.json"),
    )?)?;
    let input = serde_json::to_string(&serde_json::json!({
        "contract": contract,
        "results": results,
        "subject_hash": SUBJECT_HASH,
    }))?;
    let mcp_call =
        run_json(cairn().args(["mcp", "--call", "cairn_agent_completion_validate", &input]))?;
    let mcp = &mcp_call["structuredContent"];
    assert_eq!(mcp["valid"], cli["valid"], "{mcp:#}");
    assert_eq!(
        mcp["completion_receipt_hash"],
        cli["completion_receipt_hash"]
    );
    assert_eq!(
        mcp["completion"]["diagnostics"],
        cli["completion"]["diagnostics"]
    );
    assert_eq!(mcp["mcp"]["checks_executed"], false);
    Ok(())
}
