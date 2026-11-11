use cairn_core::canonical_json_hash;
use serde_json::{json, Value};
use std::path::Path;

use crate::support::{path_string, with_portable_content_hash_receipt};

pub(super) fn evidence_index_final_receipt(
    root: &Path,
    command: &str,
    mut issues: Vec<Value>,
    bundle_summary: Option<Value>,
    bundle_relative: Value,
    extra: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    let empty = json!({});
    let summary = bundle_summary.as_ref().unwrap_or(&empty);
    let mut receipt = json!({
        "command": command,
        "root": path_string(root),
        "bundle": "<bundle>",
        "bundle_relative": bundle_relative,
        "out": "<out>",
        "valid": valid,
        "verdict": if valid { "pass" } else { "fail" },
        "issues": issues,
        "bundle_kind": summary.get("bundle_kind").cloned().unwrap_or(Value::Null),
        "bundle_receipt": summary.get("bundle_receipt").cloned().unwrap_or(Value::Null),
        "bundle_receipt_hash": summary.get("bundle_receipt_hash").cloned().unwrap_or(Value::Null),
        "receipt_chain": summary.get("receipt_chain").cloned().unwrap_or_else(|| json!({})),
        "policy_hash": summary.get("policy_hash").cloned().unwrap_or(Value::Null),
        "artifacts": summary.get("artifacts").cloned().unwrap_or_else(|| json!([])),
    });
    if let Some(object) = receipt.as_object_mut() {
        if let Some(extra_object) = extra.as_object() {
            for (key, value) in extra_object {
                object.insert(key.clone(), value.clone());
            }
        }
    }
    with_portable_content_hash_receipt(root, receipt)
}
