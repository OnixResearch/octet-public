use std::{
    path::{Path, PathBuf},
    process::Command,
};

use cairn_core::{blake3_hex, canonical_json_hash};
use serde_json::{json, Value};

use crate::{
    evidence::write_evidence_json,
    support::{CapabilityRoot, CapabilityRootKind},
};

pub(super) fn git_head_state(root: &Path, issues: &mut Vec<Value>) -> Value {
    let head = git_output(root, &["rev-parse", "--short", "HEAD"]);
    let branch = git_output(root, &["branch", "--show-current"]);
    let status = git_output(root, &["status", "--porcelain"]);
    let upstream = git_output(root, &["rev-parse", "--abbrev-ref", "@{u}"]).unwrap_or_default();
    let ahead_behind = if upstream.is_empty() {
        String::new()
    } else {
        git_output(
            root,
            &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
        )
        .unwrap_or_default()
    };

    if head.is_none() {
        issues.push(release_evidence_issue(
            "release_evidence.git_head_unavailable",
            "git.head",
            "readable git HEAD",
            "unavailable",
        ));
    }
    if let Some(status_text) = &status {
        if !status_text.is_empty() {
            issues.push(release_evidence_issue(
                "release_evidence.worktree_dirty",
                "git.status",
                "clean worktree",
                "dirty",
            ));
        }
    } else {
        issues.push(release_evidence_issue(
            "release_evidence.git_status_unavailable",
            "git.status",
            "readable git status",
            "unavailable",
        ));
    }

    json!({
        "payload_commit": head.unwrap_or_default(),
        "branch": branch.unwrap_or_default(),
        "upstream": upstream,
        "ahead_behind": ahead_behind,
        "clean": status.as_deref() == Some(""),
    })
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(super) fn collect_invalid_receipt(role: &str, receipt: &Value, issues: &mut Vec<Value>) {
    if receipt.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(release_evidence_issue(
            &format!("release_evidence.{role}_invalid"),
            role,
            "valid existing rail receipt",
            "invalid",
        ));
    }
}

pub(super) fn write_json_artifact(
    output_root: &CapabilityRoot,
    path: &Path,
    value: &Value,
    issues: &mut Vec<Value>,
    role: &str,
) {
    write_evidence_json(output_root, path, value, issues, role);
}

pub(super) fn write_markdown_artifact(path: &Path, markdown: &str, issues: &mut Vec<Value>) {
    let output = CapabilityRoot::open_file_parent(CapabilityRootKind::Release, path, true);
    let Ok((output_root, relative)) = output else {
        issues.push(release_evidence_issue(
            "release_evidence.index_parent_create_failed",
            "out",
            "writable output parent",
            "failed",
        ));
        return;
    };
    if output_root.write(&relative, markdown.as_bytes()).is_err() {
        issues.push(release_evidence_issue(
            "release_evidence.index_write_failed",
            "out",
            "writable Markdown index",
            "failed",
        ));
    }
}

pub(super) fn release_evidence_issue(
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

pub(super) fn read_file_hash(
    path: &Path,
    issues: &mut Vec<Value>,
    role: &str,
    field: &str,
) -> Option<String> {
    match CapabilityRoot::open_file_parent(CapabilityRootKind::Release, path, false)
        .and_then(|(file_root, relative)| file_root.read(&relative))
    {
        Ok(bytes) => Some(blake3_hex(&bytes)),
        Err(_) => {
            issues.push(release_evidence_issue(
                &format!("release_evidence_publish.{field}_unreadable"),
                role,
                "readable file",
                "unreadable",
            ));
            None
        }
    }
}

pub(super) fn write_json_file(path: &Path, value: &Value, issues: &mut Vec<Value>, role: &str) {
    let output = CapabilityRoot::open_file_parent(CapabilityRootKind::Release, path, true);
    let Ok((output_root, relative)) = output else {
        issues.push(release_evidence_issue(
            "release_evidence_publish.json_parent_create_failed",
            role,
            "writable JSON parent",
            "failed",
        ));
        return;
    };
    let Ok(bytes) = serde_json::to_vec_pretty(value) else {
        issues.push(release_evidence_issue(
            "release_evidence_publish.json_serialize_failed",
            role,
            "serializable JSON",
            "failed",
        ));
        return;
    };
    if output_root.write(&relative, &bytes).is_err() {
        issues.push(release_evidence_issue(
            "release_evidence_publish.json_write_failed",
            role,
            "writable JSON file",
            "failed",
        ));
    }
}

pub(super) fn safe_relative_path_text(relative: &str) -> bool {
    let path = PathBuf::from(relative);
    !relative.is_empty()
        && !path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
}

pub(super) fn directory_manifest_hash(path: &Path) -> Option<String> {
    let directory_root = CapabilityRoot::open(CapabilityRootKind::Release, path).ok()?;
    let mut entries = Vec::new();
    collect_directory_hashes(&directory_root, Path::new("."), &mut entries).ok()?;
    Some(canonical_json_hash(&json!(entries)))
}

fn collect_directory_hashes(
    directory_root: &CapabilityRoot,
    relative_directory: &Path,
    entries: &mut Vec<Value>,
) -> std::io::Result<()> {
    let mut children = directory_root.entries(relative_directory)?;
    children.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for child in children {
        let relative = if relative_directory == Path::new(".") {
            PathBuf::from(child.file_name())
        } else {
            relative_directory.join(child.file_name())
        };
        if child.file_type().is_symlink() {
            continue;
        }
        if child.file_type().is_dir() {
            collect_directory_hashes(directory_root, &relative, entries)?;
        } else if child.file_type().is_file() {
            let bytes = directory_root.read(&relative)?;
            let path = relative.to_string_lossy().replace('\\', "/");
            entries.push(json!({"path": path, "hash": blake3_hex(&bytes)}));
        }
    }
    Ok(())
}
