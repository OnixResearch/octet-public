use std::{error::Error, path::PathBuf, process::Command};

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    let output = command.output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_json_failure(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    let output = command.output()?;
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn assert_hash_field(value: &Value, field: &str) -> Result<(), Box<dyn Error>> {
    let hash = value[field].as_str().ok_or("hash field missing")?;
    assert_eq!(hash.len(), 64, "{field} should be a BLAKE3 hex hash");
    assert!(
        hash.chars().all(|character| character.is_ascii_hexdigit()),
        "{field} should be hex: {hash}"
    );
    Ok(())
}

fn assert_issue_code(receipt: &Value, expected_code: &str) -> Result<(), Box<dyn Error>> {
    let issues = receipt["issues"].as_array().ok_or("issues array")?;
    assert!(
        issues.iter().any(|issue| issue["code"] == expected_code),
        "missing issue code {expected_code}: {issues:?}"
    );
    Ok(())
}

#[test]
fn function_object_evidence_validator_accepts_positive_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn refs: content_addressed_rust_evidence.function_object.source_token,
    // content_addressed_rust_evidence.canonical_hash.portable.
    let fixture = repo_root()
        .join("fixtures/content-addressed-rust-evidence/function-object-v1/positive.json");
    let receipt = run_json(&[
        "evidence",
        "function-object",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_object_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn function_object_evidence_validator_rejects_ambient_entropy_fixture() -> Result<(), Box<dyn Error>>
{
    // Active Cairn ref: content_addressed_rust_evidence.canonical_hash.no_entropy.
    let fixture = repo_root()
        .join("fixtures/content-addressed-rust-evidence/function-object-v1/ambient-entropy.json");
    let receipt = run_json_failure(&[
        "evidence",
        "function-object",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "function_object.ambient_entropy_in_hash_material")?;
    Ok(())
}

#[test]
fn name_pointer_evidence_validator_accepts_pointer_move_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn refs: content_addressed_rust_evidence.name_pointer.move,
    // content_addressed_rust_evidence.export_bundle.mutation_manifest.
    let fixture = repo_root()
        .join("fixtures/content-addressed-rust-evidence/pointers-and-caveats/pointer-move.json");
    let receipt = run_json(&[
        "evidence",
        "name-pointer",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_pointer_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn name_pointer_evidence_validator_rejects_drift_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.name_pointer.drift.
    let fixture = repo_root()
        .join("fixtures/content-addressed-rust-evidence/pointers-and-caveats/pointer-drift.json");
    let receipt = run_json_failure(&[
        "evidence",
        "name-pointer",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "name_pointer.drift_detected")?;
    Ok(())
}

#[test]
fn name_pointer_evidence_validator_rejects_unknown_effect_pure_cache_fixture(
) -> Result<(), Box<dyn Error>> {
    // Active Cairn refs: content_addressed_rust_evidence.caveats.dependency_status,
    // content_addressed_rust_evidence.caveats.effects_fail_closed.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/pointers-and-caveats/unknown-effect-pure-cache.json",
    );
    let receipt = run_json_failure(&[
        "evidence",
        "name-pointer",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "name_pointer.pure_cache_overclaim")?;
    Ok(())
}

#[test]
fn export_bundle_evidence_validator_accepts_positive_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn refs: content_addressed_rust_evidence.export_bundle.replay,
    // content_addressed_rust_evidence.compiler_ir.hir_dependencies.
    // r[verify cli_maintainability.export_bundle_core_modules.behavior_preserved.focused_checks]
    // r[verify cli_maintainability.export_bundle_core_modules.behavior_preserved.repository_checks]
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/ir-and-bundles/export-bundle-positive.json",
    );
    let receipt = run_json(&[
        "evidence",
        "export-bundle",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_bundle_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn export_bundle_evidence_validator_rejects_unsorted_objects() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.export_bundle.replay.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/ir-and-bundles/export-bundle-unsorted-objects.json",
    );
    let receipt = run_json_failure(&[
        "evidence",
        "export-bundle",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "export_bundle.unsorted_objects")?;
    Ok(())
}

#[test]
fn export_bundle_evidence_validator_rejects_unknown_pointer_object() -> Result<(), Box<dyn Error>> {
    // Active Cairn refs: content_addressed_rust_evidence.export_bundle.replay,
    // content_addressed_rust_evidence.export_bundle.mutation_manifest.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/ir-and-bundles/export-bundle-unknown-pointer.json",
    );
    let receipt = run_json_failure(&[
        "evidence",
        "export-bundle",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "export_bundle.unknown_pointer_object")?;
    Ok(())
}

#[test]
fn circuit_primitive_evidence_validator_accepts_positive_fixture() -> Result<(), Box<dyn Error>> {
    // r[impl electrical_circuit_primitives.topology.reference_node]
    // r[impl electrical_circuit_primitives.components.passive_primitives]
    // r[impl electrical_circuit_primitives.units.dimensioned_values]
    // r[impl electrical_circuit_primitives.receipts.portable_hashes]
    // r[impl electrical_circuit_primitive_receipts.validator.cli]
    // r[impl electrical_circuit_primitive_receipts.validator_cli]
    // r[impl electrical_circuit_primitive_receipts.policy]
    // r[impl electrical_circuit_primitive_receipts.policy.family]
    let fixture =
        repo_root().join("fixtures/electrical-circuit-primitives/positive-rc-filter.json");
    let receipt = run_json(&[
        "evidence",
        "circuit-primitive",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_graph_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn circuit_primitive_evidence_validator_rejects_missing_terminal() -> Result<(), Box<dyn Error>> {
    // r[impl electrical_circuit_primitives.topology.connection_arity]
    let fixture = repo_root().join("fixtures/electrical-circuit-primitives/missing-terminal.json");
    let receipt = run_json_failure(&[
        "evidence",
        "circuit-primitive",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "circuit_primitive.terminal_binding_missing")?;
    Ok(())
}

#[test]
fn circuit_primitive_evidence_validator_rejects_safety_overclaim() -> Result<(), Box<dyn Error>> {
    // r[impl electrical_circuit_primitives.non_claims.no_safety_claim]
    let fixture = repo_root().join("fixtures/electrical-circuit-primitives/safety-overclaim.json");
    let receipt = run_json_failure(&[
        "evidence",
        "circuit-primitive",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "circuit_primitive.non_claim_missing")?;
    Ok(())
}

#[test]
fn downstream_pilot_evidence_validator_accepts_crunch_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.downstream_pilots.crunch.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/downstream-pilots/crunch-mantle-positive.json",
    );
    let receipt = run_json(&[
        "evidence",
        "downstream-pilot",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_pilot_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn downstream_pilot_evidence_validator_accepts_aspen_fixture() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.downstream_pilots.aspen.
    let fixture = repo_root()
        .join("fixtures/content-addressed-rust-evidence/downstream-pilots/aspen-ci-positive.json");
    let receipt = run_json(&[
        "evidence",
        "downstream-pilot",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().map(Vec::len), Some(0));
    assert_hash_field(&receipt, "computed_pilot_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn downstream_pilot_evidence_validator_rejects_crunch_overclaim() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.downstream_pilots.crunch.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/downstream-pilots/crunch-correctness-overclaim.json",
    );
    let receipt = run_json_failure(&[
        "evidence",
        "downstream-pilot",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(&receipt, "downstream_pilot.build_correctness_overclaim")?;
    Ok(())
}

#[test]
fn downstream_pilot_evidence_validator_rejects_aspen_overclaim() -> Result<(), Box<dyn Error>> {
    // Active Cairn ref: content_addressed_rust_evidence.downstream_pilots.aspen.
    let fixture = repo_root().join(
        "fixtures/content-addressed-rust-evidence/downstream-pilots/aspen-distributed-overclaim.json",
    );
    let receipt = run_json_failure(&[
        "evidence",
        "downstream-pilot",
        "validate",
        fixture.to_str().ok_or("fixture path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_issue_code(
        &receipt,
        "downstream_pilot.distributed_correctness_overclaim",
    )?;
    Ok(())
}
