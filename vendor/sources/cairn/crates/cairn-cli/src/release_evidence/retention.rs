// r[impl release_evidence.cli_source_split.retention_module]
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::canonical_json_hash;
use serde_json::{json, Value};

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::support::{
    path_string, print_json, read_option, read_string_option, with_portable_content_hash_receipt,
    CapabilityRoot, CapabilityRootKind,
};

use super::common::{
    read_file_hash, release_evidence_issue, safe_relative_path_text, write_json_file,
};
use super::publish::validate_publish_label;

const RETENTION_PUBLICATION_SCAN_MAX_DEPTH: usize = 8;

pub(super) fn run_release_evidence_retention_plan(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = release_evidence_retention_options(&args)?;
    let receipt = release_evidence_retention_plan_receipt(&options);
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

pub(super) struct ReleaseEvidenceRetentionOptions {
    pub(super) root: PathBuf,
    pub(super) publish_root: PathBuf,
    pub(super) out: PathBuf,
    pub(super) channel: Option<String>,
    pub(super) keep_latest: usize,
    pub(super) retain: Option<String>,
}

fn release_evidence_retention_options(
    args: &[String],
) -> Result<ReleaseEvidenceRetentionOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let Some(publish_root) = read_string_option(args, "--publish-root").map(PathBuf::from) else {
        return Err("release-evidence retention-plan requires --publish-root <dir>".into());
    };
    let Some(out) = read_string_option(args, "--out").map(PathBuf::from) else {
        return Err("release-evidence retention-plan requires --out <file>".into());
    };
    let keep_latest = read_string_option(args, "--keep-latest")
        .map(|value| value.parse::<usize>())
        .transpose()
        .map_err(|_| "--keep-latest must be a non-negative integer")?
        .unwrap_or(1);
    Ok(ReleaseEvidenceRetentionOptions {
        root,
        publish_root,
        out,
        channel: read_string_option(args, "--channel"),
        keep_latest,
        retain: read_string_option(args, "--retain"),
    })
}

pub(super) fn release_evidence_retention_plan_receipt(
    options: &ReleaseEvidenceRetentionOptions,
) -> Value {
    // r[impl release_evidence.retention_rotation.cli_contract]
    // r[impl release_evidence.retention_rotation.cli_contract.inputs_outputs]
    // r[impl release_evidence.retention_rotation.composes_publication_validators]
    // r[impl release_evidence.retention_rotation.composes_publication_validators.manifest_receipt]
    // r[impl release_evidence.retention_rotation.plan_receipt]
    // r[impl release_evidence.retention_rotation.plan_receipt.decisions]
    // r[impl release_evidence.retention_rotation.plan_receipt.ordering]
    // r[impl release_evidence.retention_rotation.fail_closed]
    // r[impl release_evidence.retention_rotation.fail_closed.unsafe_or_stale]
    // r[impl release_evidence.retention_rotation.operator_docs]
    // r[impl release_evidence.retention_rotation.operator_docs.non_claims]
    let mut issues = Vec::new();
    validate_publish_label("channel", options.channel.as_deref(), &mut issues);
    validate_publish_label("retain", options.retain.as_deref(), &mut issues);
    let mut candidates = retention_candidates(options, &mut issues);
    let mut seen_current = std::collections::BTreeSet::new();
    for candidate in &mut candidates {
        if candidate.get("valid").and_then(Value::as_bool) != Some(true) {
            candidate["action"] = json!("invalid");
            continue;
        }
        let release = candidate["labels"]
            .get("release")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let channel = candidate["labels"]
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let retention = candidate["labels"]
            .get("retention")
            .and_then(Value::as_str)
            .unwrap_or("");
        if options
            .channel
            .as_deref()
            .is_some_and(|wanted| wanted != channel)
        {
            candidate["action"] = json!("keep");
            candidate["reason"] = json!("outside_selected_channel");
        } else if options
            .retain
            .as_deref()
            .is_some_and(|wanted| wanted == retention)
        {
            candidate["action"] = json!("keep");
            candidate["reason"] = json!("retention_label_matched");
        } else if !seen_current.insert((channel.clone(), release.clone())) {
            candidate["action"] = json!("invalid");
            candidate["reason"] = json!("duplicate_channel_release");
            issues.push(release_evidence_issue(
                "release_evidence_retention.duplicate_channel_release",
                "labels.channel_release",
                "unique channel/release identity",
                "duplicate",
            ));
        }
    }
    let mut sortable: Vec<(String, usize)> = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| {
            let channel = candidate["labels"]
                .get("channel")
                .and_then(Value::as_str)
                .unwrap_or("");
            let release = candidate["labels"]
                .get("release")
                .and_then(Value::as_str)
                .unwrap_or("");
            (
                format!(
                    "{channel}\u{0}{release}\u{0}{}",
                    candidate["path"].as_str().unwrap_or("")
                ),
                index,
            )
        })
        .collect();
    sortable.sort_by(|left, right| right.0.cmp(&left.0));
    let mut kept_by_channel = std::collections::BTreeMap::<String, usize>::new();
    for (_, index) in sortable {
        if candidates[index]
            .get("action")
            .and_then(Value::as_str)
            .is_some()
        {
            continue;
        }
        let channel = candidates[index]["labels"]
            .get("channel")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let kept = kept_by_channel.entry(channel).or_default();
        if *kept < options.keep_latest {
            candidates[index]["action"] = json!("keep");
            candidates[index]["reason"] = json!("within_keep_latest");
            *kept += 1;
        } else {
            candidates[index]["action"] = json!("prune_candidate");
            candidates[index]["reason"] = json!("superseded_by_newer_release");
        }
    }
    candidates.sort_by_key(canonical_json_hash);
    let plan = json!({
        "schema": "cairn.release-evidence.retention-plan.v1",
        "publish_root": path_string(&options.publish_root),
        "selectors": {
            "channel": options.channel,
            "keep_latest": options.keep_latest,
            "retain": options.retain,
        },
        "decisions": candidates,
        "non_claims": [
            "local_retention_plan_only",
            "no_remote_deletion",
            "no_credential_validation",
            "no_remote_mirror_trust",
            "no_signature_validation",
            "no_tag_movement",
            "no_public_release_page_mutation",
        ],
    });
    let mut plan_hash = None;
    if issues.is_empty() {
        write_json_file(&options.out, &plan, &mut issues, "retention_plan");
        plan_hash = read_file_hash(
            &options.out,
            &mut issues,
            "retention_plan",
            "retention_plan",
        );
    }
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        &options.root,
        json!({
            "command": "release-evidence retention-plan",
            "root": path_string(&options.root),
            "publish_root": path_string(&options.publish_root),
            "out": path_string(&options.out),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "plan_hash": plan_hash.unwrap_or_default(),
            "decision_count": plan["decisions"].as_array().map(Vec::len).unwrap_or(0),
            "decisions": plan["decisions"].clone(),
            "issues": issues,
        }),
    )
}

fn retention_candidates(
    options: &ReleaseEvidenceRetentionOptions,
    issues: &mut Vec<Value>,
) -> Vec<Value> {
    let publish_root =
        match CapabilityRoot::open(CapabilityRootKind::Release, &options.publish_root) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(release_evidence_issue(
                    "release_evidence_retention.publish_root_unreadable",
                    "publish_root",
                    "readable publication root",
                    "unreadable",
                ));
                return Vec::new();
            }
        };
    let mut candidates = Vec::new();
    collect_retention_candidates(&publish_root, Path::new("."), 0, &mut candidates, issues);
    candidates
}

fn collect_retention_candidates(
    publish_root: &CapabilityRoot,
    current: &Path,
    depth: usize,
    candidates: &mut Vec<Value>,
    issues: &mut Vec<Value>,
) {
    if depth > RETENTION_PUBLICATION_SCAN_MAX_DEPTH {
        issues.push(release_evidence_issue(
            "release_evidence_retention.scan_depth_exceeded",
            "publish_root",
            "bounded publication tree depth",
            "too_deep",
        ));
        return;
    }
    let Ok(mut entries) = publish_root.entries(current) else {
        issues.push(release_evidence_issue(
            "release_evidence_retention.directory_unreadable",
            &current.to_string_lossy().replace('\\', "/"),
            "readable publication directory",
            "unreadable",
        ));
        return;
    };
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        let relative_path = if current == Path::new(".") {
            PathBuf::from(entry.file_name())
        } else {
            current.join(entry.file_name())
        };
        let relative = relative_path.to_string_lossy().replace('\\', "/");
        if !safe_relative_path_text(&relative) {
            issues.push(release_evidence_issue(
                "release_evidence_retention.path_unsafe",
                "candidate.path",
                "safe relative candidate path",
                "unsafe",
            ));
            continue;
        }
        if entry.file_type().is_symlink() {
            issues.push(release_evidence_issue(
                "release_evidence_retention.symlink_rejected",
                &relative,
                "real publication directory",
                "symlink",
            ));
            continue;
        }
        if !entry.file_type().is_dir() {
            continue;
        }
        let manifest_path = relative_path.join("manifest.json");
        let receipt_path = relative_path.join("receipt.json");
        let has_manifest = capability_regular_file_exists(publish_root, &manifest_path);
        let has_receipt = capability_regular_file_exists(publish_root, &receipt_path);
        if has_manifest || has_receipt {
            candidates.push(read_retention_candidate(
                publish_root,
                &relative_path,
                &relative,
                issues,
            ));
        } else {
            collect_retention_candidates(
                publish_root,
                &relative_path,
                depth + 1,
                candidates,
                issues,
            );
        }
    }
}

fn capability_regular_file_exists(root: &CapabilityRoot, relative: &Path) -> bool {
    root.symlink_metadata(relative)
        .is_ok_and(|metadata| metadata.is_file() && !metadata.file_type().is_symlink())
}

fn read_retention_candidate(
    publish_root: &CapabilityRoot,
    relative_directory: &Path,
    relative: &str,
    issues: &mut Vec<Value>,
) -> Value {
    let manifest_relative = relative_directory.join("manifest.json");
    let receipt_relative = relative_directory.join("receipt.json");
    let manifest_path = publish_root
        .display_path(&manifest_relative)
        .unwrap_or_else(|_| publish_root.path().join(&manifest_relative));
    let receipt_path = publish_root
        .display_path(&receipt_relative)
        .unwrap_or_else(|_| publish_root.path().join(&receipt_relative));
    let manifest_hash =
        read_file_hash(&manifest_path, issues, relative, "retention_manifest").unwrap_or_default();
    let receipt_hash =
        read_file_hash(&receipt_path, issues, relative, "retention_receipt").unwrap_or_default();
    let manifest: Option<Value> = publish_root
        .read_bounded(&manifest_relative, AGENT_RECEIPT_MAX_BYTES)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let receipt: Option<Value> = publish_root
        .read_bounded(&receipt_relative, AGENT_RECEIPT_MAX_BYTES)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    let mut valid = true;
    if manifest.is_none() || receipt.is_none() {
        valid = false;
    }
    let stated_manifest_hash = receipt
        .as_ref()
        .and_then(|receipt| receipt.get("manifest_hash"))
        .and_then(Value::as_str)
        .unwrap_or("");
    if !stated_manifest_hash.is_empty() && stated_manifest_hash != manifest_hash {
        valid = false;
        issues.push(release_evidence_issue(
            "release_evidence_retention.manifest_hash_mismatch",
            relative,
            "receipt manifest hash matches manifest.json",
            "mismatch",
        ));
    }
    if receipt
        .as_ref()
        .and_then(|receipt| receipt.get("valid"))
        .and_then(Value::as_bool)
        != Some(true)
    {
        valid = false;
        issues.push(release_evidence_issue(
            "release_evidence_retention.publication_receipt_invalid",
            relative,
            "valid publication receipt",
            "invalid",
        ));
    }
    let labels = manifest
        .as_ref()
        .and_then(|manifest| manifest.get("labels"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    for field in ["channel", "release", "retention"] {
        validate_publish_label(field, labels.get(field).and_then(Value::as_str), issues);
    }
    json!({
        "path": relative,
        "manifest_hash": manifest_hash,
        "receipt_hash": receipt_hash,
        "labels": labels,
        "valid": valid,
    })
}
