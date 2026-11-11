use std::path::Path;

use serde_json::Value;

use cairn_core::verified::policy::Policy;

use crate::determinism::determinism_verify_replay_receipt;

use super::hashes::receipt_stated_or_computed_hash;
use super::issues::{agent_bundle_issue, compare_bundle_field};
use super::{AGENT_RECEIPT_MAX_BYTES, REQUIRED_AGENT_REPLAY_GROUP};

pub(super) fn verify_agent_receipt(
    path: &Path,
    policy: &Policy,
    issues: &mut Vec<Value>,
) -> (Value, Value) {
    match super::super::io::read_agent_receipt_file(path) {
        Ok(receipt) => {
            let validation = super::super::validation::agent_receipt_validation_receipt(
                "agent-receipt validate",
                None,
                &receipt,
                false,
                policy,
            );
            let hash = validation["agent_receipt_hash"].clone();
            if validation["valid"].as_bool() != Some(true) {
                issues.push(agent_bundle_issue(
                    "agent_receipt.invalid",
                    "agent_receipt",
                    "valid agent receipt",
                    "invalid",
                ));
            }
            (hash, receipt)
        }
        Err(_) => {
            issues.push(agent_bundle_issue(
                "agent_receipt.read_failed",
                "agent_receipt_path",
                "readable agent receipt",
                "unreadable",
            ));
            (Value::Null, Value::Null)
        }
    }
}

pub(super) fn verify_replay_receipt(
    root: &Path,
    path: &Path,
    issues: &mut Vec<Value>,
) -> (Value, Value, Value) {
    let verifier = determinism_verify_replay_receipt(root, path);
    let receipt_hash = verifier["replay_receipt_hash"].clone();
    let attestation_hash = verifier["environment_attestation_hash"].clone();
    if verifier["valid"].as_bool() != Some(true) {
        issues.push(agent_bundle_issue(
            "replay_receipt.invalid",
            "replay_receipt",
            "valid saved replay receipt",
            "invalid",
        ));
    }
    (receipt_hash, attestation_hash, verifier)
}

pub(super) fn verify_saved_replay_verification(
    root: &Path,
    path: &Path,
    issues: &mut Vec<Value>,
) -> (Value, Value) {
    match super::super::io::read_json_artifact(path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(value) => {
            let hash = receipt_stated_or_computed_hash(root, &value);
            if value["valid"].as_bool() != Some(true) {
                issues.push(agent_bundle_issue(
                    "replay_verification.invalid",
                    "replay_verification",
                    "valid replay verification receipt",
                    "invalid",
                ));
            }
            (hash, value)
        }
        Err(_) => {
            issues.push(agent_bundle_issue(
                "replay_verification.read_failed",
                "replay_verification_path",
                "readable replay verification receipt",
                "unreadable",
            ));
            (Value::Null, Value::Null)
        }
    }
}

pub(super) fn verify_optional_receipt_artifact(
    root: &Path,
    path: &Path,
    issues: &mut Vec<Value>,
    issue_prefix: &str,
    expected: &str,
) -> Value {
    match super::super::io::read_json_artifact(path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(value) => {
            let hash = receipt_stated_or_computed_hash(root, &value);
            if value["valid"].as_bool() != Some(true) {
                issues.push(agent_bundle_issue(
                    &format!("{issue_prefix}.invalid"),
                    issue_prefix,
                    expected,
                    "invalid",
                ));
            }
            hash
        }
        Err(_) => {
            issues.push(agent_bundle_issue(
                &format!("{issue_prefix}.read_failed"),
                &format!("{issue_prefix}_path"),
                &format!("readable {issue_prefix} receipt"),
                "unreadable",
            ));
            Value::Null
        }
    }
}

pub(super) fn compare_replay_verification(
    issues: &mut Vec<Value>,
    saved_replay_verification: &Value,
    replay_verifier: &Value,
) {
    if saved_replay_verification.is_object() && replay_verifier.is_object() {
        compare_bundle_field(
            issues,
            "replay_verification.replay_receipt_hash_mismatch",
            "replay_verification.replay_receipt_hash",
            &replay_verifier["replay_receipt_hash"],
            &saved_replay_verification["replay_receipt_hash"],
        );
        compare_bundle_field(
            issues,
            "replay_verification.policy_hash_mismatch",
            "replay_verification.replay_policy_hash",
            &replay_verifier["replay_policy_hash"],
            &saved_replay_verification["replay_policy_hash"],
        );
        compare_bundle_field(
            issues,
            "replay_verification.attestation_hash_mismatch",
            "replay_verification.environment_attestation_hash",
            &replay_verifier["environment_attestation_hash"],
            &saved_replay_verification["environment_attestation_hash"],
        );
    }
}

pub(super) fn compare_agent_replay_evidence(
    issues: &mut Vec<Value>,
    agent_receipt: &Value,
    replay_verifier: &Value,
    current_policy_hash: &str,
) {
    if agent_receipt.is_object() && replay_verifier.is_object() {
        let evidence = agent_receipt.get("replay_evidence").unwrap_or(&Value::Null);
        compare_bundle_field(
            issues,
            "agent_replay_evidence.replay_hash_mismatch",
            "agent_receipt.replay_evidence.replay_receipt_hash",
            &replay_verifier["replay_receipt_hash"],
            evidence.get("replay_receipt_hash").unwrap_or(&Value::Null),
        );
        compare_bundle_field(
            issues,
            "agent_replay_evidence.policy_hash_mismatch",
            "agent_receipt.replay_evidence.replay_policy_hash",
            &Value::String(current_policy_hash.to_string()),
            evidence.get("replay_policy_hash").unwrap_or(&Value::Null),
        );
        compare_bundle_field(
            issues,
            "agent_replay_evidence.attestation_hash_mismatch",
            "agent_receipt.replay_evidence.replay_environment_attestation_hash",
            &replay_verifier["environment_attestation_hash"],
            evidence
                .get("replay_environment_attestation_hash")
                .unwrap_or(&Value::Null),
        );
        let groups = evidence
            .get("replay_selected_groups")
            .and_then(Value::as_array)
            .map(|groups| groups.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        if !groups.contains(&REQUIRED_AGENT_REPLAY_GROUP) {
            issues.push(agent_bundle_issue(
                "agent_replay_evidence.missing_group_coverage",
                "agent_receipt.replay_evidence.replay_selected_groups",
                REQUIRED_AGENT_REPLAY_GROUP,
                "missing",
            ));
        }
    }
}
