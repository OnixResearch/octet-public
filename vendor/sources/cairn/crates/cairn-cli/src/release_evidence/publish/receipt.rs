use cairn_core::canonical_json_hash;
use serde_json::{json, Value};

use crate::support::{
    path_string, with_portable_content_hash_receipt, CapabilityRoot, CapabilityRootKind,
};

use super::labels::release_evidence_publish_labels;
use super::options::ReleaseEvidencePublishOptions;
use crate::release_evidence::common::release_evidence_issue;

pub(super) fn write_release_evidence_publish_receipt(
    options: &ReleaseEvidencePublishOptions,
    receipt: Value,
) -> Value {
    if receipt.get("valid").and_then(Value::as_bool) != Some(true) {
        return receipt;
    }
    let receipt_path = options.publish_dir.join("receipt.json");
    let output = CapabilityRoot::open_file_parent(CapabilityRootKind::Release, &receipt_path, true);
    let Ok((output_root, relative)) = output else {
        let mut issues = vec![release_evidence_issue(
            "release_evidence_publish.receipt_parent_create_failed",
            "publish_dir/receipt.json",
            "writable receipt parent",
            "failed",
        )];
        issues.sort_by_key(canonical_json_hash);
        return publish_receipt_with_replaced_issues(options, receipt, issues);
    };
    let bytes = serde_json::to_vec_pretty(&receipt).unwrap_or_default();
    if output_root.write(&relative, &bytes).is_err() {
        let mut issues = vec![release_evidence_issue(
            "release_evidence_publish.receipt_write_failed",
            "publish_dir/receipt.json",
            "writable publication receipt",
            "failed",
        )];
        issues.sort_by_key(canonical_json_hash);
        return publish_receipt_with_replaced_issues(options, receipt, issues);
    }
    receipt
}

fn publish_receipt_with_replaced_issues(
    options: &ReleaseEvidencePublishOptions,
    mut receipt: Value,
    issues: Vec<Value>,
) -> Value {
    if let Some(object) = receipt.as_object_mut() {
        object.remove("receipt_hash");
        object.remove("preserves_content_hash");
        object.insert("valid".to_string(), json!(false));
        object.insert("verdict".to_string(), json!("fail"));
        object.insert("issues".to_string(), Value::Array(issues));
    }
    with_portable_content_hash_receipt(&options.root, receipt)
}

pub(super) fn release_evidence_publish_final_receipt(
    options: &ReleaseEvidencePublishOptions,
    mut issues: Vec<Value>,
    source_index_hash: Option<String>,
    source_index_receipt_hash: Value,
    source_index_verification_receipt_hash: Value,
    manifest_hash: Option<String>,
    materialized: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        &options.root,
        json!({
            "command": "release-evidence publish",
            "root": path_string(&options.root),
            "index": path_string(&options.index),
            "artifact_dir": path_string(&options.artifact_dir),
            "publish_dir": path_string(&options.publish_dir),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "source_index_hash": source_index_hash.unwrap_or_default(),
            "source_index_receipt_hash": source_index_receipt_hash,
            "source_index_verification_receipt_hash": source_index_verification_receipt_hash,
            "manifest_hash": manifest_hash.unwrap_or_default(),
            "labels": release_evidence_publish_labels(options),
            "materialized_count": materialized.len(),
            "materialized": materialized,
            "issues": issues,
        }),
    )
}
