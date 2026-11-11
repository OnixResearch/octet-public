// r[impl cairn.structured_lifecycle.probes]
// r[verify cairn.structured_lifecycle.probes.declared_oracle]
// r[verify cairn.structured_lifecycle.probes.execution_receipt]
// r[verify cairn.structured_lifecycle.probes.positive_negative]
// r[verify cairn.structured_lifecycle.probes.data_predicates_first]
// r[impl cairn.probe_execution_boundary.default_gate_no_execution]
// r[verify cairn.probe_execution_boundary.default_gate_no_execution.existing_data_only]
// r[verify cairn.probe_execution_boundary.default_gate_no_execution.missing_evidence]
// r[impl cairn.probe_execution_boundary.explicit_probe_run]
// r[verify cairn.probe_execution_boundary.explicit_probe_run.authorized_local_command]
// r[verify cairn.probe_execution_boundary.explicit_probe_run.denied_command]
// r[impl cairn.probe_execution_boundary.gates_verify_receipts]
// r[verify cairn.probe_execution_boundary.gates_verify_receipts.valid_saved_receipt]
// r[verify cairn.probe_execution_boundary.gates_verify_receipts.command_mismatch]
// r[verify cairn.probe_execution_boundary.gates_verify_receipts.coverage_class]
// r[impl cairn.probe_execution_boundary.inline_probe_mode]
// r[verify cairn.probe_execution_boundary.inline_probe_mode.disabled_default]
// r[verify cairn.probe_execution_boundary.inline_probe_mode.receipt_links]
// r[impl cairn.probe_execution_boundary.fail_closed]
// r[verify cairn.probe_execution_boundary.fail_closed.tampered_receipt]
// r[verify cairn.probe_execution_boundary.fail_closed.stale_hash]
// r[verify cairn.probe_execution_boundary.fail_closed.insufficient_coverage]
use std::{
    error::Error,
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

use cairn_core::{
    all_probe_assertions_passed, blake3_hex, canonical_json_hash, changes_dir,
    evaluate_probe_assertions, layout_name, probe_declaration_hash, probe_issue_json,
    probe_receipt_hash, validate_probe_completion_status, validate_probe_declaration,
    validate_probe_receipt, GateIssue, LifecycleLayout, Policy, PROBE_RECEIPT_SCHEMA_V1,
};
use serde_json::{json, Value};

use crate::support::{
    default_policy_path, load_policy_file, path_string, print_json, read_option,
    read_string_option, safe_relative_path,
};

const PROBE_DECLARATIONS_PATH: &str = "probes/probes.json";
const SUCCESS_STATUS_CLASS: &str = "success";
const FAILURE_STATUS_CLASS: &str = "failure";
const MAX_SAVED_PROBE_BYTES: usize = 64 * 1024;
const DEFAULT_EXPECTED_STATUS_CLASS: &str = SUCCESS_STATUS_CLASS;
const CAIRN_LOCAL_PROFILE: &str = "cairn-local";
const READ_ONLY_COMMANDS: &[&str] = &[
    "doctor",
    "gate",
    "policy",
    "test-harness",
    "traceability",
    "tracey",
    "validate",
];
const PLAN_COMMANDS: &[&str] = &["archive", "sync"];
const MUTATION_FLAG: &str = "--execute";

pub(crate) fn run_probe(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("probe requires a command: run or validate".into());
    };
    match command {
        "run" => run_probe_run(args.into_iter().skip(1).collect()),
        "validate" => run_probe_validate(args.into_iter().skip(1).collect()),
        unknown => Err(format!("unknown probe command: {unknown}").into()),
    }
}

fn run_probe_run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.probe_execution_boundary.explicit_probe_run]
    // r[impl cairn.probe_execution_boundary.explicit_probe_run.authorized_local_command]
    // r[impl cairn.probe_execution_boundary.explicit_probe_run.denied_command]
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path.clone())?;
    let probe_file =
        read_string_option(&args, "--probe-file").ok_or("probe run requires --probe-file PATH")?;
    let declaration_path = repo_relative_path(&root, &probe_file)
        .ok_or("probe run requires repository-relative --probe-file PATH")?;
    let declaration = read_json_file(&declaration_path)?;
    let mut issues = validate_probe_declaration(&declaration)
        .into_iter()
        .map(|issue| probe_issue_json(&issue))
        .collect::<Vec<_>>();
    let denied = probe_command_denial(&declaration);
    if let Some(denial) = denied {
        issues.push(denial);
        let receipt = denied_probe_receipt(&root, &policy, &declaration, issues);
        return finish_probe_receipt(&root, &args, receipt, false);
    }
    if !issues.is_empty() {
        let receipt = denied_probe_receipt(&root, &policy, &declaration, issues);
        return finish_probe_receipt(&root, &args, receipt, false);
    }
    let argv = command_argv_with_policy(&declaration, &policy_path)?;
    let output = Command::new(std::env::current_exe()?)
        .args(&argv)
        .current_dir(&root)
        .output()?;
    let stdout_json = serde_json::from_slice::<Value>(&output.stdout).unwrap_or(Value::Null);
    let assertion_values = declaration["assertions"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut assertion_results =
        evaluate_probe_assertions(&stdout_json, &output_json_assertions(&assertion_values));
    assertion_results.extend(evaluate_file_assertions(&root, &assertion_values));
    let actual_status = status_class(output.status.success());
    let expected_status = expected_status_class(&declaration);
    let status_matches = actual_status == expected_status;
    let assertions_passed = all_probe_assertions_passed(&assertion_results);
    let mut receipt = json!({
        "schema": PROBE_RECEIPT_SCHEMA_V1,
        "command": "probe run",
        "probe_id": declaration["id"].clone(),
        "requirements": declaration["requirements"].clone(),
        "coverage_class": declaration["coverage_class"].clone(),
        "command_identity": declaration["command"]["identity"].clone(),
        "argv": argv,
        "fixtures": fixture_hashes(&root, &declaration),
        "status_class": actual_status,
        "expected_status_class": expected_status,
        "status_matches": status_matches,
        "assertions": assertion_results,
        "stdout_blake3": blake3_hex(&output.stdout),
        "stderr_blake3": blake3_hex(&output.stderr),
        "stdout_bytes": output.stdout.len(),
        "stderr_bytes": output.stderr.len(),
        "root": ".",
        "probe_file": safe_relative_path(&root, &declaration_path),
        "lifecycle_bundle_hash": probe_declaration_hash(&declaration),
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "issues": [],
        "valid": status_matches && assertions_passed,
        "verdict": if status_matches && assertions_passed {"PASS"} else {"FAIL"},
    });
    let mut receipt_issues = Vec::new();
    if !status_matches {
        receipt_issues.push(json!({
            "code": "probe.status_class.mismatch",
            "message": "probe command exit status class did not match expected_status_class",
            "field_path": "status_class",
        }));
    }
    if !assertions_passed {
        receipt_issues.push(json!({
            "code": "probe.assertions.failed",
            "message": "one or more probe assertions failed",
            "field_path": "assertions",
        }));
    }
    receipt["issues"] = Value::Array(receipt_issues);
    finish_probe_receipt(
        &root,
        &args,
        with_probe_receipt_hash(receipt),
        status_matches && assertions_passed,
    )
}

fn run_probe_validate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let probe_file = read_string_option(&args, "--probe-file")
        .ok_or("probe validate requires --probe-file PATH")?;
    let receipt_file =
        read_string_option(&args, "--receipt").ok_or("probe validate requires --receipt PATH")?;
    let declaration = read_json_file(
        &repo_relative_path(&root, &probe_file)
            .ok_or("probe validate requires repository-relative --probe-file PATH")?,
    )?;
    let receipt = read_json_file(
        &repo_relative_path(&root, &receipt_file)
            .ok_or("probe validate requires repository-relative --receipt PATH")?,
    )?;
    let issues = validate_probe_receipt(
        &declaration,
        &receipt,
        &probe_declaration_hash(&declaration),
        Some(&canonical_json_hash(&policy.raw)),
    )
    .into_iter()
    .map(|issue| probe_issue_json(&issue))
    .collect::<Vec<_>>();
    let valid = issues.is_empty();
    print_json(with_probe_receipt_hash(json!({
        "schema": "cairn.probe-validation-receipt.v1",
        "command": "probe validate",
        "root": ".",
        "probe_file": probe_file,
        "receipt": receipt_file,
        "issues": issues,
        "valid": valid,
        "verdict": if valid {"PASS"} else {"FAIL"},
    })))
}

#[derive(Debug, Clone)]
pub(crate) struct ProbeGateEvidence {
    pub issues: Vec<GateIssue>,
    pub binding: Value,
}

pub(crate) fn observe_gate_probes(
    root: &Path,
    layout: LifecycleLayout,
    change: &str,
    policy: &Policy,
) -> ProbeGateEvidence {
    // r[impl cairn.probe_execution_boundary.authoritative_evidence_binding]
    // r[impl cairn.probe_execution_boundary.authoritative_evidence_binding.changed]
    // r[impl cairn.probe_execution_boundary.authoritative_evidence_binding.private]
    // r[impl cairn.probe_execution_boundary.authoritative_status]
    // r[impl cairn.probe_execution_boundary.authoritative_status.passed]
    // r[impl cairn.probe_execution_boundary.authoritative_status.non_pass]
    // r[impl cairn.probe_execution_boundary.authoritative_status.unknown]
    // r[impl cairn.probe_execution_boundary.authoritative_diagnostics]
    // r[impl cairn.probe_execution_boundary.authoritative_diagnostics.actionable]
    // r[impl cairn.probe_execution_boundary.authoritative_diagnostics.safe]
    // r[impl cairn.probe_execution_boundary.authoritative_boundary]
    // r[impl cairn.probe_execution_boundary.authoritative_boundary.no_execution]
    // r[impl cairn.probe_execution_boundary.authoritative_boundary.pure_core]
    let declarations_path = root
        .join(changes_dir(layout))
        .join(change)
        .join(PROBE_DECLARATIONS_PATH);
    if !declarations_path.exists() {
        return ProbeGateEvidence {
            issues: Vec::new(),
            binding: json!({"declarations_present": false, "probes": []}),
        };
    }
    let declaration_bytes = match read_bounded_probe_bytes(&declarations_path) {
        Ok(bytes) => bytes,
        Err(message) => {
            return probe_observation_failure(
                policy,
                "probe.declarations.unreadable",
                message,
                json!({"declarations_present": true, "probes": []}),
            );
        }
    };
    let declarations: Value = match serde_json::from_slice(&declaration_bytes) {
        Ok(value) => value,
        Err(_) => {
            return probe_observation_failure(
                policy,
                "probe.declarations.malformed",
                "saved probe declarations are not valid JSON".to_string(),
                json!({
                    "declarations_present": true,
                    "declarations_hash": blake3_hex(&declaration_bytes),
                    "probes": [],
                }),
            );
        }
    };
    let probes = probes_array(&declarations);
    let mut messages = Vec::with_capacity(probes.len());
    let mut bindings = Vec::with_capacity(probes.len());
    for probe in probes {
        let probe_messages =
            gate_probe_declaration_issues(root, &declarations_path, &probe, policy).unwrap_or_else(
                |_| {
                    vec![format!(
                        "probe {}: saved receipt is unreadable or malformed",
                        display_probe_id(&probe)
                    )]
                },
            );
        messages.extend(probe_messages);
        bindings.push(probe_evidence_binding(root, &probe));
    }
    ProbeGateEvidence {
        issues: messages
            .into_iter()
            .map(|message| probe_gate_issue(message, policy))
            .collect(),
        binding: json!({
            "declarations_present": true,
            "declarations_path": safe_relative_path(root, &declarations_path),
            "declarations_hash": blake3_hex(&declaration_bytes),
            "probes": bindings,
            "non_claim": "saved probe evidence is bounded to the declared command, assertions, policy, and receipt identity",
        }),
    }
}

fn read_bounded_probe_bytes(path: &Path) -> Result<Vec<u8>, String> {
    let metadata =
        fs::metadata(path).map_err(|_| "saved probe evidence is unreadable".to_string())?;
    if metadata.len() > MAX_SAVED_PROBE_BYTES as u64 {
        return Err(format!(
            "saved probe evidence exceeds the {MAX_SAVED_PROBE_BYTES}-byte limit"
        ));
    }
    fs::read(path).map_err(|_| "saved probe evidence is unreadable".to_string())
}

fn probe_observation_failure(
    policy: &Policy,
    code: &str,
    message: String,
    binding: Value,
) -> ProbeGateEvidence {
    ProbeGateEvidence {
        issues: vec![GateIssue {
            code: code.to_string(),
            message,
            scope: policy.gate_finding_scope.clone(),
            promoted: policy.gate_finding_promoted.clone(),
        }],
        binding,
    }
}

fn probe_gate_issue(message: String, policy: &Policy) -> GateIssue {
    let code = message
        .strip_prefix("probe ")
        .and_then(|rest| rest.split(':').next())
        .filter(|candidate| candidate.contains('.'))
        .map_or_else(
            || "probe.saved_evidence.invalid".to_string(),
            |candidate| candidate.to_string(),
        );
    GateIssue {
        code,
        message,
        scope: policy.gate_finding_scope.clone(),
        promoted: policy.gate_finding_promoted.clone(),
    }
}

fn probe_evidence_binding(root: &Path, declaration: &Value) -> Value {
    let probe_id = display_probe_id(declaration);
    let declaration_hash = probe_declaration_hash(declaration);
    let receipt_path = declaration
        .get("receipt_path")
        .and_then(Value::as_str)
        .filter(|path| repo_relative_path(root, path).is_some());
    let receipt_bytes = receipt_path
        .and_then(|path| repo_relative_path(root, path))
        .and_then(|path| read_bounded_probe_bytes(&path).ok());
    let receipt_hash = receipt_bytes.as_ref().map(|bytes| blake3_hex(bytes));
    let status_class = receipt_bytes
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<Value>(bytes).ok())
        .and_then(|receipt| receipt.get("status_class").cloned());
    let reproduction_argv = declaration
        .get("command")
        .and_then(|command| command.get("argv"))
        .cloned()
        .unwrap_or(Value::Array(Vec::new()));
    json!({
        "probe_id": probe_id,
        "declaration_hash": declaration_hash,
        "receipt_path": receipt_path,
        "receipt_hash": receipt_hash,
        "status_class": status_class,
        "reproduction_argv": reproduction_argv,
        "remediation": "run the declared probe explicitly and save a current passing receipt",
        "severity": "blocking",
    })
}

fn gate_probe_declaration_issues(
    root: &Path,
    declarations_path: &Path,
    declaration: &Value,
    policy: &Policy,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut messages = validate_probe_declaration(declaration)
        .into_iter()
        .map(|issue| format!("probe {}: {}", issue.code, issue.message))
        .collect::<Vec<_>>();
    let Some(receipt_path) = declaration.get("receipt_path").and_then(Value::as_str) else {
        messages.push(format!(
            "probe {}: missing saved probe receipt path",
            display_probe_id(declaration)
        ));
        return Ok(messages);
    };
    let Some(receipt_full_path) = repo_relative_path(root, receipt_path) else {
        messages.push(format!(
            "probe {}: receipt_path must be repository-relative",
            display_probe_id(declaration)
        ));
        return Ok(messages);
    };
    if !receipt_full_path.exists() {
        messages.push(format!(
            "probe {}: saved probe receipt missing at {receipt_path}",
            display_probe_id(declaration)
        ));
        return Ok(messages);
    }
    let receipt = read_json_file(&receipt_full_path)?;
    let receipt_issues = validate_probe_receipt(
        declaration,
        &receipt,
        &probe_declaration_hash(declaration),
        Some(&canonical_json_hash(&policy.raw)),
    );
    messages.extend(
        receipt_issues
            .into_iter()
            .chain(validate_probe_completion_status(&receipt))
            .map(|issue| {
                format!(
                    "probe {} in {}: {}",
                    issue.code,
                    path_string(declarations_path),
                    issue.message
                )
            }),
    );
    Ok(messages)
}

fn denied_probe_receipt(
    root: &Path,
    policy: &Policy,
    declaration: &Value,
    issues: Vec<Value>,
) -> Value {
    with_probe_receipt_hash(json!({
        "schema": PROBE_RECEIPT_SCHEMA_V1,
        "command": "probe run",
        "probe_id": declaration.get("id").cloned().unwrap_or(Value::Null),
        "requirements": declaration.get("requirements").cloned().unwrap_or(Value::Array(Vec::new())),
        "coverage_class": declaration.get("coverage_class").cloned().unwrap_or(Value::Null),
        "command_identity": declaration.get("command").and_then(|value| value.get("identity")).cloned().unwrap_or(Value::Null),
        "argv": declaration.get("command").and_then(|value| value.get("argv")).cloned().unwrap_or(Value::Array(Vec::new())),
        "status_class": "denied",
        "expected_status_class": expected_status_class(declaration),
        "status_matches": false,
        "assertions": [],
        "stdout_blake3": blake3_hex(&[]),
        "stderr_blake3": blake3_hex(&[]),
        "stdout_bytes": 0,
        "stderr_bytes": 0,
        "root": ".",
        "probe_file": ".",
        "lifecycle_bundle_hash": probe_declaration_hash(declaration),
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "issues": issues,
        "valid": false,
        "verdict": "FAIL",
        "layout": layout_name(crate::support::detect_layout(root).layout),
    }))
}

fn finish_probe_receipt(
    root: &Path,
    args: &[String],
    receipt: Value,
    successful: bool,
) -> Result<(), Box<dyn Error>> {
    if let Some(out) = read_string_option(args, "--out") {
        let out_path = repo_relative_path(root, &out)
            .ok_or("probe run requires repository-relative --out PATH")?;
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, serde_json::to_string_pretty(&receipt)? + "\n")?;
    }
    print_json(receipt)?;
    if successful {
        Ok(())
    } else {
        Err("probe run failed".into())
    }
}

fn probe_command_denial(declaration: &Value) -> Option<Value> {
    let command = declaration.get("command")?;
    let profile = command
        .get("profile")
        .and_then(Value::as_str)
        .unwrap_or(CAIRN_LOCAL_PROFILE);
    if profile != CAIRN_LOCAL_PROFILE {
        return Some(denial(
            "probe.command.profile_denied",
            "unsupported command profile",
        ));
    }
    let argv = command.get("argv")?.as_array()?;
    let first = argv.first().and_then(Value::as_str).unwrap_or_default();
    let requests_mutation = argv
        .iter()
        .any(|value| value.as_str() == Some(MUTATION_FLAG));
    let read_only_allowed = READ_ONLY_COMMANDS.iter().any(|allowed| allowed == &first);
    let plan_allowed = PLAN_COMMANDS.iter().any(|allowed| allowed == &first) && !requests_mutation;
    let mutation_allowed = PLAN_COMMANDS.iter().any(|allowed| allowed == &first)
        && requests_mutation
        && declaration.get("allow_mutation").and_then(Value::as_bool) == Some(true);
    if read_only_allowed || plan_allowed || mutation_allowed {
        None
    } else {
        Some(denial(
            "probe.command.capability_denied",
            "probe command is not authorized by the local command profile",
        ))
    }
}

fn denial(code: &str, message: &str) -> Value {
    json!({"code": code, "message": message, "field_path": "command"})
}

fn command_argv_with_policy(
    declaration: &Value,
    policy_path: &Path,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut argv = declaration["command"]["argv"]
        .as_array()
        .ok_or("command.argv must be an array")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| "command.argv entries must be strings".into())
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    if should_append_policy_arg(&argv) {
        argv.push("--policy".to_string());
        argv.push(path_string(
            policy_path
                .canonicalize()
                .unwrap_or_else(|_| policy_path.to_path_buf()),
        ));
    }
    Ok(argv)
}

fn should_append_policy_arg(argv: &[String]) -> bool {
    let Some(command) = argv.first().map(String::as_str) else {
        return false;
    };
    let accepts_policy = matches!(command, "archive" | "doctor" | "gate" | "sync" | "validate");
    accepts_policy && !argv.iter().any(|argument| argument == "--policy")
}

fn output_json_assertions(assertions: &[Value]) -> Vec<Value> {
    assertions
        .iter()
        .filter(|assertion| assertion.get("file_blake3_equals").is_none())
        .cloned()
        .collect()
}

fn evaluate_file_assertions(root: &Path, assertions: &[Value]) -> Vec<Value> {
    assertions
        .iter()
        .filter(|assertion| assertion.get("file_blake3_equals").is_some())
        .map(|assertion| evaluate_file_assertion(root, assertion))
        .collect()
}

fn evaluate_file_assertion(root: &Path, assertion: &Value) -> Value {
    let id = assertion
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unnamed")
        .to_string();
    let file = assertion
        .get("file")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let expected = assertion
        .get("file_blake3_equals")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let actual = file_hash(root, &file);
    let passed = actual.as_deref() == Some(expected.as_str());
    json!({
        "id": id,
        "file": file,
        "operator": "file_blake3_equals",
        "expected": expected,
        "actual": actual,
        "passed": passed,
    })
}

fn fixture_hashes(root: &Path, declaration: &Value) -> Value {
    let fixtures = declaration
        .get("fixtures")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Value::Array(
        fixtures
            .iter()
            .filter_map(Value::as_str)
            .map(|path| json!({"path": path, "blake3": file_hash(root, path)}))
            .collect(),
    )
}

fn file_hash(root: &Path, path: &str) -> Option<String> {
    fs::read(repo_relative_path(root, path)?)
        .ok()
        .map(|bytes| blake3_hex(&bytes))
}

fn repo_relative_path(root: &Path, path: &str) -> Option<PathBuf> {
    let candidate = Path::new(path);
    if path.trim().is_empty() || candidate.is_absolute() {
        return None;
    }
    if candidate.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(root.join(candidate))
}

fn expected_status_class(declaration: &Value) -> String {
    declaration
        .get("expected_status_class")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_EXPECTED_STATUS_CLASS)
        .to_string()
}

fn status_class(success: bool) -> &'static str {
    if success {
        SUCCESS_STATUS_CLASS
    } else {
        FAILURE_STATUS_CLASS
    }
}

fn with_probe_receipt_hash(mut value: Value) -> Value {
    let receipt_hash = probe_receipt_hash(&value);
    value["receipt_hash"] = Value::String(receipt_hash);
    value
}

fn read_json_file(path: &Path) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn probes_array(value: &Value) -> Vec<Value> {
    if let Some(items) = value.get("probes").and_then(Value::as_array) {
        return items.clone();
    }
    if let Some(items) = value.as_array() {
        return items.clone();
    }
    Vec::new()
}

fn display_probe_id(declaration: &Value) -> String {
    declaration
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(argv: Vec<&str>) -> Value {
        json!({
            "schema": "cairn.probe.v1",
            "id": "demo",
            "requirements": ["req.demo"],
            "coverage_class": "positive",
            "command": {"identity": "demo", "argv": argv},
            "assertions": [{"id": "valid", "path": "valid", "equals": true}]
        })
    }

    #[test]
    fn read_only_command_is_allowed() {
        assert!(probe_command_denial(&declaration(vec!["validate", "--root", "."])).is_none());
    }

    #[test]
    fn execute_flag_is_denied_without_mutation_capability() {
        assert!(probe_command_denial(&declaration(vec!["archive", "demo", "--execute"])).is_some());
    }

    #[test]
    fn unknown_command_is_denied() {
        assert!(probe_command_denial(&declaration(vec!["unknown"])).is_some());
    }

    #[test]
    fn repository_relative_path_accepts_safe_child_path() {
        assert_eq!(
            repo_relative_path(Path::new("repo"), "receipts/probe.json"),
            Some(PathBuf::from("repo/receipts/probe.json"))
        );
    }

    #[test]
    fn repository_relative_path_rejects_escape_paths() {
        assert!(repo_relative_path(Path::new("repo"), "../secret").is_none());
        assert!(repo_relative_path(Path::new("repo"), "/tmp/secret").is_none());
        assert!(repo_relative_path(Path::new("repo"), "").is_none());
    }
}
