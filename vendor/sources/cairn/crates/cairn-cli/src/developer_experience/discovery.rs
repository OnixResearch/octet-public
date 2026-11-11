use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{
    plan_reverse_spec_discovery, DiscoverySnapshot, DISCOVERY_MAX_BYTES, DISCOVERY_MAX_FILES,
    DISCOVERY_POLICY_ID,
};
use serde_json::{json, Value};

use crate::developer_experience::common::write_output_file;
use crate::support::{
    path_string, print_json, read_option, read_string_option, safe_relative_path,
    with_portable_receipt_hash,
};

pub(super) fn run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.developer_experience.reverse_spec_discovery.cli]
    let root = read_option(args.clone(), "--root", ".");
    let out = read_string_option(&args, "--out").map(PathBuf::from);
    let include_paths = include_values(&args);
    let mut snapshots = Vec::new();
    let mut skipped = Vec::new();
    for include in include_paths {
        collect_discovery_include(&root, &include, &mut snapshots, &mut skipped)?;
    }
    let plan = plan_reverse_spec_discovery(snapshots);
    let mut issues = plan.issues.clone();
    let mut written = Vec::new();
    if let Some(out) = out.as_ref() {
        if plan.valid {
            let draft = discovery_draft(
                &plan
                    .candidates
                    .iter()
                    .map(candidate_json)
                    .collect::<Vec<_>>(),
            );
            write_output_file(&root, out, &draft, &mut issues, &mut written)?;
        }
    }
    let receipt = with_portable_receipt_hash(
        &root,
        json!({
            "command": "discover_specs",
            "root": path_string(&root),
            "valid": issues.is_empty(),
            "policy": DISCOVERY_POLICY_ID,
            "limits": {"max_files": DISCOVERY_MAX_FILES, "max_bytes": DISCOVERY_MAX_BYTES},
            "issues": issues,
            "selected_paths": plan.selected_paths,
            "skipped_paths": merge_skipped(plan.skipped_paths, skipped),
            "candidates": plan.candidates.into_iter().map(|candidate| json!({
                "id": candidate.id,
                "evidence_path": candidate.evidence_path,
                "summary": candidate.summary,
                "review_status": candidate.review_status,
            })).collect::<Vec<_>>(),
            "input_hash": plan.input_hash,
            "written": written,
        }),
    );
    print_json(receipt)
}

fn include_values(args: &[String]) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == "--include")
        .map(|pair| pair[1].clone())
        .collect()
}

fn collect_discovery_include(
    root: &Path,
    include: &str,
    snapshots: &mut Vec<DiscoverySnapshot>,
    skipped: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.developer_experience.reverse_spec_directory_inventory.expand]
    if !is_safe_discovery_path(include) {
        skipped.push(include.to_string());
        return Ok(());
    }
    let path = root.join(include);
    let include_name = include.rsplit('/').next().unwrap_or("");
    if is_ignored_discovery_name(include_name) {
        skipped.push(include.to_string());
        return Ok(());
    }
    if path.is_file() {
        read_discovery_file(include.to_string(), &path, snapshots, skipped);
        return Ok(());
    }
    if path.is_dir() {
        collect_discovery_dir(root, include, snapshots, skipped)?;
        return Ok(());
    }
    skipped.push(include.to_string());
    Ok(())
}

fn collect_discovery_dir(
    root: &Path,
    include: &str,
    snapshots: &mut Vec<DiscoverySnapshot>,
    skipped: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let mut pending = vec![include.to_string()];
    while let Some(relative_dir) = pending.pop() {
        let entries = match fs::read_dir(root.join(&relative_dir)) {
            Ok(entries) => entries,
            Err(_) => {
                skipped.push(relative_dir);
                continue;
            }
        };
        let mut children = Vec::new();
        for entry in entries.flatten() {
            let child = entry.path();
            children.push(safe_relative_path(root, &child));
        }
        children.sort();
        for child in children.into_iter().rev() {
            let name = child.rsplit('/').next().unwrap_or("");
            if is_ignored_discovery_name(name) {
                skipped.push(child);
                continue;
            }
            let path = root.join(&child);
            if path.is_dir() {
                pending.push(child);
            } else if path.is_file() && is_discovery_text_path(&child) {
                read_discovery_file(child, &path, snapshots, skipped);
            } else {
                skipped.push(child);
            }
        }
    }
    skipped.sort();
    skipped.dedup();
    Ok(())
}

fn read_discovery_file(
    relative: String,
    path: &Path,
    snapshots: &mut Vec<DiscoverySnapshot>,
    skipped: &mut Vec<String>,
) {
    match fs::read_to_string(path) {
        Ok(content) => snapshots.push(DiscoverySnapshot {
            path: relative,
            content,
        }),
        Err(_) => skipped.push(relative),
    }
}

fn is_safe_discovery_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with("\\")
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}

fn is_ignored_discovery_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".hg" | ".svn" | "target" | "node_modules" | "vendor" | "dist" | "build"
    ) || name.starts_with('.')
}

fn is_discovery_text_path(path: &str) -> bool {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");
    matches!(
        extension,
        "rs" | "md"
            | "txt"
            | "toml"
            | "ncl"
            | "json"
            | "yaml"
            | "yml"
            | "ts"
            | "tsx"
            | "js"
            | "jsx"
            | "py"
            | "go"
    )
}

fn merge_skipped(mut left: Vec<String>, mut right: Vec<String>) -> Vec<String> {
    left.append(&mut right);
    left.sort();
    left.dedup();
    left
}

fn candidate_json(candidate: &cairn_core::DiscoveryCandidate) -> Value {
    json!({
        "id": candidate.id,
        "evidence_path": candidate.evidence_path,
        "summary": candidate.summary,
        "review_status": candidate.review_status,
    })
}

fn discovery_draft(candidates: &[Value]) -> String {
    let mut output = String::from(
        "# Reverse Spec Discovery Draft

",
    );
    for candidate in candidates {
        output.push_str(&format!(
            "- `{}` from `{}`: {} ({})
",
            candidate["id"].as_str().unwrap_or("candidate"),
            candidate["evidence_path"].as_str().unwrap_or("unknown"),
            candidate["summary"].as_str().unwrap_or("candidate"),
            candidate["review_status"]
                .as_str()
                .unwrap_or("review-needed"),
        ));
    }
    output
}
