use super::*;
use crate::support::CapabilityRoot;

pub(super) fn export_agent_receipt_bound_to_replay(
    repository_root: &CapabilityRoot,
    output_root: &CapabilityRoot,
    template_path: &Path,
    output_path: &Path,
    replay_verification: &Value,
    policy: &Policy,
) -> Result<Value, Box<dyn Error>> {
    let template_bytes = repository_root.read_path(template_path)?;
    let template_byte_count = u64::try_from(template_bytes.len()).unwrap_or(u64::MAX);
    if template_byte_count > AGENT_RECEIPT_MAX_BYTES {
        return Err("agent receipt template exceeds the byte limit".into());
    }
    let mut receipt: Value = serde_json::from_slice(&template_bytes)?;
    let evidence = json!({
        "replay_artifact_path": "replay-agent.json",
        "replay_receipt_hash": replay_verification
            .get("replay_receipt_hash")
            .cloned()
            .unwrap_or(Value::Null),
        "replay_policy_hash": replay_verification
            .get("replay_policy_hash")
            .cloned()
            .unwrap_or(Value::Null),
        "replay_environment_attestation_hash": replay_verification
            .get("environment_attestation_hash")
            .cloned()
            .unwrap_or(Value::Null),
        "replay_selected_cases": replay_verification
            .get("selected_cases")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "replay_selected_groups": replay_verification
            .get("selected_groups")
            .cloned()
            .unwrap_or_else(|| json!([])),
        "replay_verdict": if replay_verification.get("valid").and_then(Value::as_bool) == Some(true) { "pass" } else { "fail" },
    });
    receipt["replay_evidence"] = evidence;
    if let Some(object) = receipt.as_object_mut() {
        object.remove("receipt_hash");
    }
    let validation = agent_receipt_validation_receipt(
        "agent-receipt validate",
        Some(output_path.to_path_buf()),
        &receipt,
        false,
        policy,
    );
    receipt["receipt_hash"] = validation
        .get("agent_receipt_hash")
        .cloned()
        .unwrap_or(Value::Null);
    let final_validation = agent_receipt_validation_receipt(
        "agent-receipt validate",
        Some(output_path.to_path_buf()),
        &receipt,
        false,
        policy,
    );
    if final_validation.get("valid").and_then(Value::as_bool) != Some(true) {
        return Err("generated agent receipt failed validation".into());
    }
    write_evidence_json(
        output_root,
        output_path,
        &receipt,
        &mut Vec::new(),
        "agent_receipt",
    );
    Ok(receipt)
}
