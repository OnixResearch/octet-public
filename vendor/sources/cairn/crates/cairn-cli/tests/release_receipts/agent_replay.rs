use super::support::*;
use serde_json::{json, Value};
use std::error::Error;
use std::fs;

#[test]
fn agent_receipt_preserves_hash_cli_is_deterministic() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.determinism.preserves_hash_cli.command]
    // r[verify cairn.determinism.preserves_hash_cli.bounded_read]
    // r[verify cairn.determinism.preserves_hash_cli.no_schema_churn]
    let receipt = assert_reproducible_receipt(
        &[
            "agent-receipt",
            "preserves-hash",
            "fixtures/agent-receipts/positive.json",
        ],
        &["agent_receipt_preserves_hash", "input_hash", "receipt_hash"],
    )?;
    assert_object_keys(
        &receipt,
        &[
            "agent_receipt_preserves_hash",
            "command",
            "input_hash",
            "input_kind",
            "issues",
            "max_content_bytes",
            "path",
            "preserves_content_hash",
            "receipt_hash",
            "valid",
            "verdict",
        ],
    )?;
    assert_eq!(receipt["command"], "agent-receipt preserves-hash");
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "PASS");
    assert_eq!(receipt["issues"].as_array().ok_or("issues")?.len(), 0);
    assert_eq!(receipt["input_kind"], "path");
    assert_eq!(receipt["max_content_bytes"], 64 * 1024);
    assert_eq!(
        receipt["agent_receipt_preserves_hash"],
        "b7ea89b9561d00b0c9adcdcf03b267d16a5c3ff24e17321e86462e261e33c6c2"
    );
    Ok(())
}

#[test]
fn agent_receipt_preserves_hash_cli_reports_non_object_json() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.determinism.preserves_hash_cli.validation]
    let path = temp_project("preserves-hash-non-object")?.join("receipt.json");
    fs::write(&path, "[]\n")?;
    let path_string = path.display().to_string();
    let receipt = run_json(&["agent-receipt", "preserves-hash", &path_string])?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "FAIL");
    assert_eq!(receipt["agent_receipt_preserves_hash"], "");
    assert_eq!(receipt["issues"][0]["code"], "receipt.not_object");
    assert_hash_field(&receipt, "input_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn agent_receipt_validate_replays_fixtures_deterministically() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.agent_receipt_validation_surface]
    // r[verify cairn.supersede.agent_receipt_validation_surface.cli]
    // r[verify cairn.supersede.agent_receipt_validation_surface.cli.schema]
    // r[verify cairn.supersede.agent_receipt_validation_surface.docs]
    // r[verify cairn.supersede.agent_receipt_validation_surface.fixtures]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.schema]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.validation]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.failures]
    // r[verify cairn.supersede.agent_receipt_replay_evidence.replay]
    // r[verify cairn.supersede.agent_receipt_replay_group_coverage.validation]
    // r[verify cairn.supersede.agent_receipt_replay_group_coverage.replay]
    let positive = assert_reproducible_receipt(
        &[
            "agent-receipt",
            "validate",
            "fixtures/agent-receipts/positive.json",
        ],
        &["agent_receipt_hash", "input_hash", "receipt_hash"],
    )?;
    assert_object_keys(
        &positive,
        &[
            "agent_receipt_hash",
            "authoritative_completion",
            "classification",
            "command",
            "current_replay_policy_hash",
            "input_hash",
            "input_kind",
            "issues",
            "max_content_bytes",
            "path",
            "receipt_hash",
            "replay_evidence",
            "required_replay_cases",
            "required_replay_group",
            "stated_agent_receipt_hash",
            "valid",
            "verdict",
        ],
    )?;
    assert_eq!(positive["valid"], true);
    assert_eq!(positive["verdict"], "INSPECTED");
    assert_eq!(positive["classification"], "legacy_unscoped");
    assert_eq!(positive["authoritative_completion"], false);
    assert!(positive["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .any(|issue| issue["code"] == "verification.missing_contract"));
    assert_hash_field(&positive, "current_replay_policy_hash")?;
    assert_hash_field(&positive["replay_evidence"], "replay_receipt_hash")?;
    assert_hash_field(&positive["replay_evidence"], "replay_policy_hash")?;
    assert_eq!(positive["required_replay_group"], "agent");
    assert_eq!(
        positive["required_replay_cases"],
        json!(policy_replay_group_cases("agent")?)
    );
    assert_eq!(
        positive["replay_evidence"]["replay_selected_groups"],
        json!(["agent"])
    );
    assert_eq!(positive["replay_evidence"]["replay_verdict"], "pass");

    let negative = assert_reproducible_receipt(
        &[
            "agent-receipt",
            "validate",
            "fixtures/agent-receipts/missing-verification.json",
        ],
        &["agent_receipt_hash", "input_hash", "receipt_hash"],
    )?;
    assert_eq!(negative["valid"], false);
    let issues = negative["issues"].as_array().ok_or("negative issues")?;
    assert!(issues
        .iter()
        .any(|issue| issue["code"] == "verification.missing_contract"));
    Ok(())
}

#[test]
fn agent_receipt_replay_evidence_failures_are_deterministic() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.agent_receipt_replay_evidence.failures]
    // r[verify cairn.supersede.agent_receipt_replay_group_coverage.failures]
    // r[verify cairn.supersede.replay_environment_attestation.agent]
    let base_text = fs::read_to_string(repo_root().join("fixtures/agent-receipts/positive.json"))?;
    let base: Value = serde_json::from_str(&base_text)?;
    let cases = [
        (
            "missing",
            {
                let mut value = base.clone();
                value.as_object_mut().unwrap().remove("replay_evidence");
                value
            },
            "replay_evidence.missing",
        ),
        (
            "stale-policy",
            {
                let mut value = base.clone();
                value["replay_evidence"]["replay_policy_hash"] = Value::String(
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                );
                value
            },
            "replay_evidence.policy_hash_stale",
        ),
        (
            "insufficient",
            {
                let mut value = base.clone();
                value["replay_evidence"]["replay_selected_cases"] = json!(["status"]);
                value
            },
            "replay_evidence.insufficient_coverage",
        ),
        (
            "missing-group",
            {
                let mut value = base.clone();
                value["replay_evidence"]["replay_selected_groups"] = json!(["smoke"]);
                value
            },
            "replay_evidence.missing_group_coverage",
        ),
        (
            "bad-attestation-hash",
            {
                let mut value = base.clone();
                value["replay_evidence"]["replay_environment_attestation_hash"] =
                    json!("not-a-blake3-hash");
                value
            },
            "replay_evidence.invalid_environment_attestation_hash",
        ),
    ];
    for (name, value, expected_code) in cases {
        let path = temp_project(&format!("agent-replay-{name}"))?.join("receipt.json");
        fs::write(&path, serde_json::to_string_pretty(&value)? + "\n")?;
        let first = run_json(&[
            "agent-receipt",
            "validate",
            path.to_str().ok_or("utf8 receipt path")?,
        ])?;
        let second = run_json(&[
            "agent-receipt",
            "validate",
            path.to_str().ok_or("utf8 receipt path")?,
        ])?;
        assert_eq!(first["receipt_hash"], second["receipt_hash"]);
        assert_eq!(first["valid"], false);
        assert!(
            first["issues"]
                .as_array()
                .ok_or("issues")?
                .iter()
                .any(|issue| issue["code"] == expected_code),
            "{first:#?}"
        );
    }
    Ok(())
}
