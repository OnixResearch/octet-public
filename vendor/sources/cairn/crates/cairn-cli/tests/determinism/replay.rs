use std::{error::Error, fs};

use serde_json::{json, Value};

use super::support::{assert_hash_field, cairn, policy_json, repo_root, run_json, temp_project};

#[test]
fn determinism_replay_oracle_is_stable_and_non_mutating() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.determinism_replay.workspace_oracle]
    // r[verify cairn.supersede.determinism_replay.normalization]
    // r[verify cairn.supersede.determinism_replay.repeated]
    // r[verify cairn.supersede.determinism_replay.cross_root]
    // r[verify cairn.supersede.determinism_replay.receipt]
    // r[verify cairn.supersede.determinism_replay.mcp_cases]
    // r[verify cairn.supersede.determinism_replay.mismatch_diagnostics]
    // r[verify cairn.supersede.determinism_replay.policy_plan]
    // r[verify cairn.supersede.determinism_replay.policy_schema]
    // r[verify cairn.supersede.determinism_replay.expanded_surfaces]
    // r[verify cairn.supersede.determinism_replay.policy_diagnostics]
    // r[verify cairn.supersede.determinism_replay.ci_case_selection]
    // r[verify cairn.supersede.determinism_replay.ci_failure_ergonomics]
    // r[verify cairn.supersede.determinism_replay.groups.policy]
    // r[verify cairn.supersede.determinism_replay.groups.receipt]
    // r[verify cairn.supersede.determinism_entropy_inventory.policy]
    // r[verify cairn.supersede.determinism_entropy_inventory.receipt]
    // r[verify cairn.supersede.determinism_entropy_inventory.tests]
    // r[verify cairn.supersede.replay_environment_attestation.policy]
    // r[verify cairn.supersede.replay_environment_attestation.receipt]
    // r[verify cairn.supersede.replay_environment_attestation.secret_safe]
    // r[verify cairn.determinism.preserves_hash_cli.replay_policy]
    // r[verify cairn.determinism.preserves_hash_cli.replay_execution]
    // r[verify cairn.determinism.preserves_hash_cli.replay_validation]
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.replay_case]
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.tests]
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.validation]
    // r[verify cairn.lifecycle_views.determinism]
    let first = run_json(&["determinism", "replay", "--root", "."])?;
    let second = run_json(&["determinism", "replay", "--root", "."])?;
    assert_eq!(first["valid"], true);
    assert_eq!(first["verdict"], "pass");
    assert_eq!(first["mismatches"].as_array().unwrap().len(), 0);
    assert_eq!(
        first["nondeterminism_failures"].as_array().unwrap().len(),
        0
    );
    assert_eq!(first["receipt_hash"], second["receipt_hash"]);
    assert_eq!(first["summary"]["total_cases"], 19);
    assert_eq!(first["summary"]["mismatch_count"], 0);
    assert_eq!(first["available_cases"], first["selected_cases"]);
    assert_eq!(
        first["available_groups"],
        serde_json::json!(["agent", "full", "release", "smoke"])
    );
    assert_eq!(first["selected_groups"], serde_json::json!([]));
    assert_hash_field(&first, "receipt_hash")?;
    assert_hash_field(&first, "entropy_policy_hash")?;
    let policy = policy_json()?;
    let entropy_inventory = first["entropy_inventory"].as_array().unwrap();
    let mut expected_entropy_inventory = policy["determinism_replay_policy"]["entropy_inventory"]
        .as_array()
        .unwrap()
        .clone();
    expected_entropy_inventory.sort_by_key(|entry| entry["id"].as_str().unwrap().to_owned());
    assert_eq!(entropy_inventory, &expected_entropy_inventory);
    assert!(entropy_inventory
        .iter()
        .any(|entry| entry["id"] == "archive_date_env" && entry["handling"] == "reject"));
    assert!(first["entropy_assumptions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["id"] == "cwd_root_paths"));
    assert!(first["entropy_assumptions"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["handling"] != "reject"));
    let environment_attestation = &first["environment_attestation"];
    assert_eq!(environment_attestation["cairn_package"], "cairn");
    assert_eq!(
        environment_attestation["cairn_version"],
        env!("CARGO_PKG_VERSION")
    );
    assert_hash_field(environment_attestation, "generated_policy_hash")?;
    assert_hash_field(environment_attestation, "entropy_policy_hash")?;
    assert_hash_field(environment_attestation, "attestation_policy_hash")?;
    assert_hash_field(environment_attestation, "attestation_hash")?;
    assert_eq!(
        environment_attestation["entropy_policy_hash"],
        first["entropy_policy_hash"]
    );
    assert_eq!(
        environment_attestation["selected_cases"],
        first["selected_cases"]
    );
    assert_eq!(
        environment_attestation["selected_groups"],
        first["selected_groups"]
    );
    assert_eq!(
        environment_attestation["replay_scope"]["selector"],
        "default"
    );
    let mut expected_environment_facts = policy["determinism_replay_policy"]
        ["environment_attestation"]
        .as_array()
        .unwrap()
        .clone();
    expected_environment_facts.sort_by_key(|entry| entry["id"].as_str().unwrap().to_owned());
    assert_eq!(
        environment_attestation["policy_facts"].as_array().unwrap(),
        &expected_environment_facts
    );
    let attestation_text = environment_attestation.to_string();
    assert!(!attestation_text.contains(&repo_root().display().to_string()));
    assert!(!attestation_text.contains("SECRET"));
    let case_ids = first["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|case| case["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        case_ids,
        vec![
            "agent_completion_validate",
            "agent_receipt_preserves_hash",
            "agent_receipt_validate",
            "archive_dry_run",
            "gate_design",
            "gate_proposal",
            "gate_tasks",
            "mcp_describe",
            "mcp_status",
            "migrate_dry_run",
            "preserves_corpus_freshness",
            "receipt_preserves_diff",
            "release_readiness",
            "status",
            "sync_dry_run",
            "validate",
            "view_graph",
            "view_kanban",
            "view_mindmap"
        ]
    );
    let policy_case_count = policy["determinism_replay_policy"]["cases"]
        .as_array()
        .unwrap()
        .len();
    assert_eq!(first["cases"].as_array().unwrap().len(), policy_case_count);
    let policy_groups = policy["determinism_replay_policy"]["groups"]
        .as_array()
        .unwrap();
    assert_eq!(policy_groups.len(), 4);
    assert!(policy_groups.iter().any(|group| group["id"] == "agent"));
    assert!(policy_groups.iter().any(|group| group["id"] == "full"));
    assert!(first["cases"].as_array().unwrap().iter().all(|case| {
        case["runs"].as_array().unwrap().len() >= 2
            && case["expected_mutability"] == "read_only_or_dry_run"
            && case["fixture_root"].is_string()
    }));
    assert!(first["mutability_observations"]
        .as_array()
        .unwrap()
        .iter()
        .all(|observation| observation["mutated"] == false));
    Ok(())
}

#[test]
fn determinism_replay_supports_ci_case_selection_and_output_modes() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.determinism_replay.ci_case_selection]
    // r[verify cairn.supersede.determinism_replay.ci_output_modes]
    // r[verify cairn.supersede.determinism_replay.ci_tests_docs]
    // r[verify cairn.supersede.determinism_replay.groups.cli]
    // r[verify cairn.supersede.determinism_replay.groups.receipt]
    // r[verify cairn.supersede.determinism_replay.groups.docs]
    let selected = run_json(&[
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "agent_receipt_preserves_hash",
    ])?;
    // r[verify cairn.determinism.preserves_hash_cli.replay_policy]
    // r[verify cairn.determinism.preserves_hash_cli.replay_execution]
    // r[verify cairn.determinism.preserves_hash_cli.replay_validation]
    assert_eq!(selected["valid"], true);
    assert_eq!(
        selected["selected_cases"],
        serde_json::json!(["agent_receipt_preserves_hash"])
    );
    assert_eq!(selected["selected_groups"], serde_json::json!([]));
    assert_eq!(selected["summary"]["total_cases"], 1);
    assert_eq!(selected["cases"].as_array().unwrap().len(), 1);
    assert_eq!(selected["cases"][0]["id"], "agent_receipt_preserves_hash");
    assert_eq!(
        selected["cases"][0]["hash_fields"],
        serde_json::json!([
            "agent_receipt_preserves_hash",
            "input_hash",
            "preserves_content_hash",
            "receipt_hash"
        ])
    );
    assert_hash_field(&selected["cases"][0], "portable_evidence_hash")?;
    assert_hash_field(&selected["cases"][0]["runs"][0], "normalized_output_hash")?;

    let selected = run_json(&[
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "preserves_corpus_freshness",
    ])?;
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.replay_case]
    assert_eq!(selected["valid"], true);
    assert_eq!(
        selected["selected_cases"],
        serde_json::json!(["preserves_corpus_freshness"])
    );
    assert_eq!(selected["summary"]["total_cases"], 1);
    assert_eq!(selected["cases"][0]["id"], "preserves_corpus_freshness");
    assert_eq!(
        selected["cases"][0]["hash_fields"],
        serde_json::json!(["corpus_hash", "receipt_hash"])
    );
    assert!(selected["mutability_observations"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |observation| observation["case"] == "preserves_corpus_freshness"
                && observation["mutated"] == false
        ));
    assert_hash_field(&selected["cases"][0], "portable_evidence_hash")?;
    assert_hash_field(&selected["cases"][0]["runs"][0], "normalized_output_hash")?;

    let selected = run_json(&["determinism", "replay", "--root", ".", "--case", "status"])?;
    assert_eq!(selected["valid"], true);
    assert_eq!(selected["selected_cases"], serde_json::json!(["status"]));
    assert_eq!(selected["selected_groups"], serde_json::json!([]));
    assert_eq!(selected["summary"]["total_cases"], 1);
    assert_eq!(selected["cases"].as_array().unwrap().len(), 1);
    assert_eq!(selected["cases"][0]["id"], "status");

    let artifact = temp_project("replay-artifact")?.join("receipt.json");
    let mut summary = cairn();
    summary.args([
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
    let output = summary.output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("determinism replay: verdict=pass total=1"));
    assert!(stdout.contains("artifact="));
    let artifact_receipt: Value = serde_json::from_slice(&fs::read(&artifact)?)?;
    assert_eq!(artifact_receipt["receipt_hash"], selected["receipt_hash"]);

    let mut jsonl = cairn();
    jsonl.args([
        "determinism",
        "replay",
        "--root",
        ".",
        "--case",
        "status",
        "--format",
        "jsonl",
    ]);
    let output = jsonl.output()?;
    assert!(output.status.success());
    let lines = String::from_utf8(output.stdout)?;
    let events = lines
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(events[0]["event"], "summary");
    assert!(events
        .iter()
        .any(|event| event["event"] == "case" && event["id"] == "status"));

    let group = run_json(&["determinism", "replay", "--root", ".", "--group", "smoke"])?;
    assert_eq!(group["valid"], true);
    assert_eq!(group["selected_groups"], json!(["smoke"]));
    assert_eq!(group["selected_cases"], json!(["status", "validate"]));
    assert_eq!(group["summary"]["total_cases"], 2);
    Ok(())
}
