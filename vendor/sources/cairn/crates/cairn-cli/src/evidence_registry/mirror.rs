use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{
    path_string, portable_json_hash, with_portable_receipt_hash, CapabilityRoot, CapabilityRootKind,
};
use cairn_core::canonical_json_hash;

use super::build_verify::evidence_registry_verify_receipt;
use super::common::*;
use super::labels::{
    entry_labels_from_registry_entry, evidence_registry_reject_label_conflicts,
    evidence_registry_validate_labels,
};
use super::{EvidenceRegistryMirrorVerifyOptions, EvidenceRegistryVerifyOptions};

pub(super) fn evidence_registry_mirror_verify_receipt(
    options: &EvidenceRegistryMirrorVerifyOptions,
) -> Value {
    // r[impl cairn.supersede.registry_mirror_verification.command]
    // r[impl cairn.supersede.registry_mirror_verification.receipt]
    // r[impl cairn.supersede.registry_mirror_verification.failures]
    let root = options.root.clone();
    let registry_path = options
        .registry
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let mirror = options.mirror.clone().unwrap_or_else(|| PathBuf::from("."));
    let registry: Value =
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &registry_path, false)
            .and_then(|(registry_root, relative)| {
                registry_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
            })
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(value) => value,
            None => {
                return evidence_registry_mirror_verify_final_receipt(
                    &root,
                    vec![evidence_registry_issue(
                        "evidence_registry_mirror_verify.registry_unreadable",
                        "registry",
                        "readable registry",
                        "unreadable",
                    )],
                    Value::Null,
                    Value::Null,
                    Vec::new(),
                )
            }
        };
    let mut issues = Vec::new();
    let mirror_root = match CapabilityRoot::open(CapabilityRootKind::Registry, &mirror) {
        Ok(capability) => Some(capability),
        Err(_) => {
            issues.push(evidence_registry_issue(
                "evidence_registry_mirror_verify.mirror_unreadable",
                "mirror",
                "readable mirror root",
                "unreadable",
            ));
            None
        }
    };
    let registry_hash = registry.get("receipt_hash").cloned().unwrap_or(Value::Null);
    if let Some(hash) = registry_hash.as_str() {
        let mut hash_input = registry.clone();
        if let Some(object) = hash_input.as_object_mut() {
            object.remove("receipt_hash");
        }
        if portable_json_hash(&root, &hash_input) != hash {
            issues.push(evidence_registry_issue(
                "evidence_registry_mirror_verify.registry_hash_mismatch",
                "receipt_hash",
                "portable receipt hash",
                "mismatch",
            ));
        }
    } else {
        issues.push(evidence_registry_issue(
            "evidence_registry_mirror_verify.registry_hash_missing",
            "receipt_hash",
            "BLAKE3 hex",
            "missing",
        ));
    }
    let registry_verification = evidence_registry_verify_receipt(&EvidenceRegistryVerifyOptions {
        root: root.clone(),
        registry: Some(registry_path.clone()),
    });
    if registry_verification.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_registry_issue(
            "evidence_registry_mirror_verify.registry_verify_failed",
            "registry",
            "valid registry verification",
            "invalid",
        ));
    }
    let mut entries = Vec::new();
    for (position, entry) in registry
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let Some(index_path) = evidence_registry_resolve_index_path(&mirror, entry) else {
            issues.push(evidence_registry_issue(
                "evidence_registry_mirror_verify.index_path_unsafe",
                "entries.index_relative",
                "safe relative index path",
                "unsafe",
            ));
            continue;
        };
        let index_is_regular = mirror_root
            .as_ref()
            .and_then(|capability| capability.symlink_metadata_path(&index_path).ok())
            .is_some_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink());
        if !index_is_regular {
            issues.push(evidence_registry_issue(
                "evidence_registry_mirror_verify.index_missing",
                "entries.index_relative",
                "mirrored regular index file",
                "missing",
            ));
            continue;
        }
        let entry_labels = entry_labels_from_registry_entry(entry);
        if let Some(labels) = entry_labels.as_ref() {
            evidence_registry_validate_labels(
                &root,
                labels,
                "evidence_registry_mirror_verify",
                &mut issues,
            );
        }
        if let Some(current) = evidence_registry_entry_from_index(
            &root,
            &index_path,
            &mirror,
            position,
            "evidence_registry_mirror_verify",
            entry_labels.as_ref(),
            &mut issues,
        ) {
            if entry.get("index_receipt_hash") != current.get("index_receipt_hash") {
                issues.push(evidence_registry_issue(
                    "evidence_registry_mirror_verify.index_hash_mismatch",
                    "entries.index_receipt_hash",
                    "declared registry index hash",
                    "mismatch",
                ));
            } else if !evidence_registry_entries_equal(entry, &current) {
                issues.push(evidence_registry_issue(
                    "evidence_registry_mirror_verify.entry_mismatch",
                    "entries",
                    "current mirrored index entry",
                    "mismatch",
                ));
            }
            entries.push(current);
        }
    }
    entries.sort_by_key(canonical_json_hash);
    evidence_registry_reject_label_conflicts(
        &entries,
        "evidence_registry_mirror_verify",
        &mut issues,
    );
    evidence_registry_mirror_verify_final_receipt(
        &root,
        issues,
        registry_hash,
        registry_verification
            .get("receipt_hash")
            .cloned()
            .unwrap_or(Value::Null),
        entries,
    )
}

fn evidence_registry_mirror_verify_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    registry_hash: Value,
    registry_verification_receipt_hash: Value,
    entries: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence registry mirror-verify",
            "root": path_string(root),
            "registry": "<registry>",
            "mirror": "<mirror>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "registry_hash": registry_hash,
            "registry_verification_receipt_hash": registry_verification_receipt_hash,
            "entry_count": entries.len(),
            "entries": entries,
        }),
    )
}
