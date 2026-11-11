use cairn_core::canonical_json_hash;
use serde_json::{json, Value};
use std::path::Path;

use crate::support::{path_string, with_portable_content_hash_receipt};

pub(super) fn evidence_republish_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    mut artifacts: Vec<Value>,
    import_receipt_hash: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    artifacts.sort_by_key(|artifact| {
        format!(
            "{}:{}",
            artifact.get("role").and_then(Value::as_str).unwrap_or(""),
            artifact.get("path").and_then(Value::as_str).unwrap_or("")
        )
    });
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "evidence republish",
            "root": path_string(root),
            "imported": "<imported>",
            "out": "<out>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "artifacts": artifacts,
            "import_receipt_hash": import_receipt_hash,
        }),
    )
}
