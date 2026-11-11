// r[impl release_evidence.cli_source_split.rotate_module]
use std::{error::Error, path::PathBuf};

use cairn_core::canonical_json_hash;
use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{
    path_string, print_json, read_option, read_string_option, with_portable_content_hash_receipt,
    CapabilityRoot, CapabilityRootKind,
};

use super::common::{
    directory_manifest_hash, read_file_hash, release_evidence_issue, safe_relative_path_text,
};

pub(super) fn run_release_evidence_rotate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = release_evidence_rotate_options(&args)?;
    let receipt = release_evidence_rotate_receipt(&options);
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

pub(super) struct ReleaseEvidenceRotateOptions {
    pub(super) root: PathBuf,
    pub(super) plan: PathBuf,
    pub(super) execute: bool,
}

fn release_evidence_rotate_options(
    args: &[String],
) -> Result<ReleaseEvidenceRotateOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let Some(plan) = read_string_option(args, "--plan").map(PathBuf::from) else {
        return Err("release-evidence rotate requires --plan <file>".into());
    };
    Ok(ReleaseEvidenceRotateOptions {
        root,
        plan,
        execute: args.iter().any(|arg| arg == "--execute"),
    })
}

pub(super) fn release_evidence_rotate_receipt(options: &ReleaseEvidenceRotateOptions) -> Value {
    // r[impl release_evidence.retention_rotation.rotation_execution]
    // r[impl release_evidence.retention_rotation.rotation_execution.verify_before_mutation]
    // r[impl release_evidence.retention_rotation.fail_closed]
    let mut issues = Vec::new();
    let plan_hash =
        read_file_hash(&options.plan, &mut issues, "plan", "rotation_plan").unwrap_or_default();
    let plan: Option<Value> =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Release, &options.plan, false)
            .and_then(|(plan_root, relative)| {
                plan_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
            })
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    if plan.is_none() {
        issues.push(release_evidence_issue(
            "release_evidence_rotation.plan_unreadable",
            "plan",
            "readable retention plan JSON",
            "unreadable",
        ));
    }
    let publish_root = plan
        .as_ref()
        .and_then(|plan| plan.get("publish_root"))
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_default();
    let publish_capability = CapabilityRoot::open(CapabilityRootKind::Release, &publish_root);
    if options.execute && publish_capability.is_err() {
        issues.push(release_evidence_issue(
            "release_evidence_rotation.publish_root_unreadable",
            "publish_root",
            "readable publication root",
            "unreadable",
        ));
    }
    let mut mutations = Vec::new();
    if let Some(plan) = &plan {
        for decision in plan
            .get("decisions")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if decision.get("action").and_then(Value::as_str) != Some("prune_candidate") {
                continue;
            }
            let relative = decision.get("path").and_then(Value::as_str).unwrap_or("");
            if !safe_relative_path_text(relative) {
                issues.push(release_evidence_issue(
                    "release_evidence_rotation.path_unsafe",
                    "decision.path",
                    "safe relative prune path",
                    "unsafe",
                ));
                continue;
            }
            let target = publish_root.join(relative);
            let before_hash = directory_manifest_hash(&target).unwrap_or_default();
            if options.execute && issues.is_empty() {
                let remove_result = publish_capability
                    .as_ref()
                    .map_err(|error| error.kind())
                    .and_then(|capability| {
                        capability
                            .remove_dir_all(PathBuf::from(relative).as_path())
                            .map_err(|error| error.kind())
                    });
                if remove_result.is_err() {
                    issues.push(release_evidence_issue(
                        "release_evidence_rotation.remove_failed",
                        relative,
                        "removable local publication tree",
                        "failed",
                    ));
                }
            }
            let after_hash = directory_manifest_hash(&target).unwrap_or_default();
            mutations.push(json!({
                "path": relative,
                "action": if options.execute { "removed" } else { "would_remove" },
                "before_hash": before_hash,
                "after_hash": after_hash,
            }));
        }
    }
    mutations.sort_by_key(canonical_json_hash);
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        &options.root,
        json!({
            "command": "release-evidence rotate",
            "root": path_string(&options.root),
            "plan": path_string(&options.plan),
            "execute": options.execute,
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "plan_hash": plan_hash,
            "mutation_count": mutations.len(),
            "mutation_manifest": mutations,
            "issues": issues,
        }),
    )
}
