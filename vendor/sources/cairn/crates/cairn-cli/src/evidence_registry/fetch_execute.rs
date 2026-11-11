use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{path_string, with_portable_receipt_hash, CapabilityRoot, CapabilityRootKind};
use cairn_core::{blake3_hex, canonical_json_hash};

use super::common::evidence_registry_issue;
use super::EvidenceRegistryFetchExecuteOptions;

pub(super) fn evidence_registry_fetch_execute_receipt(
    options: &EvidenceRegistryFetchExecuteOptions,
) -> Value {
    // r[evidence_registry_execution.fetch.behavior.fetch.plan.materializes.verified.artifacts]
    // r[evidence_registry_execution.fetch.behavior.hash.mismatch.fails.closed]
    // r[evidence_registry_execution.fetch.safety.unsafe.path.is.rejected]
    // r[evidence_registry_execution.fetch.safety.no.hidden.network.dependency]
    let root = options.root.clone();
    let plan_path = options.plan.clone().unwrap_or_else(|| PathBuf::from("."));
    let source = options.source.clone().unwrap_or_else(|| PathBuf::from("."));
    let out = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let plan =
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &plan_path, false)
            .and_then(|(plan_root, relative)| {
                plan_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
            })
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        {
            Some(value) => value,
            None => {
                issues.push(evidence_registry_issue(
                    "evidence_registry_fetch_execute.plan_unreadable",
                    "plan",
                    "readable fetch plan receipt",
                    "unreadable",
                ));
                Value::Null
            }
        };
    if plan.get("valid").and_then(Value::as_bool) != Some(true)
        || plan.get("command").and_then(Value::as_str) != Some("evidence registry fetch-plan")
        || plan.get("receipt_hash").and_then(Value::as_str).is_none()
    {
        issues.push(evidence_registry_issue(
            "evidence_registry_fetch_execute.plan_unverified",
            "plan",
            "valid registry fetch-plan receipt",
            "unverified",
        ));
    }
    let mut materialized = Vec::new();
    let source_root = CapabilityRoot::open(CapabilityRootKind::Registry, &source);
    let output_root = CapabilityRoot::open_or_create(CapabilityRootKind::Registry, &out);
    if source_root.is_err() {
        issues.push(evidence_registry_issue(
            "evidence_registry_fetch_execute.source_unreadable",
            "source",
            "readable local source root",
            "unreadable",
        ));
    }
    if output_root.is_err() {
        issues.push(evidence_registry_issue(
            "evidence_registry_fetch_execute.output_create_failed",
            "out",
            "writable output root",
            "failed",
        ));
    }
    if let (true, Ok(source_root), Ok(output_root)) = (issues.is_empty(), source_root, output_root)
    {
        let entries = plan
            .get("entries")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for (entry_index, entry) in entries.iter().enumerate() {
            let artifacts = entry
                .get("artifacts")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for (artifact_index, artifact) in artifacts.iter().enumerate() {
                let field_path = format!("entries[{entry_index}].artifacts[{artifact_index}].path");
                let Some(relative) = artifact.get("path").and_then(Value::as_str) else {
                    issues.push(evidence_registry_issue(
                        "evidence_registry_fetch_execute.artifact_path_missing",
                        &field_path,
                        "artifact relative path",
                        "missing",
                    ));
                    continue;
                };
                let Some(safe_relative) = safe_registry_fetch_relative_path(relative) else {
                    issues.push(evidence_registry_issue(
                        "evidence_registry_fetch_execute.artifact_path_unsafe",
                        &field_path,
                        "safe relative artifact path",
                        "unsafe",
                    ));
                    continue;
                };
                let expected_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
                let bytes = match source_root.read(safe_relative) {
                    Ok(bytes) => bytes,
                    Err(_) => {
                        issues.push(evidence_registry_issue(
                            "evidence_registry_fetch_execute.artifact_missing",
                            &field_path,
                            "local source artifact",
                            "missing",
                        ));
                        continue;
                    }
                };
                let observed_hash = blake3_hex(&bytes);
                if observed_hash != expected_hash {
                    issues.push(evidence_registry_issue(
                        "evidence_registry_fetch_execute.artifact_hash_mismatch",
                        &format!("entries[{entry_index}].artifacts[{artifact_index}].hash"),
                        "declared artifact hash",
                        "mismatch",
                    ));
                    continue;
                }
                if let Some(parent) = safe_relative
                    .parent()
                    .filter(|parent| !parent.as_os_str().is_empty())
                {
                    if output_root.create_dir_all(parent).is_err() {
                        issues.push(evidence_registry_issue(
                            "evidence_registry_fetch_execute.output_create_failed",
                            "out",
                            "writable output parent",
                            "failed",
                        ));
                        continue;
                    }
                }
                if output_root.write(safe_relative, &bytes).is_err() {
                    issues.push(evidence_registry_issue(
                        "evidence_registry_fetch_execute.output_write_failed",
                        "out",
                        "writable output artifact",
                        "failed",
                    ));
                    continue;
                }
                let materialized_hash = match output_root.read(safe_relative) {
                    Ok(saved) => blake3_hex(&saved),
                    Err(_) => "missing".to_string(),
                };
                if materialized_hash != expected_hash {
                    issues.push(evidence_registry_issue(
                        "evidence_registry_fetch_execute.materialized_hash_mismatch",
                        "out",
                        "materialized artifact hash",
                        "mismatch",
                    ));
                    continue;
                }
                materialized.push(json!({
                    "entry_index": entry_index,
                    "artifact_index": artifact_index,
                    "role": artifact.get("role").cloned().unwrap_or(Value::Null),
                    "path": relative,
                    "destination": relative,
                    "expected_hash": expected_hash,
                    "observed_hash": observed_hash,
                }));
            }
        }
    }
    evidence_registry_fetch_execute_final_receipt(&root, issues, &plan, materialized)
}

fn safe_registry_fetch_relative_path(path: &str) -> Option<&Path> {
    let relative = Path::new(path);
    if path.is_empty() || path.contains('\0') || relative.is_absolute() {
        return None;
    }
    if relative
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(relative)
}

fn evidence_registry_fetch_execute_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    plan: &Value,
    mut materialized: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    materialized.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence registry fetch-execute",
            "root": path_string(root),
            "plan": "<plan>",
            "source": "<source>",
            "out": "<out>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "fetch_plan_receipt_hash": plan.get("receipt_hash").cloned().unwrap_or(Value::Null),
            "registry_receipt_hash": plan.get("registry_receipt_hash").cloned().unwrap_or(Value::Null),
            "registry_verification_receipt_hash": plan
                .get("registry_verification_receipt_hash")
                .cloned()
                .unwrap_or(Value::Null),
            "materialized_count": materialized.len(),
            "materialized": materialized,
        }),
    )
}
