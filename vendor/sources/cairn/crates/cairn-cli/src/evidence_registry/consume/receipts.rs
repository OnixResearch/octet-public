use super::*;

fn evidence_registry_selectors_json(selectors: &EvidenceRegistryConsumeSelectors) -> Value {
    let mut object = serde_json::Map::new();
    if let Some(value) = selectors.bundle_kind.as_ref() {
        object.insert("bundle_kind".to_string(), json!(value));
    }
    if let Some(value) = selectors.policy_hash.as_ref() {
        object.insert("policy_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.channel.as_ref() {
        object.insert("channel".to_string(), json!(value));
    }
    if let Some(value) = selectors.release.as_ref() {
        object.insert("release".to_string(), json!(value));
    }
    if let Some(value) = selectors.platform.as_ref() {
        object.insert("platform".to_string(), json!(value));
    }
    if let Some(value) = selectors.compatibility_class.as_ref() {
        object.insert("compatibility_class".to_string(), json!(value));
    }
    if let Some(value) = selectors.index_receipt_hash.as_ref() {
        object.insert("index_receipt_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.index_verification_receipt_hash.as_ref() {
        object.insert("index_verification_receipt_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.bundle_receipt_hash.as_ref() {
        object.insert("bundle_receipt_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.artifact_role.as_ref() {
        object.insert("artifact_role".to_string(), json!(value));
    }
    if let Some(value) = selectors.artifact_hash.as_ref() {
        object.insert("artifact_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.coverage_audit_receipt_hash.as_ref() {
        object.insert("coverage_audit_receipt_hash".to_string(), json!(value));
    }
    if let Some(value) = selectors.release_readiness_receipt_hash.as_ref() {
        object.insert("release_readiness_receipt_hash".to_string(), json!(value));
    }
    Value::Object(object)
}

pub(super) fn evidence_registry_select_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    selectors: &EvidenceRegistryConsumeSelectors,
    verification: Value,
    selected_entry: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence registry select",
            "root": path_string(root),
            "registry": "<registry>",
            "out": "<out>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "selectors": evidence_registry_selectors_json(selectors),
            "registry_receipt_hash": verification.get("registry_hash").cloned().unwrap_or(Value::Null),
            "registry_verification_receipt_hash": verification.get("receipt_hash").cloned().unwrap_or(Value::Null),
            "selected_entry": selected_entry,
        }),
    )
}

pub(super) fn evidence_registry_fetch_plan_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    selectors: &EvidenceRegistryConsumeSelectors,
    verification: Value,
    entries: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence registry fetch-plan",
            "root": path_string(root),
            "registry": "<registry>",
            "out": "<out>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "selectors": evidence_registry_selectors_json(selectors),
            "registry_receipt_hash": verification.get("registry_hash").cloned().unwrap_or(Value::Null),
            "registry_verification_receipt_hash": verification.get("receipt_hash").cloned().unwrap_or(Value::Null),
            "entry_count": entries.len(),
            "entries": entries,
        }),
    )
}
