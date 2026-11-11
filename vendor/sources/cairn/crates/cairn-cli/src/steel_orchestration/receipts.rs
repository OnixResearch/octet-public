use cairn_core::{
    canonical_json_hash, SteelOrchestrationProfile, STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1,
};
use serde_json::{json, Value};

use super::{
    evaluation::{evaluate_script_if_enabled, fallback_reason, receipt_outcome, ScriptEvaluation},
    profiles::{fallback_profile, select_profile},
};

// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split.receipts]
pub(super) fn default_gate_receipt(
    args: &[String],
    profiles: &[SteelOrchestrationProfile],
    requested_profile_id: Option<&str>,
    steel_enabled: bool,
) -> Value {
    let selected_profile = select_profile(profiles, requested_profile_id);
    let profile = selected_profile.cloned().unwrap_or_else(fallback_profile);
    let mut issues = Vec::new();

    if selected_profile.is_none() {
        issues.push(json!({
            "code": "profile_not_found",
            "class": "denied",
            "message": "requested Steel orchestration profile is not declared by policy"
        }));
    }

    let is_default_candidate =
        matches!(profile.mode.as_str(), "selected_default" | "broad_default");
    if !steel_enabled {
        issues.push(json!({
            "code": "steel_policy_disabled",
            "class": "denied",
            "message": "default Steel promotion requires the policy Steel orchestration surface to be enabled"
        }));
    }
    if !is_default_candidate {
        issues.push(json!({
            "code": "profile_not_default_candidate",
            "class": "denied",
            "message": "default Steel promotion requires a policy-selected default workflow profile"
        }));
    }
    if is_default_candidate {
        require_promotion_hash(
            args,
            "--equivalence-receipt-hash",
            "equivalence_receipt_hash",
            &mut issues,
        );
        require_promotion_hash(
            args,
            "--fallback-receipt-hash",
            "fallback_receipt_hash",
            &mut issues,
        );
        require_promotion_hash(
            args,
            "--ci-reproducibility-receipt-hash",
            "ci_reproducibility_receipt_hash",
            &mut issues,
        );
        require_promotion_hash(
            args,
            "--diagnostics-receipt-hash",
            "diagnostics_receipt_hash",
            &mut issues,
        );
        if profile.fallback_modes.is_empty() {
            issues.push(json!({
                "code": "missing_fallback_mode",
                "class": "denied",
                "message": "default Steel profiles must retain a policy-declared fallback mode"
            }));
        }
    }

    let evidence = json!({
        "equivalence_receipt_hash": read_string_option(args, "--equivalence-receipt-hash"),
        "fallback_receipt_hash": read_string_option(args, "--fallback-receipt-hash"),
        "ci_reproducibility_receipt_hash": read_string_option(args, "--ci-reproducibility-receipt-hash"),
        "diagnostics_receipt_hash": read_string_option(args, "--diagnostics-receipt-hash"),
    });
    with_receipt_hash(json!({
        "schema_version": STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1,
        "command": "steel-orchestration default-gate",
        "valid": issues.iter().all(|issue| issue.get("class").and_then(Value::as_str) != Some("denied")),
        "verdict": if issues.iter().any(|issue| issue.get("class").and_then(Value::as_str) == Some("denied")) { "FAIL" } else { "PASS" },
        "profile_id": profile.id,
        "mode": profile.mode,
        "steel_enabled": steel_enabled,
        "default_candidate": is_default_candidate,
        "required_evidence": [
            "equivalence_receipt_hash",
            "fallback_receipt_hash",
            "ci_reproducibility_receipt_hash",
            "diagnostics_receipt_hash"
        ],
        "evidence": evidence,
        "operator_diagnostics_visible": read_string_option(args, "--diagnostics-receipt-hash").is_some(),
        "fallback_modes": profile.fallback_modes,
        "issues": issues,
        "receipt_hash": "",
    }))
}

// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split.receipts]
pub(super) fn orchestration_receipt(
    profiles: &[SteelOrchestrationProfile],
    requested_profile_id: Option<&str>,
    requested_script_hash: Option<&str>,
    script_path: Option<&std::path::PathBuf>,
    steel_enabled: bool,
) -> Value {
    let selected_profile = select_profile(profiles, requested_profile_id);
    let mut issues = Vec::new();
    if selected_profile.is_none() {
        issues.push(json!({
            "code": "profile_not_found",
            "class": "denied",
            "message": "requested Steel orchestration profile is not declared by policy"
        }));
    }

    let profile = selected_profile.cloned().unwrap_or_else(fallback_profile);
    let evaluation =
        evaluate_script_if_enabled(steel_enabled, &profile, script_path, requested_script_hash);
    issues.extend(evaluation.issues.clone());
    let outcome = receipt_outcome(steel_enabled, &profile, &evaluation);
    let fallback_reason = fallback_reason(steel_enabled, &profile, &evaluation);

    receipt_from_parts(
        profile,
        evaluation,
        issues,
        steel_enabled,
        outcome,
        fallback_reason,
    )
}

fn receipt_from_parts(
    profile: SteelOrchestrationProfile,
    evaluation: ScriptEvaluation,
    issues: Vec<Value>,
    steel_enabled: bool,
    outcome: &'static str,
    fallback_reason: &'static str,
) -> Value {
    with_receipt_hash(json!({
        "schema_version": STEEL_ORCHESTRATION_RECEIPT_SCHEMA_V1,
        "command": "steel-orchestration receipt",
        "valid": issues.iter().all(|issue| issue.get("class").and_then(Value::as_str) != Some("denied")),
        "verdict": if issues.iter().any(|issue| issue.get("class").and_then(Value::as_str) == Some("denied")) { "FAIL" } else { "PASS" },
        "outcome": outcome,
        "mode": profile.mode,
        "profile_id": profile.id,
        "steel_enabled": steel_enabled,
        "steel_runtime_available": evaluation.runtime_available,
        "effects_executed": false,
        "script_hash": evaluation.script_hash,
        "authorized_actions": evaluation.authorized_actions,
        "denied_actions": evaluation.denied_actions,
        "fallback_reason": fallback_reason,
        "redaction_classes": profile.redaction_classes,
        "receipt_schema_version": profile.receipt_schema_version,
        "issues": issues,
        "receipt_hash": "",
    }))
}

fn require_promotion_hash(
    args: &[String],
    flag: &'static str,
    field: &'static str,
    issues: &mut Vec<Value>,
) {
    match read_string_option(args, flag) {
        Some(hash) if is_hex_64(&hash) => {}
        Some(_) => issues.push(json!({
            "code": "invalid_promotion_evidence_hash",
            "class": "denied",
            "field": field,
            "message": "default Steel promotion evidence must be a 64-character lowercase hex receipt hash"
        })),
        None => issues.push(json!({
            "code": "missing_promotion_evidence",
            "class": "denied",
            "field": field,
            "message": "default Steel promotion requires equivalence, fallback, CI reproducibility, and diagnostics receipt evidence"
        })),
    }
}

fn is_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn with_receipt_hash(mut receipt: Value) -> Value {
    let mut material = receipt.clone();
    material["receipt_hash"] = json!("");
    receipt["receipt_hash"] = json!(canonical_json_hash(&material));
    receipt
}

fn read_string_option(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window.first().map(String::as_str) == Some(flag))
        .and_then(|window| window.get(1))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_promotion_hash_fails_closed() {
        let mut issues = Vec::new();
        let args = [
            "--equivalence-receipt-hash".to_string(),
            "not-hex".to_string(),
        ];

        require_promotion_hash(
            &args,
            "--equivalence-receipt-hash",
            "equivalence_receipt_hash",
            &mut issues,
        );

        assert_eq!(issues[0]["code"], "invalid_promotion_evidence_hash");
        assert_eq!(issues[0]["class"], "denied");
    }
}
