use std::path::PathBuf;

use cairn_core::{
    agent_run_preserves_receipt_hash, canonical_json_hash, evaluate_agent_run_completion,
    evaluate_authoritative_agent_completion, preserves_content_hash, AgentRunGateIssue, Policy,
};
use serde_json::{json, Value};

use crate::determinism::replay_groups_from_policy;
use crate::support::{path_string, with_receipt_hash};

pub(crate) struct AgentCompletionValidationInput<'a> {
    pub command: &'a str,
    pub contract_path: Option<PathBuf>,
    pub results_path: Option<PathBuf>,
    pub contract: &'a Value,
    pub results: &'a Value,
    pub current_subject_hash: &'a str,
    pub policy: &'a Policy,
    pub input_kind: &'a str,
}

pub(crate) fn agent_completion_validation_receipt(
    input: AgentCompletionValidationInput<'_>,
) -> Value {
    // r[impl cairn.supersede.agent_run_receipts.execution_boundary.no_execution]
    // r[impl cairn.supersede.agent_run_receipts.completion_authority.citation]
    let current_policy_hash = canonical_json_hash(&input.policy.raw);
    let report = evaluate_authoritative_agent_completion(
        input.contract,
        input.results,
        input.current_subject_hash,
        &current_policy_hash,
    );
    with_receipt_hash(json!({
        "command": input.command,
        "classification": report.classification,
        "input_kind": input.input_kind,
        "contract_path": input.contract_path.map(path_string),
        "results_path": input.results_path.map(path_string),
        "current_subject_hash": input.current_subject_hash,
        "current_policy_hash": current_policy_hash,
        "valid": report.valid,
        "verdict": report.verdict,
        "completion_receipt_hash": report.receipt_hash,
        "completion": report,
        "max_content_bytes": AGENT_RECEIPT_MAX_BYTES,
        "checks_executed": false,
    }))
}

use super::{AGENT_RECEIPT_MAX_BYTES, REQUIRED_AGENT_REPLAY_GROUP};

pub(crate) fn agent_receipt_validation_receipt(
    command: &str,
    path: Option<PathBuf>,
    receipt: &Value,
    is_mcp: bool,
    policy: &Policy,
) -> Value {
    // r[impl cairn.supersede.agent_receipt_validation_surface.cli]
    // r[impl cairn.supersede.agent_receipt_validation_surface.cli.schema]
    // r[impl cairn.supersede.agent_receipt_replay_evidence.validation]
    // r[impl cairn.supersede.agent_receipt_replay_evidence.failures]
    let report = evaluate_agent_run_completion(receipt);
    // r[impl cairn.supersede.agent_receipt_replay_group_coverage.validation]
    let current_replay_policy_hash = canonical_json_hash(&policy.raw);
    let (required_replay_cases, required_group_issue) = agent_required_replay_cases(policy);
    let replay_issues = agent_receipt_replay_evidence_issues(
        receipt,
        &current_replay_policy_hash,
        &required_replay_cases,
        required_group_issue,
    );
    let mut issues = report
        .issues
        .iter()
        .map(agent_gate_issue_json)
        .collect::<Vec<_>>();
    issues.extend(replay_issues.iter().map(agent_gate_issue_json));
    issues.sort_by_key(canonical_json_hash);
    let is_legacy_inspection = report.classification == "legacy_unscoped"
        && report
            .issues
            .iter()
            .all(|issue| issue.code == "verification.missing_contract");
    let is_valid = (report.valid || is_legacy_inspection) && replay_issues.is_empty();
    let verdict = if report.valid && replay_issues.is_empty() {
        "PASS"
    } else if is_valid {
        "INSPECTED"
    } else {
        "FAIL"
    };
    let input_kind = if is_mcp && path.is_none() {
        "inline"
    } else {
        "path"
    };
    with_receipt_hash(json!({
        "command": command,
        "path": path.map(path_string),
        "input_kind": input_kind,
        "classification": report.classification,
        "authoritative_completion": false,
        "valid": is_valid,
        "verdict": verdict,
        "issues": issues,
        "agent_receipt_hash": report.receipt_hash,
        "input_hash": canonical_json_hash(receipt),
        "stated_agent_receipt_hash": receipt.get("receipt_hash").and_then(Value::as_str).unwrap_or_default(),
        "current_replay_policy_hash": current_replay_policy_hash,
        "required_replay_group": REQUIRED_AGENT_REPLAY_GROUP,
        "required_replay_cases": required_replay_cases,
        "replay_evidence": agent_replay_evidence_summary(receipt),
        "max_content_bytes": AGENT_RECEIPT_MAX_BYTES,
    }))
}

fn agent_required_replay_cases(policy: &Policy) -> (Vec<String>, Option<AgentRunGateIssue>) {
    match replay_groups_from_policy(policy) {
        Ok(groups) => {
            let Some(group) = groups
                .iter()
                .find(|group| group.id == REQUIRED_AGENT_REPLAY_GROUP)
            else {
                return (
                    Vec::new(),
                    Some(AgentRunGateIssue {
                        code: "replay_evidence.required_group_missing".to_string(),
                        message: format!(
                            "generated replay policy must define required agent replay group: {REQUIRED_AGENT_REPLAY_GROUP}"
                        ),
                    }),
                );
            };
            let mut cases = group.cases.clone();
            cases.sort();
            cases.dedup();
            (cases, None)
        }
        Err(error) => (
            Vec::new(),
            Some(AgentRunGateIssue {
                code: "replay_evidence.required_group_missing".to_string(),
                message: format!(
                    "generated replay policy missing usable agent replay group metadata: {error}"
                ),
            }),
        ),
    }
}

fn agent_receipt_replay_evidence_issues(
    receipt: &Value,
    current_policy_hash: &str,
    required_replay_cases: &[String],
    required_group_issue: Option<AgentRunGateIssue>,
) -> Vec<AgentRunGateIssue> {
    let Some(evidence) = receipt.get("replay_evidence").and_then(Value::as_object) else {
        return vec![AgentRunGateIssue {
            code: "replay_evidence.missing".to_string(),
            message: "agent-run receipts require replay_evidence from a passing determinism replay receipt".to_string(),
        }];
    };
    let mut issues = Vec::new();
    if let Some(issue) = required_group_issue {
        issues.push(issue);
    }
    let replay_hash = evidence
        .get("replay_receipt_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !is_blake3_hex_text(replay_hash) {
        issues.push(AgentRunGateIssue {
            code: "replay_evidence.invalid_hash".to_string(),
            message: "replay_evidence.replay_receipt_hash must be a BLAKE3 hex hash".to_string(),
        });
    }
    let replay_verdict = evidence
        .get("replay_verdict")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if replay_verdict != "pass" {
        issues.push(AgentRunGateIssue {
            code: "replay_evidence.verdict_not_pass".to_string(),
            message: "replay_evidence.replay_verdict must be pass".to_string(),
        });
    }
    let replay_policy_hash = evidence
        .get("replay_policy_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if replay_policy_hash != current_policy_hash {
        issues.push(AgentRunGateIssue {
            code: "replay_evidence.policy_hash_stale".to_string(),
            message:
                "replay_evidence.replay_policy_hash must match the current generated policy hash"
                    .to_string(),
        });
    }
    let selected_groups = evidence
        .get("replay_selected_groups")
        .and_then(Value::as_array)
        .map(|groups| groups.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    if !selected_groups.contains(&REQUIRED_AGENT_REPLAY_GROUP) {
        issues.push(AgentRunGateIssue {
            code: "replay_evidence.missing_group_coverage".to_string(),
            message: format!(
                "replay evidence must include required selected group: {REQUIRED_AGENT_REPLAY_GROUP}"
            ),
        });
    }
    if let Some(attestation_hash) = evidence.get("replay_environment_attestation_hash") {
        if !attestation_hash.as_str().is_some_and(is_blake3_hex_text) {
            issues.push(AgentRunGateIssue {
                code: "replay_evidence.invalid_environment_attestation_hash".to_string(),
                message:
                    "replay_evidence.replay_environment_attestation_hash must be a BLAKE3 hex hash"
                        .to_string(),
            });
        }
    }
    let selected_cases = evidence
        .get("replay_selected_cases")
        .and_then(Value::as_array)
        .map(|cases| cases.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .unwrap_or_default();
    for required in required_replay_cases {
        if !selected_cases.contains(&required.as_str()) {
            issues.push(AgentRunGateIssue {
                code: "replay_evidence.insufficient_coverage".to_string(),
                message: format!("replay evidence missing required case: {required}"),
            });
        }
    }
    issues
}

fn agent_replay_evidence_summary(receipt: &Value) -> Value {
    let Some(evidence) = receipt.get("replay_evidence") else {
        return Value::Null;
    };
    json!({
        "replay_receipt_hash": evidence.get("replay_receipt_hash").cloned().unwrap_or(Value::Null),
        "replay_policy_hash": evidence.get("replay_policy_hash").cloned().unwrap_or(Value::Null),
        "replay_selected_cases": evidence.get("replay_selected_cases").cloned().unwrap_or(Value::Null),
        "replay_selected_groups": evidence.get("replay_selected_groups").cloned().unwrap_or(Value::Null),
        "replay_verdict": evidence.get("replay_verdict").cloned().unwrap_or(Value::Null),
        "replay_environment_attestation_hash": evidence.get("replay_environment_attestation_hash").cloned().unwrap_or(Value::Null),
        "replay_artifact_path": evidence.get("replay_artifact_path").cloned().unwrap_or(Value::Null),
    })
}

pub(crate) fn agent_preserves_hash_receipt(path: PathBuf, receipt: &Value) -> Value {
    // r[impl cairn.determinism.preserves_hash_cli.command]
    // r[impl cairn.determinism.preserves_hash_cli.bounded_read]
    // r[impl cairn.determinism.preserves_hash_cli.no_schema_churn]
    let mut issues = Vec::new();
    let agent_receipt_preserves_hash = if receipt.is_object() {
        match agent_run_preserves_receipt_hash(receipt) {
            Ok(hash) => hash,
            Err(error) => {
                issues.push(json!({
                    "code": "preserves.encode_failed",
                    "message": format!("failed to encode agent receipt as Preserves packed bytes: {error}"),
                }));
                String::new()
            }
        }
    } else {
        issues.push(json!({
            "code": "receipt.not_object",
            "message": "agent-run receipt must be a JSON object",
        }));
        String::new()
    };
    let content_hash = if receipt.is_object() {
        preserves_content_hash(receipt).ok()
    } else {
        None
    };
    issues.sort_by_key(canonical_json_hash);
    let is_valid = issues.is_empty();
    with_receipt_hash(json!({
        "command": "agent-receipt preserves-hash",
        "path": path_string(path),
        "input_kind": "path",
        "valid": is_valid,
        "verdict": if is_valid { "PASS" } else { "FAIL" },
        "issues": issues,
        "agent_receipt_preserves_hash": agent_receipt_preserves_hash,
        "preserves_content_hash": content_hash,
        "input_hash": canonical_json_hash(receipt),
        "max_content_bytes": AGENT_RECEIPT_MAX_BYTES,
    }))
}

pub(crate) fn is_blake3_hex_text(candidate: &str) -> bool {
    candidate.len() == 64 && candidate.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

fn agent_gate_issue_json(issue: &AgentRunGateIssue) -> Value {
    json!({"code": issue.code, "message": issue.message})
}
