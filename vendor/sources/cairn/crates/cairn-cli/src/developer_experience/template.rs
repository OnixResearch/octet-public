use std::{error::Error, path::Path};

use cairn_core::{onboarding_templates, plan_template_application, TEMPLATE_POLICY_ID};
use serde_json::{json, Value};

use crate::developer_experience::common::{existing_paths, write_planned_files};
use crate::support::{
    file_content_manifest, has_flag, path_string, portable_json_hash, print_json, read_option,
    with_portable_receipt_hash,
};

pub(super) fn run_list(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.developer_experience.onboarding_templates.catalog]
    let root = read_option(args, "--root", ".");
    let templates = onboarding_templates()
        .into_iter()
        .map(|template| {
            json!({
                "id": template.id,
                "description": template.description,
                "files": template.files.into_iter().map(|file| file.path).collect::<Vec<_>>(),
            })
        })
        .collect::<Vec<_>>();
    print_json(with_portable_receipt_hash(
        &root,
        json!({
            "command": "template_list",
            "root": path_string(&root),
            "valid": true,
            "policy": TEMPLATE_POLICY_ID,
            "templates": templates,
        }),
    ))
}

pub(super) fn run_apply(args: Vec<String>, template: &str) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.developer_experience.onboarding_templates.apply]
    // r[impl cairn.developer_experience.onboarding_template_catalog_expansion.apply]
    let root = read_option(args.clone(), "--root", ".");
    let execute = has_flag(&args, "--execute");
    let existing = existing_template_paths(&root, template);
    let plan = plan_template_application(template, &existing);
    let files = onboarding_templates()
        .into_iter()
        .find(|candidate| candidate.id == template)
        .map(|profile| profile.files)
        .unwrap_or_default();
    let mut issues = plan.issues.clone();
    let touched = files
        .iter()
        .map(|file| root.join(&file.path))
        .collect::<Vec<_>>();
    let before = file_content_manifest(&root, &touched);
    let mut written = Vec::new();
    if execute && plan.valid {
        write_planned_files(&root, &files, &existing, &mut issues, &mut written)?;
    }
    let after = file_content_manifest(&root, &touched);
    print_json(with_portable_receipt_hash(
        &root,
        json!({
            "command": "template_apply",
            "root": path_string(&root),
            "template": template,
            "dry_run": !execute,
            "valid": issues.is_empty(),
            "policy": TEMPLATE_POLICY_ID,
            "issues": issues,
            "files": plan.files.into_iter().map(|entry| json!({
                "path": entry.path,
                "content_hash": entry.content_hash,
                "exists": entry.exists,
                "action": entry.action,
            })).collect::<Vec<_>>(),
            "mutated": execute && !written.is_empty(),
            "written": written,
            "mutation_manifest": if execute { json!({"before": before, "after": after, "manifest_hash": portable_json_hash(&root, &after)}) } else { Value::Null },
        }),
    ))
}

fn existing_template_paths(root: &Path, template: &str) -> Vec<String> {
    onboarding_templates()
        .into_iter()
        .find(|candidate| candidate.id == template)
        .map(|profile| existing_paths(root, &profile.files))
        .unwrap_or_default()
}
