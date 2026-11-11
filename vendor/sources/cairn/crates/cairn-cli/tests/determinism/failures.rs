use std::error::Error;

use super::support::{assert_hash_field, cairn, run_json_failure, run_json_failure_command};

#[test]
fn determinism_replay_unknown_case_is_machine_readable_failure() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.determinism_replay.ci_case_selection]
    // r[verify cairn.supersede.determinism_replay.ci_failure_ergonomics]
    // r[verify cairn.supersede.determinism_replay.groups.failures]
    let value = run_json_failure(&[
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "not_a_case",
    ])?;
    assert_eq!(value["valid"], false);
    assert_eq!(value["mismatches"][0]["class"], "unknown_case");
    assert_eq!(value["mismatches"][0]["actual"], "not_a_case");
    assert!(value["mismatches"][0]["reproduce"]
        .as_str()
        .unwrap()
        .contains("--case not_a_case"));
    assert_hash_field(&value, "receipt_hash")?;

    let group = run_json_failure(&[
        "determinism",
        "replay",
        "--root",
        ".",
        "--group",
        "not_a_group",
    ])?;
    assert_eq!(group["valid"], false);
    assert_eq!(group["mismatches"][0]["class"], "unknown_group");
    assert_eq!(group["mismatches"][0]["actual"], "not_a_group");
    assert!(group["mismatches"][0]["reproduce"]
        .as_str()
        .unwrap()
        .contains("--group not_a_group"));

    let conflict = run_json_failure(&[
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "status",
        "--group",
        "smoke",
    ])?;
    assert_eq!(conflict["valid"], false);
    assert_eq!(conflict["mismatches"][0]["class"], "selector_conflict");
    assert_eq!(conflict["mismatches"][0]["field_path"], "--case/--group");
    assert_hash_field(&conflict, "receipt_hash")
}

#[test]
fn determinism_replay_rejects_mutation_flags() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.determinism_replay.no_mutation]
    let value = run_json_failure(&[
        "determinism",
        "replay",
        "--root",
        "fixtures/basic",
        "--execute",
    ])?;
    assert_eq!(value["valid"], false);
    assert_eq!(value["verdict"], "fail");
    assert_eq!(value["mismatches"][0]["code"], "replay.mutation_denied");
    assert_hash_field(&value, "receipt_hash")
}

#[test]
fn determinism_replay_rejects_hidden_environment_entropy() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.determinism_replay.hidden_entropy]
    // r[verify cairn.supersede.determinism_entropy_inventory.failures]
    // r[verify cairn.supersede.determinism_entropy_inventory.tests]
    let mut command = cairn();
    command
        .args(["determinism", "replay", "--root", "fixtures/basic"])
        .env("CAIRN_ARCHIVE_DATE", "2026-05-18");
    let value = run_json_failure_command(&mut command)?;
    assert_eq!(value["valid"], false);
    assert_eq!(value["verdict"], "fail");
    assert_eq!(
        value["nondeterminism_failures"][0]["code"],
        "replay.hidden_entropy.environment"
    );
    assert_eq!(
        value["nondeterminism_failures"][0]["source"],
        "CAIRN_ARCHIVE_DATE"
    );
    assert_eq!(
        value["nondeterminism_failures"][0]["inventory_id"],
        "archive_date_env"
    );
    assert_eq!(value["nondeterminism_failures"][0]["handling"], "reject");
    assert!(
        !serde_json::to_string(&value["nondeterminism_failures"])?.contains("2026-05-18"),
        "replay failure must not disclose rejected environment values"
    );
    assert_hash_field(&value, "entropy_policy_hash")?;
    assert_hash_field(&value, "receipt_hash")
}
