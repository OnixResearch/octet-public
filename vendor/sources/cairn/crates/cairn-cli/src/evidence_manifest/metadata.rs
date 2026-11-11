use std::path::Path;

use cairn_core::canonical_json_hash;
use serde_json::Value;

use crate::agent_receipt::{read_json_artifact, AGENT_RECEIPT_MAX_BYTES};

use super::issues::evidence_manifest_issue;

pub(super) fn read_optional_manifest_metadata(
    path: Option<&Path>,
    field: &str,
    issues: &mut Vec<Value>,
) -> Option<Value> {
    let path = path?;
    match read_json_artifact(path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(value) if value.is_object() => Some(value),
        Ok(_) => {
            issues.push(evidence_manifest_issue(
                "generate.metadata_not_object",
                field,
                "JSON object",
                "non-object",
            ));
            None
        }
        Err(_) => {
            issues.push(evidence_manifest_issue(
                "generate.metadata_read_failed",
                field,
                "readable JSON object",
                "unreadable",
            ));
            None
        }
    }
}

pub(super) fn optional_metadata_hash(
    manifest: &Value,
    field: &str,
    issues: &mut Vec<Value>,
) -> Value {
    match manifest.get(field) {
        None | Some(Value::Null) => Value::Null,
        Some(value) if value.is_object() => Value::String(canonical_json_hash(value)),
        Some(_) => {
            issues.push(evidence_manifest_issue(
                "manifest.metadata_invalid",
                field,
                "object",
                "invalid",
            ));
            Value::Null
        }
    }
}
