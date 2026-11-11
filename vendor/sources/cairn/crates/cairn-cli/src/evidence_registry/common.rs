use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::evidence::{evidence_index_verify_receipt, EvidenceIndexVerifyOptions};
use crate::support::{path_string, with_portable_receipt_hash, CapabilityRoot, CapabilityRootKind};
use cairn_core::canonical_json_hash;

use super::labels::{evidence_registry_labels_json, EvidenceRegistryLabels};

pub(super) fn evidence_registry_entry_from_index(
    root: &Path,
    index_path: &Path,
    base: &Path,
    position: usize,
    prefix: &'static str,
    labels: Option<&EvidenceRegistryLabels>,
    issues: &mut Vec<Value>,
) -> Option<Value> {
    let index: Value =
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, index_path, false)
            .and_then(|(index_root, relative)| {
                index_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
            })
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(value) => value,
            None => {
                issues.push(evidence_registry_issue(
                    &format!("{prefix}.index_unreadable"),
                    "index",
                    "readable evidence index",
                    "unreadable",
                ));
                return None;
            }
        };
    if index.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_registry_issue(
            &format!("{prefix}.index_invalid"),
            "index",
            "valid evidence index",
            "invalid",
        ));
    }
    let verify_options = EvidenceIndexVerifyOptions {
        root: root.to_path_buf(),
        index: Some(index_path.to_path_buf()),
    };
    let verify = evidence_index_verify_receipt(&verify_options);
    if verify.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_registry_issue(
            &format!("{prefix}.index_verify_failed"),
            "index",
            "valid index verification",
            "invalid",
        ));
    }
    let index_relative = evidence_registry_index_relative(base, index_path);
    let labels = labels
        .map(evidence_registry_labels_json)
        .unwrap_or_else(|| evidence_registry_labels_json(&EvidenceRegistryLabels::default()));
    Some(json!({
        "index": format!("<index:{position}>"),
        "index_relative": index_relative,
        "index_receipt_hash": index.get("receipt_hash").cloned().unwrap_or(Value::Null),
        "index_verification_receipt_hash": verify.get("receipt_hash").cloned().unwrap_or(Value::Null),
        "labels": labels,
        "bundle_kind": index.get("bundle_kind").cloned().unwrap_or(Value::Null),
        "bundle_receipt_hash": index.get("bundle_receipt_hash").cloned().unwrap_or(Value::Null),
        "receipt_chain": index.get("receipt_chain").cloned().unwrap_or_else(|| json!({})),
        "policy_hash": index.get("policy_hash").cloned().unwrap_or(Value::Null),
        "artifacts": index.get("artifacts").cloned().unwrap_or_else(|| json!([])),
    }))
}

pub(super) fn evidence_registry_entries_equal(expected: &Value, current: &Value) -> bool {
    for key in [
        "index_receipt_hash",
        "labels",
        "bundle_kind",
        "bundle_receipt_hash",
        "receipt_chain",
        "policy_hash",
        "artifacts",
    ] {
        if expected.get(key) != current.get(key) {
            return false;
        }
    }
    true
}

fn evidence_registry_index_relative(base: &Path, index_path: &Path) -> Value {
    if let Ok(relative) = index_path.strip_prefix(base) {
        if let Some(text) = relative.to_str() {
            if !text.is_empty() && !text.starts_with('/') && !text.contains("..") {
                return json!(text);
            }
        }
    }
    if let (Some(parent), Some(name)) = (index_path.parent(), index_path.file_name()) {
        if parent == base {
            return json!(name.to_string_lossy().to_string());
        }
    }
    Value::Null
}

pub(super) fn evidence_registry_resolve_index_path(base: &Path, entry: &Value) -> Option<PathBuf> {
    let relative = entry.get("index_relative").and_then(Value::as_str)?;
    if relative.is_empty() || relative.starts_with('/') || relative.contains("..") {
        return None;
    }
    Some(base.join(relative))
}

pub(super) fn evidence_registry_issue(
    code: &str,
    field_path: &str,
    expected: &str,
    actual: &str,
) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}

pub(super) fn evidence_registry_final_receipt(
    root: &Path,
    command: &str,
    mut issues: Vec<Value>,
    entries: Vec<Value>,
    out: &str,
    registry_hash: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": command,
            "root": path_string(root),
            "out": out,
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "entry_count": entries.len(),
            "entries": entries,
            "registry_hash": registry_hash,
        }),
    )
}

pub(super) fn evidence_registry_verify_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    registry_hash: Value,
    entries: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence registry verify",
            "root": path_string(root),
            "registry": "<registry>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "registry_hash": registry_hash,
            "entry_count": entries.len(),
            "entries": entries,
        }),
    )
}
