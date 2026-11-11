use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
    process::{Command, Output},
    time::SystemTime,
};

use serde_json::Value;

const STATUS_INVALID_ARGUMENTS: i32 = 2;
const STATUS_NOT_FOUND: i32 = 3;
const STATUS_POLICY_ERROR: i32 = 4;
const STATUS_INTERNAL_ERROR: i32 = 70;
const STATUS_NESTED_FAILURE: i32 = 1;
const TEST_SMALL_CAPTURE_BOUND: u64 = 24 * 1_024;
const TEST_REQUIRED_DIAGNOSTIC_COUNT: u64 = 10;
const MACHINE_SCHEMA: &str = "cairn.cli-machine.v1";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn run(args: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(cairn().args(args).output()?)
}

fn envelope(output: &Output) -> Result<Value, Box<dyn Error>> {
    assert!(output.stderr.is_empty(), "controlled stderr must be empty");
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn temp_test_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let dir = env::temp_dir().join(format!(
        "cairn-machine-cli-test-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn machine_success_wraps_the_unchanged_result() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.envelope.success]
    // r[verify cairn.cli_machine_contract.compatibility.nested]
    // r[verify cairn.cli_machine_contract.mode.select]
    // r[verify cairn.cli_machine_contract.privacy.nested]
    let direct = run(&["status", "--root", "fixtures/basic"])?;
    let machine = run(&["--machine", "status", "--root", "fixtures/basic"])?;
    assert!(direct.status.success());
    assert!(machine.status.success());
    let direct_result: Value = serde_json::from_slice(&direct.stdout)?;
    let wrapped = envelope(&machine)?;
    assert_eq!(wrapped["schema_version"], MACHINE_SCHEMA);
    assert_eq!(wrapped["ok"], true);
    assert_eq!(wrapped["command"], "status");
    assert_eq!(wrapped["exit_class"], "success");
    assert_eq!(wrapped["diagnostics"], serde_json::json!([]));
    assert!(!wrapped["non_claims"]
        .as_array()
        .expect("machine non-claims")
        .is_empty());
    assert_eq!(wrapped["result"], direct_result);
    assert_eq!(
        wrapped["result"]["receipt_hash"],
        direct_result["receipt_hash"]
    );
    Ok(())
}

#[test]
fn machine_mode_wraps_workflow_status() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.mode.select]
    // r[verify cairn.cli_machine_contract.compatibility.nested]
    let output = run(&[
        "--machine",
        "workflow",
        "status",
        "demo",
        "--root",
        "fixtures/basic",
    ])?;
    assert!(output.status.success());
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], true);
    assert_eq!(wrapped["command"], "workflow");
    assert_eq!(wrapped["result"]["command"], "workflow status");
    assert_eq!(wrapped["result"]["change"], "demo");
    Ok(())
}

#[test]
fn machine_output_is_byte_repeatable() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.verification]
    // r[verify cairn.cli_machine_contract.verification.suites]
    let args = ["--machine", "status", "--root", "fixtures/basic"];
    let first = run(&args)?;
    let second = run(&args)?;
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(first.stderr.is_empty());
    assert!(second.stderr.is_empty());
    Ok(())
}

#[test]
fn blocked_receipt_remains_a_transport_success() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.exit_status.nested]
    let output = run(&[
        "--machine",
        "sync",
        "missing",
        "--root",
        "fixtures/missing-artifacts",
    ])?;
    assert!(output.status.success());
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], true);
    assert_eq!(wrapped["exit_class"], "success");
    assert_eq!(wrapped["result"]["blocked"], true);
    assert_eq!(wrapped["result"]["dry_run"], true);
    Ok(())
}

#[test]
fn nested_failure_preserves_its_nonzero_status() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.exit_status.failure]
    let output = run(&[
        "--machine",
        "validate",
        "--root",
        "fixtures/missing-artifacts",
        "--strict",
    ])?;
    assert_eq!(output.status.code(), Some(STATUS_NESTED_FAILURE));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], true);
    assert_eq!(wrapped["exit_class"], "success");
    assert_eq!(wrapped["result"]["valid"], false);
    Ok(())
}

#[test]
fn unknown_command_uses_stable_private_failure() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.diagnostics.stable]
    // r[verify cairn.cli_machine_contract.privacy.secret]
    let secret_like_command = "credential-private-token";
    let output = cairn()
        .env("CAIRN_MACHINE_SECRET", "environment-private-token")
        .args(["--machine", secret_like_command])
        .output()?;
    assert_eq!(output.status.code(), Some(STATUS_INVALID_ARGUMENTS));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], false);
    assert_eq!(wrapped["command"], "unknown");
    assert_eq!(wrapped["exit_class"], "invalid_arguments");
    assert_eq!(
        wrapped["diagnostics"][0]["code"],
        "cairn.cli.unknown_command"
    );
    let rendered = String::from_utf8(output.stdout)?;
    assert!(!rendered.contains(secret_like_command));
    assert!(!rendered.contains("environment-private-token"));
    Ok(())
}

#[test]
fn missing_command_is_a_controlled_parser_failure() -> Result<(), Box<dyn Error>> {
    let output = run(&["--machine"])?;
    assert_eq!(output.status.code(), Some(STATUS_INVALID_ARGUMENTS));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], false);
    assert_eq!(wrapped["command"], "unknown");
    assert_eq!(
        wrapped["diagnostics"][0]["code"],
        "cairn.cli.missing_command"
    );
    Ok(())
}

#[test]
fn missing_lookup_uses_not_found_without_raw_identity() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.envelope.failure]
    let missing_identity = "missing-private-identity";
    let output = run(&[
        "--machine",
        "spec",
        "show",
        missing_identity,
        "--root",
        "fixtures/basic",
    ])?;
    assert_eq!(output.status.code(), Some(STATUS_NOT_FOUND));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], false);
    assert_eq!(wrapped["command"], "spec");
    assert_eq!(wrapped["exit_class"], "not_found");
    assert_eq!(wrapped["diagnostics"][0]["code"], "cairn.cli.not_found");
    assert!(!String::from_utf8(output.stdout)?.contains(missing_identity));
    Ok(())
}

#[test]
fn incompatible_presentation_fails_without_mixed_output() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.mode.incompatible]
    let output = run(&[
        "--machine",
        "view",
        "graph",
        "--root",
        "fixtures/basic",
        "--format",
        "mermaid",
    ])?;
    assert_eq!(output.status.code(), Some(STATUS_INVALID_ARGUMENTS));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], false);
    assert_eq!(wrapped["exit_class"], "invalid_arguments");
    assert_eq!(
        wrapped["diagnostics"][0]["code"],
        "cairn.cli.incompatible_option"
    );
    Ok(())
}

#[test]
fn oversized_command_output_becomes_a_bounded_failure() -> Result<(), Box<dyn Error>> {
    let dir = temp_test_dir("bounded-output")?;
    let source = fs::read_to_string(repo_root().join("cairn-policy/generated/cairn-policy.json"))?;
    let mut policy: Value = serde_json::from_str(&source)?;
    policy["machine_contract_policy"]["bounds"]["max_diagnostics"] =
        Value::from(TEST_REQUIRED_DIAGNOSTIC_COUNT);
    policy["machine_contract_policy"]["bounds"]["max_stdout_bytes"] =
        Value::from(TEST_SMALL_CAPTURE_BOUND);
    let policy_path = dir.join("bounded-policy.json");
    fs::write(&policy_path, serde_json::to_vec_pretty(&policy)?)?;
    let output = cairn()
        .args([
            "--machine",
            "policy",
            "--policy",
            policy_path.to_str().ok_or("UTF-8 policy path")?,
        ])
        .output()?;
    assert_eq!(output.status.code(), Some(STATUS_INTERNAL_ERROR));
    let wrapped = envelope(&output)?;
    assert_eq!(wrapped["ok"], false);
    assert_eq!(wrapped["exit_class"], "internal_error");
    assert_eq!(
        wrapped["diagnostics"][0]["code"],
        "cairn.cli.output_too_large"
    );
    assert!(output.stdout.len() < usize::try_from(TEST_SMALL_CAPTURE_BOUND)?);
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn invalid_machine_policy_emits_no_envelope() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.policy.invalid]
    let dir = temp_test_dir("invalid-policy")?;
    let source = fs::read_to_string(repo_root().join("cairn-policy/generated/cairn-policy.json"))?;
    let mut policy: Value = serde_json::from_str(&source)?;
    policy["machine_contract_policy"]["diagnostics"][0]["target"] =
        Value::String("../credential-private-path".to_string());
    let policy_path = dir.join("invalid-policy.json");
    fs::write(&policy_path, serde_json::to_vec_pretty(&policy)?)?;
    let output = cairn()
        .args([
            "--machine",
            "status",
            "--root",
            "fixtures/basic",
            "--policy",
            policy_path.to_str().ok_or("UTF-8 policy path")?,
        ])
        .output()?;
    assert_eq!(output.status.code(), Some(STATUS_POLICY_ERROR));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"error: cairn.cli.policy_error\n");
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn machine_contract_docs_define_agent_and_host_handling() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.docs]
    // r[verify cairn.cli_machine_contract.docs.complete]
    // r[verify cairn.cli_machine_contract.boundary]
    // r[verify cairn.cli_machine_contract.boundary.host]
    // r[verify cairn.cli_machine_contract.boundary.nonclaim]
    let docs = fs::read_to_string(repo_root().join("docs/cli-machine-contract.md"))?;
    assert!(docs.contains("cairn --machine <command> [args]"));
    assert!(docs.contains("Do not parse `message` text."));
    assert!(docs.contains("Treat missing output as a host or pre-boundary failure."));
    assert!(docs.contains("No envelope is guaranteed after host termination"));
    assert!(docs.contains("It does not prove lifecycle readiness or release eligibility."));
    Ok(())
}

#[test]
fn default_cli_and_mcp_do_not_gain_machine_framing() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.cli_machine_contract.compatibility.default]
    let direct = run(&["status", "--root", "fixtures/basic"])?;
    assert!(direct.status.success());
    let direct_value: Value = serde_json::from_slice(&direct.stdout)?;
    assert!(direct_value.get("schema_version").is_none());

    let mcp = run(&["mcp", "--describe"])?;
    assert!(mcp.status.success());
    let mcp_value: Value = serde_json::from_slice(&mcp.stdout)?;
    assert_eq!(mcp_value["protocol_version"], "2024-11-05");
    assert!(mcp_value.get("schema_version").is_none());
    Ok(())
}
