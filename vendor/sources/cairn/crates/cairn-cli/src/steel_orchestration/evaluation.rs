use std::{fs, path::PathBuf};

use cairn_core::{
    blake3_hex, canonical_json_hash, validate_steel_action_envelope, SteelActionIssue,
    SteelOrchestrationProfile,
};
use serde_json::{json, Value};

const NO_SCRIPT_HASH: &str = "none";

pub(super) struct ScriptEvaluation {
    pub(super) runtime_available: bool,
    pub(super) script_requested: bool,
    pub(super) script_hash: String,
    pub(super) authorized_actions: Vec<Value>,
    pub(super) denied_actions: Vec<Value>,
    pub(super) issues: Vec<Value>,
}

impl ScriptEvaluation {
    pub(super) fn disabled(requested_hash: Option<&str>) -> Self {
        Self {
            runtime_available: false,
            script_requested: false,
            script_hash: requested_hash.unwrap_or(NO_SCRIPT_HASH).to_string(),
            authorized_actions: Vec::new(),
            denied_actions: Vec::new(),
            issues: Vec::new(),
        }
    }

    pub(super) fn has_denial(&self) -> bool {
        self.issues
            .iter()
            .any(|issue| issue.get("class").and_then(Value::as_str) == Some("denied"))
    }
}

// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split.evaluation]
pub(super) fn evaluate_script_if_enabled(
    is_policy_enabled: bool,
    profile: &SteelOrchestrationProfile,
    script_path: Option<&PathBuf>,
    requested_script_hash: Option<&str>,
) -> ScriptEvaluation {
    if !is_policy_enabled || profile.mode == "disabled" {
        return ScriptEvaluation::disabled(requested_script_hash);
    }

    let Some(script_path) = script_path else {
        let mut evaluation = ScriptEvaluation::disabled(requested_script_hash);
        evaluation.issues.push(json!({
            "code": "steel_runtime_unavailable",
            "class": "fallback",
            "message": "no reviewed Steel script envelope was provided"
        }));
        return evaluation;
    };

    let script_bytes = match fs::read(script_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            let mut evaluation = ScriptEvaluation::disabled(requested_script_hash);
            evaluation.script_requested = true;
            evaluation.issues.push(json!({
                "code": "steel_script_unavailable",
                "class": "fallback",
                "message": "reviewed Steel script envelope could not be read"
            }));
            return evaluation;
        }
    };
    let script_hash = blake3_hex(&script_bytes);
    let mut issues = Vec::new();
    if let Some(expected_hash) = requested_script_hash {
        if expected_hash != script_hash {
            issues.push(json!({
                "code": "script_hash_mismatch",
                "class": "denied",
                "message": "reviewed Steel script hash does not match the requested hash"
            }));
        }
    }

    let envelope: Value = match serde_json::from_slice(&script_bytes) {
        Ok(value) => value,
        Err(_) => {
            issues.push(json!({
                "code": "invalid_script_envelope_json",
                "class": "denied",
                "message": "reviewed Steel script envelope is not valid JSON"
            }));
            return ScriptEvaluation {
                runtime_available: true,
                script_requested: true,
                script_hash,
                authorized_actions: Vec::new(),
                denied_actions: Vec::new(),
                issues,
            };
        }
    };

    if envelope.get("profile_id").and_then(Value::as_str) != Some(profile.id.as_str()) {
        issues.push(json!({
            "code": "profile_mismatch",
            "class": "denied",
            "message": "reviewed Steel script envelope targets a different profile"
        }));
    }
    let validation = validate_steel_action_envelope(&envelope, &profile.allowed_host_functions);
    let denied_actions = validation
        .issues
        .iter()
        .map(steel_issue_json)
        .collect::<Vec<_>>();
    issues.extend(denied_actions.iter().cloned());
    let authorized_actions = if validation.valid && issues.is_empty() {
        envelope
            .get("actions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(redacted_action)
            .collect()
    } else {
        Vec::new()
    };

    ScriptEvaluation {
        runtime_available: true,
        script_requested: true,
        script_hash,
        authorized_actions,
        denied_actions,
        issues,
    }
}

pub(super) fn receipt_outcome(
    is_policy_enabled: bool,
    profile: &SteelOrchestrationProfile,
    evaluation: &ScriptEvaluation,
) -> &'static str {
    if !is_policy_enabled || profile.mode == "disabled" {
        "disabled"
    } else if evaluation.runtime_available && !evaluation.has_denial() {
        "planned"
    } else if evaluation.runtime_available {
        "denied"
    } else {
        "fallback"
    }
}

pub(super) fn fallback_reason(
    is_policy_enabled: bool,
    profile: &SteelOrchestrationProfile,
    evaluation: &ScriptEvaluation,
) -> &'static str {
    if !is_policy_enabled {
        "policy_disabled"
    } else if profile.mode == "disabled" {
        "profile_disabled"
    } else if evaluation.runtime_available {
        "none"
    } else if evaluation.script_requested {
        "steel_script_unavailable"
    } else {
        "steel_runtime_unavailable"
    }
}

fn steel_issue_json(issue: &SteelActionIssue) -> Value {
    json!({
        "code": issue.code,
        "class": "denied",
        "field_path": issue.field_path,
        "message": issue.message,
    })
}

fn redacted_action(action: Value) -> Value {
    json!({
        "id": action.get("id").and_then(Value::as_str).unwrap_or(""),
        "host_function": action.get("host_function").and_then(Value::as_str).unwrap_or(""),
        "request_hash": canonical_json_hash(&action),
    })
}

#[cfg(test)]
mod tests {
    use cairn_core::STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1;

    use super::*;

    #[test]
    fn enabled_profile_falls_back_until_runtime_is_wired() {
        let profile = SteelOrchestrationProfile {
            id: "comparison.lifecycle.closeout".into(),
            mode: "comparison".into(),
            allowed_host_functions: vec!["cairn.validate".into()],
            deterministic_budgets: vec!["no_fs_mutation".into()],
            redaction_classes: vec!["safe_metadata".into()],
            fallback_modes: vec!["rust_nickel".into()],
            receipt_schema_version: STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1.into(),
        };

        let evaluation = ScriptEvaluation::disabled(None);
        assert_eq!(receipt_outcome(true, &profile, &evaluation), "fallback");
        assert_eq!(
            fallback_reason(true, &profile, &evaluation),
            "steel_runtime_unavailable"
        );
    }
}
