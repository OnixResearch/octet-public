use super::support::*;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;

#[test]

fn assert_gate_receipt_schema() -> Result<(), Box<dyn Error>> {
    let gate = run_json(&["gate", "tasks", "demo", "--root", "fixtures/basic"])?;
    let schema = receipt_schema("gate")?;
    assert_schema_keys(&gate, &schema, "top_level_keys")
}

fn assert_sync_receipt_schema() -> Result<(), Box<dyn Error>> {
    let sync = run_json(&["sync", "demo", "--root", "fixtures/basic"])?;
    let schema = receipt_schema("sync")?;
    assert_schema_keys(&sync, &schema, "top_level_keys")?;
    assert_first_action_matches_schema(&sync, &schema)
}

fn assert_archive_receipt_schema() -> Result<(), Box<dyn Error>> {
    let archive = run_json(&["archive", "ready", "--root", "fixtures/archive-ready"])?;
    let schema = receipt_schema("archive")?;
    assert_schema_keys(&archive, &schema, "top_level_keys")?;
    assert_first_action_matches_schema(&archive, &schema)
}

fn assert_migration_receipt_schema() -> Result<(), Box<dyn Error>> {
    let migration = run_json(&["migrate", "openspec", "--root", "fixtures/basic"])?;
    let schema = receipt_schema("migrate")?;
    assert_schema_keys(&migration, &schema, "top_level_keys")?;
    assert_first_action_matches_schema(&migration, &schema)
}

fn assert_agent_receipt_validation_schema() -> Result<(), Box<dyn Error>> {
    let receipt = run_json(&[
        "agent-receipt",
        "validate",
        "fixtures/agent-receipts/positive.json",
    ])?;
    let schema = receipt_schema("agent-receipt validate")?;
    assert_schema_keys(&receipt, &schema, "top_level_keys")?;
    assert_schema_keys(&receipt["replay_evidence"], &schema, "replay_evidence_keys")?;
    assert_hash_field(&receipt, "receipt_hash")
}

fn assert_replay_receipt_schema() -> Result<(), Box<dyn Error>> {
    let replay = run_json(&["determinism", "replay", "--root", "."])?;
    let schema = receipt_schema("determinism replay")?;
    assert_schema_keys(&replay, &schema, "top_level_keys")?;
    assert_schema_keys(&replay["cases"][0], &schema, "case_keys")?;
    assert_hash_field(&replay, "receipt_hash")
}

fn assert_replay_verify_receipt_schema() -> Result<(), Box<dyn Error>> {
    let artifact = temp_project("verify-replay-schema")?.join("receipt.json");
    let mut replay = cairn();
    replay.args([
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "status",
        "--format",
        "summary",
        "--out",
        artifact.to_str().unwrap(),
    ]);
    assert!(replay.output()?.status.success());
    let verify = run_json_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
        artifact.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    let schema = receipt_schema("determinism verify-receipt")?;
    assert_schema_keys(&verify, &schema, "top_level_keys")?;
    assert_hash_field(&verify, "receipt_hash")
}

fn assert_agent_evidence_bundle_schema() -> Result<(), Box<dyn Error>> {
    let dir = temp_project("bundle-schema")?;
    let replay_path = dir.join("replay.json");
    let verify_path = dir.join("verify.json");
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
    assert!(replay.output()?.status.success());
    let verify = run_json_strings(&[
        "determinism".into(),
        "verify-receipt".into(),
        replay_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    fs::write(&verify_path, serde_json::to_string_pretty(&verify)? + "\n")?;
    let agent_path = dir.join("agent-receipt.json");
    let mut agent_receipt: Value = serde_json::from_str(&fs::read_to_string(
        repo_root().join("fixtures/agent-receipts/positive.json"),
    )?)?;
    agent_receipt["replay_evidence"]["replay_receipt_hash"] = verify["replay_receipt_hash"].clone();
    agent_receipt["replay_evidence"]["replay_policy_hash"] = verify["replay_policy_hash"].clone();
    agent_receipt["replay_evidence"]["replay_environment_attestation_hash"] =
        verify["environment_attestation_hash"].clone();
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
    let bundle_path = dir.join("bundle.json");
    fs::write(
        &bundle_path,
        serde_json::to_string_pretty(&json!({
            "agent_receipt_path": agent_path,
            "replay_receipt_path": replay_path,
            "replay_verification_path": verify_path,
        }))? + "\n",
    )?;
    let receipt = run_json_strings(&[
        "agent-receipt".into(),
        "verify-bundle".into(),
        bundle_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    let schema = receipt_schema("agent-receipt verify-bundle")?;
    assert_schema_keys(&receipt, &schema, "top_level_keys")?;
    assert_hash_field(&receipt, "receipt_hash")
}

#[test]
fn public_receipt_schemas_are_stable() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.receipt_schema]
    // r[verify cairn.supersede.receipt_schema.gate]
    // r[verify cairn.supersede.receipt_schema.lifecycle]
    // r[verify cairn.supersede.receipt_schema.migration]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.schema]
    // r[verify cairn.supersede.agent_receipt_replay_group_coverage.schema]
    // r[verify cairn.supersede.determinism_replay.receipt.schema]
    // r[verify cairn.supersede.determinism_replay.groups.receipt]
    // r[verify cairn.supersede.determinism_entropy_inventory.schema]
    // r[verify cairn.supersede.replay_environment_attestation.policy]
    // r[verify cairn.supersede.replay_environment_attestation.receipt]
    // r[verify cairn.supersede.replay_environment_attestation.agent]
    // r[verify cairn.supersede.replay_receipt_verification.schema]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.schema]
    assert_gate_receipt_schema()?;
    assert_sync_receipt_schema()?;
    assert_archive_receipt_schema()?;
    assert_migration_receipt_schema()?;
    assert_agent_receipt_validation_schema()?;
    assert_replay_receipt_schema()?;
    assert_replay_verify_receipt_schema()?;
    assert_agent_evidence_bundle_schema()
}

#[test]
fn public_receipt_schema_docs_list_stable_fields() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.receipt_schema.docs]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.docs]
    // r[verify cairn.supersede.agent_receipt_replay_group_coverage.docs]
    // r[verify cairn.supersede.determinism_replay.groups.docs]
    // r[verify cairn.supersede.determinism_entropy_inventory.docs]
    // r[verify cairn.supersede.replay_environment_attestation.docs]
    // r[verify cairn.supersede.replay_receipt_verification.docs]
    // r[verify cairn.supersede.agent_evidence_bundle_verification.docs]
    let readme = fs::read_to_string(repo_root().join("README.md"))?;
    assert!(readme.contains("Stable public gate fields"), "{readme}");
    assert!(
        readme.contains("`change`, `stage`, `verdict`, `valid`"),
        "{readme}"
    );
    assert!(
        readme.contains("Stable public lifecycle\nreceipt fields"),
        "{readme}"
    );
    assert!(
        readme.contains("sync receipts also include `delta_specs`"),
        "{readme}"
    );
    assert!(
        readme.contains("Stable public migration receipt\nfields"),
        "{readme}"
    );
    assert!(
        readme.contains("`source_hash` and `destination_hash` fields"),
        "{readme}"
    );
    assert!(
        readme.contains("Stable public determinism replay fields"),
        "{readme}"
    );
    assert!(
        readme
            .contains("Mismatch entries expose stable `case`, `comparison`, `field_path`, `class`"),
        "{readme}"
    );
    assert!(
        readme.contains("Replay groups are policy-owned named suites"),
        "{readme}"
    );
    assert!(
        readme.contains("`available_groups`, `selected_groups`"),
        "{readme}"
    );
    assert!(
        readme.contains("`entropy_inventory`, `entropy_assumptions`, `entropy_policy_hash`"),
        "{readme}"
    );
    assert!(readme.contains("`environment_attestation`"), "{readme}");
    assert!(readme.contains("`attestation_hash`"), "{readme}");
    assert!(
        readme.contains("Stable public replay verification fields"),
        "{readme}"
    );
    assert!(
        readme.contains("`cairn determinism verify-receipt <receipt.json> --root .`"),
        "{readme}"
    );
    assert!(
        readme.contains("raw environment values, usernames"),
        "{readme}"
    );
    assert!(
        readme.contains("Stable public agent evidence bundle verification fields"),
        "{readme}"
    );
    assert!(
        readme.contains("`cairn agent-receipt verify-bundle <bundle.json> --root .`"),
        "{readme}"
    );
    assert!(readme.contains("Rejected\nentropy failures disclose source names and handling but never environment values"), "{readme}");
    assert!(
        readme.contains("Stable public agent receipt validation fields"),
        "{readme}"
    );
    assert!(
        readme.contains("`current_replay_policy_hash`, `required_replay_group`, `required_replay_cases`, `replay_evidence`"),
        "{readme}"
    );
    assert!(readme.contains("`replay_selected_groups`"), "{readme}");
    assert!(
        readme.contains("`cairn determinism replay --group agent`"),
        "{readme}"
    );
    Ok(())
}
