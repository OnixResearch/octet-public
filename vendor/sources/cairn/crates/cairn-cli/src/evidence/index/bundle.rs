use cairn_core::{blake3_hex, canonical_json_hash};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

use crate::{
    agent_receipt::AGENT_RECEIPT_MAX_BYTES,
    support::{portable_json_hash, CapabilityRoot, CapabilityRootKind},
};

use super::issue::evidence_index_issue;

pub(super) fn evidence_index_bundle_relative(bundle: &Path, out: &Path) -> Value {
    let Some(out_parent) = out.parent() else {
        return Value::Null;
    };
    if out_parent == bundle {
        return Value::Null;
    }
    if let (Some(bundle_parent), Some(bundle_name)) = (bundle.parent(), bundle.file_name()) {
        if bundle_parent == out_parent {
            return json!(bundle_name.to_string_lossy().to_string());
        }
    }
    Value::Null
}

pub(super) fn evidence_index_verify_bundle_path(index_path: &Path, index: &Value) -> PathBuf {
    let parent = index_path.parent().unwrap_or_else(|| Path::new("."));
    let parent_is_bundle = CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, parent)
        .ok()
        .is_some_and(|bundle_root| {
            [
                "evidence-republish-receipt.json",
                "evidence-import-receipt.json",
                "evidence-export-receipt.json",
            ]
            .iter()
            .any(|relative| {
                bundle_root
                    .symlink_metadata(Path::new(relative))
                    .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
            })
        });
    if parent_is_bundle {
        return parent.to_path_buf();
    }
    if let Some(relative) = index.get("bundle_relative").and_then(Value::as_str) {
        if !relative.is_empty() && !relative.starts_with('/') && !relative.contains("..") {
            return parent.join(relative);
        }
    }
    parent.to_path_buf()
}

pub(super) fn evidence_index_bundle_summary(
    root: &Path,
    bundle_root: &CapabilityRoot,
    bundle: &Path,
    prefix: &'static str,
    issues: &mut Vec<Value>,
) -> Option<Value> {
    let receipt_files = [
        ("republished", "evidence-republish-receipt.json"),
        ("imported", "evidence-import-receipt.json"),
        ("exported", "evidence-export-receipt.json"),
    ];
    let mut present = Vec::new();
    for (kind, file) in receipt_files {
        if bundle_root
            .symlink_metadata(Path::new(file))
            .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
        {
            present.push((kind, file));
        }
    }
    if present.is_empty() {
        issues.push(evidence_index_issue(
            &format!("{prefix}.bundle_receipt_missing"),
            "bundle",
            "one canonical evidence receipt",
            "missing",
        ));
        return None;
    }
    let (kind, receipt_file) = present[0];
    let receipt: Value = match bundle_root
        .read_bounded(Path::new(receipt_file), AGENT_RECEIPT_MAX_BYTES)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
    {
        Some(value) => value,
        None => {
            issues.push(evidence_index_issue(
                &format!("{prefix}.bundle_receipt_unreadable"),
                receipt_file,
                "readable bundle receipt",
                "unreadable",
            ));
            return None;
        }
    };
    if receipt.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_index_issue(
            &format!("{prefix}.bundle_receipt_invalid"),
            receipt_file,
            "valid bundle receipt",
            "invalid",
        ));
    }
    let mut artifacts = receipt
        .get("artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    artifacts.sort_by_key(canonical_json_hash);
    for artifact in artifacts.iter() {
        validate_artifact(prefix, issues, bundle_root, bundle, artifact);
    }
    Some(json!({
        "bundle_kind": kind,
        "bundle_receipt": receipt_file,
        "bundle_receipt_hash": receipt.get("receipt_hash").cloned().unwrap_or(Value::Null),
        "receipt_chain": evidence_index_receipt_chain(&receipt),
        "artifacts": artifacts,
        "policy_hash": evidence_index_policy_hash(&receipt),
        "root_hash": portable_json_hash(root, &receipt),
    }))
}

fn validate_artifact(
    prefix: &'static str,
    issues: &mut Vec<Value>,
    bundle_root: &CapabilityRoot,
    bundle: &Path,
    artifact: &Value,
) {
    let role = artifact
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let relative = artifact.get("path").and_then(Value::as_str).unwrap_or("");
    let declared_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
    if super::super::safe_bundle_artifact_path(bundle, relative).is_none() {
        issues.push(evidence_index_issue(
            &format!("{prefix}.artifact_path_unsafe"),
            role,
            "safe relative artifact path",
            "unsafe",
        ));
        return;
    }
    match bundle_root.read(Path::new(relative)) {
        Ok(bytes) => {
            if blake3_hex(&bytes) != declared_hash {
                issues.push(evidence_index_issue(
                    &format!("{prefix}.artifact_hash_mismatch"),
                    role,
                    "declared artifact hash",
                    "mismatch",
                ));
            }
        }
        Err(_) => issues.push(evidence_index_issue(
            &format!("{prefix}.artifact_missing"),
            role,
            "declared artifact file",
            "missing",
        )),
    }
}

pub(super) fn evidence_index_receipt_chain(receipt: &Value) -> Value {
    let keys = [
        "receipt_hash",
        "export_receipt_hash",
        "verification_receipt_hash",
        "import_receipt_hash",
        "coverage_audit_receipt_hash",
        "release_readiness_receipt_hash",
    ];
    let mut object = serde_json::Map::new();
    for key in keys {
        if let Some(value) = receipt.get(key) {
            object.insert(key.to_string(), value.clone());
        }
    }
    Value::Object(object)
}

pub(super) fn evidence_index_policy_hash(receipt: &Value) -> Value {
    receipt
        .get("current_replay_policy_hash")
        .or_else(|| receipt.get("replay_policy_hash"))
        .cloned()
        .unwrap_or(Value::Null)
}

pub(super) fn evidence_index_compare_summary(
    issues: &mut Vec<Value>,
    index: &Value,
    summary: &Value,
) {
    for key in [
        "bundle_kind",
        "bundle_receipt_hash",
        "receipt_chain",
        "artifacts",
    ] {
        if index.get(key) != summary.get(key) {
            issues.push(evidence_index_issue(
                "evidence_index_verify.index_bundle_mismatch",
                key,
                "indexed bundle value",
                "mismatch",
            ));
        }
    }
}
