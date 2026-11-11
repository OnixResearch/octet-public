use std::{error::Error, fs, path::Path};

use cairn_core::{blake3_hex, plan_prompt_scaffold, SCAFFOLD_POLICY_ID};
use serde_json::{json, Value};

use crate::developer_experience::common::write_planned_files;
use crate::support::{
    file_content_manifest, has_flag, path_string, portable_json_hash, print_json, read_option,
    read_string_option, with_portable_receipt_hash,
};

pub(super) fn run_change(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.developer_experience.prompt_to_change_scaffold.cli]
    let root = read_option(args.clone(), "--root", ".");
    let execute = has_flag(&args, "--execute");
    let domain =
        read_string_option(&args, "--domain").unwrap_or_else(|| "developer-experience".to_string());
    let request = read_request(&args)?;
    let existing = existing_change_paths(&root, change);
    let plan = plan_prompt_scaffold(change, &domain, &request, &existing);
    let mut issues = plan.issues.clone();
    let touched = plan
        .files
        .iter()
        .map(|file| root.join(&file.path))
        .collect::<Vec<_>>();
    let before = file_content_manifest(&root, &touched);
    let mut written = Vec::new();
    if execute && plan.valid {
        write_planned_files(&root, &plan.files, &existing, &mut issues, &mut written)?;
    }
    let after = file_content_manifest(&root, &touched);
    let artifact_plan = plan
        .files
        .iter()
        .map(|file| {
            json!({
                "path": file.path,
                "content_hash": blake3_hex(file.content.as_bytes()),
                "exists": existing.iter().any(|path| path == &file.path),
            })
        })
        .collect::<Vec<_>>();
    print_json(with_portable_receipt_hash(
        &root,
        json!({
            "command": "scaffold_change",
            "root": path_string(&root),
            "change": plan.change,
            "domain": plan.domain,
            "dry_run": !execute,
            "valid": issues.is_empty(),
            "policy": SCAFFOLD_POLICY_ID,
            "issues": issues,
            "input_hash": plan.input_hash,
            "artifacts": artifact_plan,
            "mutated": execute && !written.is_empty(),
            "written": written,
            "mutation_manifest": if execute { json!({"before": before, "after": after, "manifest_hash": portable_json_hash(&root, &after)}) } else { Value::Null },
        }),
    ))
}

fn read_request(args: &[String]) -> Result<String, Box<dyn Error>> {
    if let Some(request) = read_string_option(args, "--request") {
        return Ok(request);
    }
    if let Some(path) = read_string_option(args, "--request-file") {
        return Ok(fs::read_to_string(path)?);
    }
    Ok(String::new())
}

fn existing_change_paths(root: &Path, change: &str) -> Vec<String> {
    let prefix = format!(".cairn/changes/{change}");
    ["proposal.md", "design.md", "tasks.md"]
        .into_iter()
        .map(|name| format!("{prefix}/{name}"))
        .filter(|path| root.join(path).exists())
        .collect()
}
