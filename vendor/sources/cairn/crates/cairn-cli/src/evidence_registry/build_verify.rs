use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{portable_json_hash, CapabilityRoot, CapabilityRootKind};
use cairn_core::canonical_json_hash;

use super::common::*;
use super::labels::{
    entry_labels_from_registry_entry, evidence_registry_reject_label_conflicts,
    evidence_registry_validate_labels,
};
use super::{EvidenceRegistryOptions, EvidenceRegistryVerifyOptions};

pub(super) fn evidence_registry_receipt(options: &EvidenceRegistryOptions) -> Value {
    // r[impl cairn.supersede.evidence_registry_rail.build]
    // r[impl cairn.supersede.evidence_registry_rail.failures]
    let root = options.root.clone();
    let out = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let mut entries = Vec::new();
    let mut hashes = std::collections::BTreeSet::new();
    evidence_registry_validate_labels(&root, &options.labels, "evidence_registry", &mut issues);
    for (position, index_path) in options.indexes.iter().enumerate() {
        if let Some(entry) = evidence_registry_entry_from_index(
            &root,
            index_path,
            out.parent().unwrap_or_else(|| Path::new(".")),
            position,
            "evidence_registry",
            Some(&options.labels),
            &mut issues,
        ) {
            if let Some(hash) = entry.get("index_receipt_hash").and_then(Value::as_str) {
                if !hashes.insert(hash.to_string()) {
                    issues.push(evidence_registry_issue(
                        "evidence_registry.duplicate_index_hash",
                        "indexes",
                        "unique index receipt hashes",
                        "duplicate",
                    ));
                }
            }
            entries.push(entry);
        }
    }
    entries.sort_by_key(canonical_json_hash);
    evidence_registry_reject_label_conflicts(&entries, "evidence_registry", &mut issues);
    let mut receipt = evidence_registry_final_receipt(
        &root,
        "evidence registry build",
        issues,
        entries,
        "<out>",
        Value::Null,
    );
    if receipt.get("valid").and_then(Value::as_bool) == Some(true) {
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &out, true) {
            Ok((output_root, relative)) => {
                if output_root
                    .write(
                        &relative,
                        &serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
                    )
                    .is_err()
                {
                    receipt = evidence_registry_final_receipt(
                        &root,
                        "evidence registry build",
                        vec![evidence_registry_issue(
                            "evidence_registry.receipt_write_failed",
                            "out",
                            "writable registry receipt",
                            "failed",
                        )],
                        Vec::new(),
                        "<out>",
                        Value::Null,
                    );
                }
            }
            Err(_) => {
                receipt = evidence_registry_final_receipt(
                    &root,
                    "evidence registry build",
                    vec![evidence_registry_issue(
                        "evidence_registry.output_create_failed",
                        "out",
                        "writable output parent",
                        "failed",
                    )],
                    Vec::new(),
                    "<out>",
                    Value::Null,
                );
            }
        }
    }
    receipt
}

pub(crate) fn evidence_registry_verify_receipt(options: &EvidenceRegistryVerifyOptions) -> Value {
    // r[impl cairn.supersede.evidence_registry_rail.verify]
    // r[impl cairn.supersede.evidence_registry_rail.failures]
    let root = options.root.clone();
    let registry_path = options
        .registry
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
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
                return evidence_registry_verify_final_receipt(
                    &root,
                    vec![evidence_registry_issue(
                        "evidence_registry_verify.registry_unreadable",
                        "registry",
                        "readable registry",
                        "unreadable",
                    )],
                    Value::Null,
                    Vec::new(),
                )
            }
        };
    let mut issues = Vec::new();
    let stated_hash = registry.get("receipt_hash").cloned().unwrap_or(Value::Null);
    if let Some(hash) = stated_hash.as_str() {
        let mut hash_input = registry.clone();
        if let Some(object) = hash_input.as_object_mut() {
            object.remove("receipt_hash");
        }
        if portable_json_hash(&root, &hash_input) != hash {
            issues.push(evidence_registry_issue(
                "evidence_registry_verify.registry_hash_mismatch",
                "receipt_hash",
                "portable receipt hash",
                "mismatch",
            ));
        }
    } else {
        issues.push(evidence_registry_issue(
            "evidence_registry_verify.registry_hash_missing",
            "receipt_hash",
            "BLAKE3 hex",
            "missing",
        ));
    }
    let mut entries = Vec::new();
    let base = registry_path.parent().unwrap_or_else(|| Path::new("."));
    for (position, entry) in registry
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .iter()
        .enumerate()
    {
        let Some(index_path) = evidence_registry_resolve_index_path(base, entry) else {
            issues.push(evidence_registry_issue(
                "evidence_registry_verify.index_path_unsafe",
                "entries.index_relative",
                "safe relative index path",
                "unsafe",
            ));
            continue;
        };
        let entry_labels = entry_labels_from_registry_entry(entry);
        if let Some(labels) = entry_labels.as_ref() {
            evidence_registry_validate_labels(
                &root,
                labels,
                "evidence_registry_verify",
                &mut issues,
            );
        }
        if let Some(current) = evidence_registry_entry_from_index(
            &root,
            &index_path,
            base,
            position,
            "evidence_registry_verify",
            entry_labels.as_ref(),
            &mut issues,
        ) {
            if !evidence_registry_entries_equal(entry, &current) {
                issues.push(evidence_registry_issue(
                    "evidence_registry_verify.entry_mismatch",
                    "entries",
                    "current index entry",
                    "mismatch",
                ));
            }
            entries.push(current);
        }
    }
    entries.sort_by_key(canonical_json_hash);
    evidence_registry_reject_label_conflicts(&entries, "evidence_registry_verify", &mut issues);
    evidence_registry_verify_final_receipt(&root, issues, stated_hash, entries)
}
