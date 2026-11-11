use cairn_core::blake3_hex;
use serde_json::{json, Value};
use std::{collections::BTreeSet, io::ErrorKind, path::Path};

use crate::support::CapabilityRoot;

use super::{issue::evidence_republish_issue, receipt::evidence_republish_final_receipt};

pub(super) fn materialize_declared_artifacts(
    imported_root: &CapabilityRoot,
    out_root: &CapabilityRoot,
    artifacts: &[Value],
    issues: &mut Vec<Value>,
) -> Vec<Value> {
    let mut copied = Vec::new();
    let mut seen_paths = BTreeSet::new();
    for artifact in artifacts {
        let role = artifact
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        let relative = artifact.get("path").and_then(Value::as_str).unwrap_or("");
        if !seen_paths.insert(relative.to_string()) {
            issues.push(evidence_republish_issue(
                "evidence_republish.duplicate_artifact_path",
                role,
                "unique artifact path",
                "duplicate",
            ));
            continue;
        }
        if super::super::safe_bundle_artifact_path(imported_root.path(), relative).is_none() {
            issues.push(evidence_republish_issue(
                "evidence_republish.artifact_path_unsafe",
                role,
                "safe relative artifact path",
                "unsafe",
            ));
            continue;
        }
        if super::super::safe_bundle_artifact_path(out_root.path(), relative).is_none() {
            issues.push(evidence_republish_issue(
                "evidence_republish.destination_path_unsafe",
                role,
                "safe relative destination path",
                "unsafe",
            ));
            continue;
        }
        let relative_path = Path::new(relative);
        let declared_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
        let source_bytes = match imported_root.read(relative_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                issues.push(evidence_republish_issue(
                    "evidence_republish.artifact_missing",
                    role,
                    "declared artifact file",
                    "missing",
                ));
                continue;
            }
        };
        let source_hash = blake3_hex(&source_bytes);
        if source_hash != declared_hash {
            issues.push(evidence_republish_issue(
                "evidence_republish.artifact_hash_mismatch",
                role,
                "declared artifact hash",
                "mismatch",
            ));
            continue;
        }
        if issues.is_empty() {
            if let Some(parent) = relative_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                if out_root.create_dir_all(parent).is_err() {
                    issues.push(evidence_republish_issue(
                        "evidence_republish.destination_create_failed",
                        role,
                        "writable destination parent",
                        "failed",
                    ));
                    continue;
                }
            }
            match out_root.symlink_metadata(relative_path) {
                Ok(_) => {
                    issues.push(evidence_republish_issue(
                        "evidence_republish.destination_collision",
                        role,
                        "absent destination artifact",
                        "exists",
                    ));
                    continue;
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {}
                Err(_) => {
                    issues.push(evidence_republish_issue(
                        "evidence_republish.destination_collision",
                        role,
                        "inspectable destination artifact",
                        "unreadable",
                    ));
                    continue;
                }
            }
            if out_root.write(relative_path, &source_bytes).is_err() {
                issues.push(evidence_republish_issue(
                    "evidence_republish.artifact_copy_failed",
                    role,
                    "copied artifact file",
                    "failed",
                ));
                continue;
            }
        }
        copied.push(json!({
            "role": role,
            "path": relative,
            "hash": source_hash,
        }));
    }
    copied
}

pub(super) fn copy_import_receipt_and_republish_receipt(
    root: &Path,
    out_root: &CapabilityRoot,
    import_receipt: &Value,
    receipt: Value,
) -> Value {
    let import_receipt_bytes = serde_json::to_vec_pretty(import_receipt).unwrap_or_default();
    if out_root
        .write(
            Path::new("evidence-import-receipt.json"),
            &import_receipt_bytes,
        )
        .is_err()
    {
        return evidence_republish_final_receipt(
            root,
            vec![evidence_republish_issue(
                "evidence_republish.import_receipt_copy_failed",
                "evidence-import-receipt.json",
                "copied import receipt",
                "failed",
            )],
            Vec::new(),
            receipt
                .get("import_receipt_hash")
                .cloned()
                .unwrap_or(Value::Null),
        );
    }
    if out_root
        .write(
            Path::new("evidence-republish-receipt.json"),
            &serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
        )
        .is_err()
    {
        return evidence_republish_final_receipt(
            root,
            vec![evidence_republish_issue(
                "evidence_republish.receipt_write_failed",
                "evidence-republish-receipt.json",
                "writable republish receipt",
                "failed",
            )],
            Vec::new(),
            receipt
                .get("import_receipt_hash")
                .cloned()
                .unwrap_or(Value::Null),
        );
    }
    receipt
}
