use std::{error::Error, fs};

use serde_json::{json, Value};

use super::support::{assert_hash_field, cairn, run_json, run_json_strings, temp_fixture};

fn run_json_failure(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn valid_probe_declaration(receipt_path: Option<&str>) -> Value {
    let mut declaration = json!({
        "schema": "cairn.probe.v1",
        "id": "validate-basic",
        "requirements": ["demo.behavior"],
        "coverage_class": "positive",
        "command": {
            "profile": "cairn-local",
            "identity": "cairn validate",
            "argv": ["validate", "--root", "."]
        },
        "expected_status_class": "success",
        "assertions": [{"id": "valid", "path": "valid", "equals": true}]
    });
    if let Some(path) = receipt_path {
        declaration["receipt_path"] = Value::String(path.to_string());
    }
    declaration
}

#[test]
fn probe_run_emits_receipt_and_gate_verifies_saved_receipt() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.probe_execution_boundary.explicit_probe_run]
    // r[verify cairn.probe_execution_boundary.explicit_probe_run.authorized_local_command]
    // r[verify cairn.probe_execution_boundary.gates_verify_receipts]
    // r[verify cairn.probe_execution_boundary.gates_verify_receipts.valid_saved_receipt]
    // r[verify cairn.probe_execution_boundary.authoritative_tests.positive]
    // r[verify cairn.probe_execution_boundary.gates_verify_receipts.coverage_class]
    let fixture = temp_fixture("probe-run-gate-valid", "fixtures/basic")?;
    let fixture_file = ".cairn/specs/demo/spec.md";
    let fixture_hash = cairn_core::blake3_hex(&fs::read(fixture.join(fixture_file))?);
    let mut probe = valid_probe_declaration(None);
    probe["fixtures"] = json!([fixture_file]);
    probe["assertions"]
        .as_array_mut()
        .ok_or("assertions")?
        .push(json!({
            "id": "fixture-hash",
            "file": fixture_file,
            "file_blake3_equals": fixture_hash,
        }));
    fs::write(
        fixture.join("probe.json"),
        serde_json::to_string_pretty(&probe)?,
    )?;
    let receipt_path = "target/probes/validate-basic.json";
    let run = run_json_strings(&[
        "probe".into(),
        "run".into(),
        "--root".into(),
        fixture.display().to_string(),
        "--probe-file".into(),
        "probe.json".into(),
        "--out".into(),
        receipt_path.into(),
    ])?;
    assert_eq!(run["valid"], true);
    assert_eq!(run["coverage_class"], "positive");
    assert_eq!(run["fixtures"][0]["path"], fixture_file);
    assert_eq!(run["assertions"][1]["operator"], "file_blake3_equals");
    assert_hash_field(&run, "lifecycle_bundle_hash")?;
    assert_hash_field(&run, "receipt_hash")?;

    let probes_dir = fixture.join(".cairn/changes/demo/probes");
    fs::create_dir_all(&probes_dir)?;
    let mut gate_probe = probe;
    gate_probe["receipt_path"] = Value::String(receipt_path.to_string());
    fs::write(
        probes_dir.join("probes.json"),
        serde_json::to_string_pretty(&json!({"probes": [gate_probe]}))?,
    )?;
    let gate = run_json_strings(&[
        "gate".into(),
        "tasks".into(),
        "demo".into(),
        "--root".into(),
        fixture.display().to_string(),
    ])?;
    assert_eq!(gate["valid"], true);
    Ok(())
}

#[test]
fn default_gate_reports_missing_probe_receipt_without_running_probe() -> Result<(), Box<dyn Error>>
{
    // r[verify cairn.probe_execution_boundary.default_gate_no_execution]
    // r[verify cairn.probe_execution_boundary.default_gate_no_execution.existing_data_only]
    // r[verify cairn.probe_execution_boundary.default_gate_no_execution.missing_evidence]
    // r[verify cairn.probe_execution_boundary.fail_closed]
    let fixture = temp_fixture("probe-gate-missing-receipt", "fixtures/basic")?;
    let receipt_path = "target/probes/missing.json";
    let probes_dir = fixture.join(".cairn/changes/demo/probes");
    fs::create_dir_all(&probes_dir)?;
    fs::write(
        probes_dir.join("probes.json"),
        serde_json::to_string_pretty(
            &json!({"probes": [valid_probe_declaration(Some(receipt_path))]}),
        )?,
    )?;
    let gate = run_json_strings(&[
        "gate".into(),
        "tasks".into(),
        "demo".into(),
        "--root".into(),
        fixture.display().to_string(),
    ])?;
    assert_eq!(gate["valid"], false);
    assert!(!fixture.join(receipt_path).exists());
    let messages = gate["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .map(|issue| issue["message"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("saved probe receipt missing")));
    Ok(())
}

fn gate_messages_after_receipt_mutation(
    name: &str,
    mutate_receipt: impl FnOnce(&mut Value),
) -> Result<Vec<String>, Box<dyn Error>> {
    let fixture = temp_fixture(name, "fixtures/basic")?;
    fs::write(
        fixture.join("probe.json"),
        serde_json::to_string_pretty(&valid_probe_declaration(None))?,
    )?;
    let receipt_path = format!("target/probes/{name}.json");
    let _ = run_json_strings(&[
        "probe".into(),
        "run".into(),
        "--root".into(),
        fixture.display().to_string(),
        "--probe-file".into(),
        "probe.json".into(),
        "--out".into(),
        receipt_path.clone(),
    ])?;
    let full_receipt_path = fixture.join(&receipt_path);
    let mut receipt: Value = serde_json::from_slice(&fs::read(&full_receipt_path)?)?;
    mutate_receipt(&mut receipt);
    fs::write(&full_receipt_path, serde_json::to_string_pretty(&receipt)?)?;

    let probes_dir = fixture.join(".cairn/changes/demo/probes");
    fs::create_dir_all(&probes_dir)?;
    fs::write(
        probes_dir.join("probes.json"),
        serde_json::to_string_pretty(
            &json!({"probes": [valid_probe_declaration(Some(&receipt_path))]}),
        )?,
    )?;
    let gate = run_json_strings(&[
        "gate".into(),
        "tasks".into(),
        "demo".into(),
        "--root".into(),
        fixture.display().to_string(),
    ])?;
    assert_eq!(gate["valid"], false);
    Ok(gate["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .map(|issue| issue["message"].as_str().unwrap_or_default().to_string())
        .collect())
}

fn restamp_probe_receipt_hash(receipt: &mut Value) {
    receipt["receipt_hash"] = Value::String(cairn_core::probe_receipt_hash(receipt));
}

#[test]
fn gate_rejects_tampered_probe_receipt() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.probe_execution_boundary.gates_verify_receipts.command_mismatch]
    // r[verify cairn.probe_execution_boundary.fail_closed.tampered_receipt]
    // r[verify cairn.probe_execution_boundary.fail_closed.stale_hash]
    let messages =
        gate_messages_after_receipt_mutation("probe-gate-tampered-receipt", |receipt| {
            receipt["command_identity"] = Value::String("different".to_string())
        })?;
    assert!(messages
        .iter()
        .any(|message| message.contains("receipt_hash must equal")));
    assert!(messages
        .iter()
        .any(|message| message.contains("command_identity does not match")));
    Ok(())
}

#[test]
fn gate_rejects_stale_policy_hash() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.probe_execution_boundary.fail_closed.stale_hash]
    let alternate_policy_hash = cairn_core::canonical_json_hash(&json!({"policy": "other"}));
    let messages =
        gate_messages_after_receipt_mutation("probe-gate-stale-policy-hash", |receipt| {
            receipt["policy_hash"] = Value::String(alternate_policy_hash);
            restamp_probe_receipt_hash(receipt);
        })?;
    assert!(messages
        .iter()
        .any(|message| message.contains("policy_hash does not match")));
    Ok(())
}

#[test]
fn gate_rejects_undeclared_probe_id_in_receipt() -> Result<(), Box<dyn Error>> {
    let messages =
        gate_messages_after_receipt_mutation("probe-gate-undeclared-probe-id", |receipt| {
            receipt["probe_id"] = Value::String("undeclared-probe".to_string());
            restamp_probe_receipt_hash(receipt);
        })?;
    assert!(messages
        .iter()
        .any(|message| message.contains("probe_id does not match")));
    Ok(())
}

#[test]
fn gate_rejects_mismatched_requirement_ids() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.probe_execution_boundary.fail_closed.insufficient_coverage]
    let messages =
        gate_messages_after_receipt_mutation("probe-gate-mismatched-requirements", |receipt| {
            receipt["requirements"] = json!(["demo.other"]);
            restamp_probe_receipt_hash(receipt);
        })?;
    assert!(messages
        .iter()
        .any(|message| message.contains("receipt requirements must match")));
    Ok(())
}

#[test]
fn probe_run_denies_unauthorized_execute_by_default() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.probe_execution_boundary.explicit_probe_run.denied_command]
    // r[verify cairn.probe_execution_boundary.inline_probe_mode.disabled_default]
    let inline = run_json(&[
        "gate",
        "tasks",
        "demo",
        "--root",
        "fixtures/basic",
        "--run-probes",
    ])?;
    assert_eq!(inline["valid"], false);
    assert!(inline["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .any(|issue| issue["message"] == "inline probe execution is disabled by policy"));

    let fixture = temp_fixture("probe-run-denied", "fixtures/basic")?;
    let declaration = json!({
        "schema": "cairn.probe.v1",
        "id": "denied-archive",
        "requirements": ["demo.behavior"],
        "coverage_class": "negative",
        "command": {
            "profile": "cairn-local",
            "identity": "cairn archive execute",
            "argv": ["archive", "demo", "--root", ".", "--execute"]
        },
        "assertions": [{"id": "blocked", "path": "blocked", "equals": true}]
    });
    fs::write(
        fixture.join("denied.json"),
        serde_json::to_string_pretty(&declaration)?,
    )?;
    let output = run_json_failure(&[
        "probe".into(),
        "run".into(),
        "--root".into(),
        fixture.display().to_string(),
        "--probe-file".into(),
        "denied.json".into(),
    ])?;
    assert_eq!(output["valid"], false);
    assert!(output["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .any(|issue| issue["code"] == "probe.command.capability_denied"));
    Ok(())
}
