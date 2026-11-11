use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, dependency_blocker_messages, evaluate_cross_repo_dependencies,
    CrossRepoDependencyReport, CrossRepoReceiptSummary,
};
use serde_json::{json, Value};

const KIBIBYTE_BYTES: u64 = 1024;
const MAX_EVIDENCE_FILE_KIBIBYTES: u64 = 64;
const MAX_EVIDENCE_FILE_BYTES: u64 = MAX_EVIDENCE_FILE_KIBIBYTES * KIBIBYTE_BYTES;
const MAX_EVIDENCE_FILES: usize = 64;
const SUMMARY_SCHEMA: &str = "cairn.cross-repo-receipt-summary.v1";

#[derive(Debug, Default)]
pub(crate) struct LoadedCrossRepoEvidence {
    pub(crate) summaries: Vec<CrossRepoReceiptSummary>,
    pub(crate) issues: Vec<String>,
}

pub(crate) fn evidence_paths_from_json(input: &Value) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let Some(value) = input.get("cross_repo_evidence") else {
        return Ok(Vec::new());
    };
    let items = value
        .as_array()
        .ok_or("cross_repo_evidence must be an array of explicit paths")?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(PathBuf::from)
                .ok_or_else(|| "cross_repo_evidence entries must be paths".into())
        })
        .collect()
}

pub(crate) fn load_cross_repo_evidence(root: &Path, inputs: &[PathBuf]) -> LoadedCrossRepoEvidence {
    // r[impl cairn.cross_repo_change_dependencies.shell]
    let mut paths = Vec::new();
    let mut issues = Vec::new();
    for input in inputs {
        let path = resolve_input(root, input);
        if path.is_dir() {
            match collect_json_files(&path) {
                Ok(mut discovered) => paths.append(&mut discovered),
                Err(error) => issues.push(format!(
                    "cross_repo_dependency.evidence_bundle_unreadable: {}",
                    error
                )),
            }
        } else {
            paths.push(path);
        }
    }
    paths.sort();
    paths.dedup();
    if paths.len() > MAX_EVIDENCE_FILES {
        return LoadedCrossRepoEvidence {
            summaries: Vec::new(),
            issues: vec![format!(
                "cross_repo_dependency.evidence_file_bound: {} files exceeds {}",
                paths.len(),
                MAX_EVIDENCE_FILES
            )],
        };
    }
    let mut summaries = Vec::new();
    for path in paths {
        match load_summary(root, &path) {
            Ok(summary) => summaries.push(summary),
            Err(error) => issues.push(error.to_string()),
        }
    }
    summaries.sort_by(|left, right| {
        (
            &left.repo,
            &left.change,
            &left.receipt_kind,
            &left.receipt_hash,
        )
            .cmp(&(
                &right.repo,
                &right.change,
                &right.receipt_kind,
                &right.receipt_hash,
            ))
    });
    LoadedCrossRepoEvidence { summaries, issues }
}

pub(crate) fn evaluate_change_cross_repo_dependencies(
    change: &cairn_core::ChangeStatus,
    evidence: &[CrossRepoReceiptSummary],
    policy_hash: &str,
) -> CrossRepoDependencyReport {
    evaluate_cross_repo_dependencies(
        &change.metadata.cross_repo_dependencies,
        evidence,
        policy_hash,
    )
}

pub(crate) fn cross_repo_reports(
    changes: &[cairn_core::ChangeStatus],
    evidence: &[CrossRepoReceiptSummary],
    policy_hash: &str,
) -> Vec<(String, CrossRepoDependencyReport)> {
    changes
        .iter()
        .filter(|change| !change.metadata.cross_repo_dependencies.is_empty())
        .map(|change| {
            (
                change.name.clone(),
                evaluate_change_cross_repo_dependencies(change, evidence, policy_hash),
            )
        })
        .collect()
}

pub(crate) fn cross_repo_blockers(
    change: &cairn_core::ChangeStatus,
    evidence: &[CrossRepoReceiptSummary],
    policy_hash: &str,
) -> Vec<String> {
    dependency_blocker_messages(&evaluate_change_cross_repo_dependencies(
        change,
        evidence,
        policy_hash,
    ))
}

pub(crate) fn reports_json(reports: &[(String, CrossRepoDependencyReport)]) -> Vec<Value> {
    reports
        .iter()
        .map(|(change, report)| {
            json!({
                "change": change,
                "report": report.to_json(),
            })
        })
        .collect()
}

fn resolve_input(root: &Path, input: &Path) -> PathBuf {
    if input.is_absolute() {
        input.to_path_buf()
    } else {
        root.join(input)
    }
}

fn collect_json_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(directory) = pending.pop() {
        reject_symlink(&directory)?;
        for entry in fs::read_dir(&directory)?.collect::<Result<Vec<_>, _>>()? {
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                return Err(
                    format!("cross-repo evidence path is a symlink: {}", path.display()).into(),
                );
            }
            if file_type.is_dir() {
                pending.push(path);
            } else if path.extension().and_then(|value| value.to_str()) == Some("json") {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

fn load_summary(root: &Path, path: &Path) -> Result<CrossRepoReceiptSummary, Box<dyn Error>> {
    reject_symlink(path)?;
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_EVIDENCE_FILE_BYTES {
        return Err(format!(
            "cross_repo_dependency.evidence_file_too_large: {}",
            safe_artifact_ref(root, path)
        )
        .into());
    }
    let bytes = fs::read(path)?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "cross_repo_dependency.evidence_malformed: {}: {error}",
            safe_artifact_ref(root, path)
        )
    })?;
    summarize_value(root, path, &value)
}

fn summarize_value(
    root: &Path,
    path: &Path,
    value: &Value,
) -> Result<CrossRepoReceiptSummary, Box<dyn Error>> {
    if value.get("schema").and_then(Value::as_str) != Some(SUMMARY_SCHEMA) {
        return Err(format!(
            "cross_repo_dependency.evidence_schema_unsupported: {}",
            safe_artifact_ref(root, path)
        )
        .into());
    }
    let required = |field: &str| {
        value
            .get(field)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                format!(
                    "cross_repo_dependency.evidence_field_missing: {}: {field}",
                    safe_artifact_ref(root, path)
                )
            })
    };
    let optional = |field: &str| value.get(field).and_then(Value::as_str).map(str::to_string);
    let embedded = value.get("receipt").cloned().unwrap_or(Value::Null);
    let embedded_hash = canonical_json_hash(&embedded);
    let declared_receipt_hash = optional("receipt_hash").unwrap_or_else(|| embedded_hash.clone());
    if !embedded.is_null() && declared_receipt_hash != embedded_hash {
        return Err(format!(
            "cross_repo_dependency.evidence_receipt_hash_mismatch: {}",
            safe_artifact_ref(root, path)
        )
        .into());
    }
    Ok(CrossRepoReceiptSummary {
        repo: required("repo")?,
        change: required("change")?,
        receipt_kind: required("receipt_kind")?,
        status: required("status")?,
        policy_hash: optional("policy_hash"),
        subject_hash: optional("subject_hash"),
        receipt_hash: declared_receipt_hash,
        artifact_ref: Some(safe_artifact_ref(root, path)),
        diagnostic_summary: optional("diagnostic_summary"),
    })
}

fn reject_symlink(path: &Path) -> Result<(), Box<dyn Error>> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(format!(
            "cross_repo_dependency.evidence_symlink_rejected: {}",
            path.file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("<evidence>")
        )
        .into());
    }
    Ok(())
}

fn safe_artifact_ref(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .filter(|relative| {
            !relative
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        })
        .map(|relative| relative.to_string_lossy().replace('\\', "/"))
        .or_else(|| {
            path.file_name()
                .and_then(|value| value.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "cross-repo-evidence.json".to_string())
}
