use serde_json::Value;
use std::path::Path;

use crate::{
    agent_receipt::AGENT_RECEIPT_MAX_BYTES,
    support::{portable_json_hash, CapabilityRoot},
};

use super::issue::evidence_republish_issue;

pub(super) fn read_import_receipt(
    imported_root: &CapabilityRoot,
    relative: &Path,
    issues: &mut Vec<Value>,
) -> Option<Value> {
    let receipt = imported_root.read(relative).ok().and_then(|bytes| {
        let byte_count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if byte_count > AGENT_RECEIPT_MAX_BYTES {
            return None;
        }
        serde_json::from_slice(&bytes).ok()
    });
    match receipt {
        Some(receipt) => Some(receipt),
        None => {
            issues.push(evidence_republish_issue(
                "evidence_republish.import_receipt_missing",
                "evidence-import-receipt.json",
                "readable import receipt",
                "missing",
            ));
            None
        }
    }
}

pub(super) fn validate_import_receipt(
    root: &Path,
    import_receipt: &Value,
    issues: &mut Vec<Value>,
) -> Value {
    let import_receipt_hash = import_receipt
        .get("receipt_hash")
        .cloned()
        .unwrap_or(Value::Null);
    if import_receipt.get("command").and_then(Value::as_str) != Some("evidence import") {
        issues.push(evidence_republish_issue(
            "evidence_republish.import_receipt_command_invalid",
            "command",
            "evidence import",
            import_receipt
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or("<missing>"),
        ));
    }
    if import_receipt.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_republish_issue(
            "evidence_republish.import_receipt_invalid",
            "valid",
            "true",
            "false",
        ));
    }
    if let Some(stated_hash) = import_receipt.get("receipt_hash").and_then(Value::as_str) {
        let mut receipt_hash_input = import_receipt.clone();
        if let Some(object) = receipt_hash_input.as_object_mut() {
            object.remove("receipt_hash");
        }
        let expected_hash = portable_json_hash(root, &receipt_hash_input);
        if stated_hash != expected_hash {
            issues.push(evidence_republish_issue(
                "evidence_republish.import_receipt_hash_mismatch",
                "receipt_hash",
                "portable receipt hash",
                "mismatch",
            ));
        }
    } else {
        issues.push(evidence_republish_issue(
            "evidence_republish.import_receipt_hash_missing",
            "receipt_hash",
            "BLAKE3 hex",
            "missing",
        ));
    }
    import_receipt_hash
}
