use std::path::Path;

use cairn_core::blake3_hex;
use serde_json::{json, Value};

use super::paths::{publish_relative_path, safe_relative_path_text};
use crate::{
    release_evidence::common::release_evidence_issue,
    support::{CapabilityRoot, CapabilityRootKind},
};

pub(in crate::release_evidence) fn copy_publish_artifact(
    artifact: &Value,
    source_bundle_dir: &Path,
    publish_dir: &Path,
    materialized: &mut Vec<Value>,
    issues: &mut Vec<Value>,
) {
    let role = artifact
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let relative = artifact.get("path").and_then(Value::as_str).unwrap_or("");
    let expected_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
    if !safe_relative_path_text(relative) {
        issues.push(release_evidence_issue(
            "release_evidence_publish.artifact_path_unsafe",
            role,
            "safe relative artifact path",
            "unsafe",
        ));
        return;
    }
    let source = source_bundle_dir.join(relative);
    let destination = publish_dir.join("artifacts").join(relative);
    copy_publish_file(
        &source,
        &destination,
        role,
        Some(expected_hash),
        publish_dir,
        materialized,
        issues,
    );
}

pub(super) fn copy_publish_file(
    source: &Path,
    destination: &Path,
    role: &str,
    expected_hash: Option<&str>,
    publish_dir: &Path,
    materialized: &mut Vec<Value>,
    issues: &mut Vec<Value>,
) {
    let source_capability =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Release, source, false);
    let Ok((source_root, source_relative)) = source_capability else {
        issues.push(release_evidence_issue(
            "release_evidence_publish.source_unreadable",
            role,
            "readable source file",
            "unreadable",
        ));
        return;
    };
    if source_root
        .symlink_metadata(&source_relative)
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        issues.push(release_evidence_issue(
            "release_evidence_publish.source_symlink_rejected",
            role,
            "regular source file",
            "symlink",
        ));
        return;
    }
    let Ok(bytes) = source_root.read(&source_relative) else {
        issues.push(release_evidence_issue(
            "release_evidence_publish.source_unreadable",
            role,
            "readable source file",
            "unreadable",
        ));
        return;
    };
    let observed_hash = blake3_hex(&bytes);
    if let Some(expected) = expected_hash {
        if observed_hash != expected {
            issues.push(release_evidence_issue(
                "release_evidence_publish.source_hash_mismatch",
                role,
                "declared source hash",
                "mismatch",
            ));
            return;
        }
    }
    let destination_capability =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Release, destination, true);
    let Ok((destination_root, destination_relative)) = destination_capability else {
        issues.push(release_evidence_issue(
            "release_evidence_publish.destination_parent_create_failed",
            role,
            "writable destination parent",
            "failed",
        ));
        return;
    };
    if destination_root
        .write(&destination_relative, &bytes)
        .is_err()
    {
        issues.push(release_evidence_issue(
            "release_evidence_publish.destination_write_failed",
            role,
            "writable destination file",
            "failed",
        ));
        return;
    }
    let copied_hash = match destination_root.read(&destination_relative) {
        Ok(copied) => blake3_hex(&copied),
        Err(_) => {
            issues.push(release_evidence_issue(
                "release_evidence_publish.destination_unreadable",
                role,
                "readable copied file",
                "unreadable",
            ));
            return;
        }
    };
    if copied_hash != observed_hash {
        issues.push(release_evidence_issue(
            "release_evidence_publish.destination_hash_mismatch",
            role,
            "copied hash equals source hash",
            "mismatch",
        ));
        return;
    }
    materialized.push(json!({
        "role": role,
        "path": publish_relative_path(publish_dir, destination),
        "expected_hash": expected_hash.unwrap_or(&observed_hash),
        "observed_hash": copied_hash,
    }));
}
