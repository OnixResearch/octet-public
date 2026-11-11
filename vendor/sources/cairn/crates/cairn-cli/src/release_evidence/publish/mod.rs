// r[impl release_evidence.cli_source_split.publish_module]
use std::error::Error;

use cairn_core::canonical_json_hash;
use serde_json::Value;

use crate::agent_receipt::AGENT_RECEIPT_MAX_BYTES;
use crate::evidence::{evidence_index_verify_receipt, EvidenceIndexVerifyOptions};
use crate::support::{print_json, CapabilityRoot, CapabilityRootKind};

use super::common::{collect_invalid_receipt, git_head_state, read_file_hash};

mod labels;
mod manifest;
mod materialize;
mod options;
mod paths;
mod receipt;

pub(super) use labels::validate_publish_label;
pub(super) use materialize::copy_publish_artifact;

// r[impl cli_maintainability.release_evidence_publish_submodule_split.focused_modules]
// r[impl cli_maintainability.release_evidence_publish_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.release_evidence_publish_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.release_evidence_publish_submodule_split.behavior_preserved]
// r[impl cli_maintainability.release_evidence_publish_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.release_evidence_publish_submodule_split.behavior_preserved.repository_checks]

use manifest::release_evidence_publish_manifest;
use materialize::copy_publish_file;
use options::{release_evidence_publish_options, ReleaseEvidencePublishOptions};
use paths::evidence_publish_source_bundle_dir;
use receipt::{release_evidence_publish_final_receipt, write_release_evidence_publish_receipt};

pub(super) fn run_release_evidence_publish(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = release_evidence_publish_options(&args)?;
    let receipt = release_evidence_publish_receipt(&options);
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

fn release_evidence_publish_receipt(options: &ReleaseEvidencePublishOptions) -> Value {
    // r[impl release_evidence.publish_materialization.cli_contract]
    // r[impl release_evidence.publish_materialization.cli_contract.inputs_outputs]
    // r[impl release_evidence.publish_materialization.composes_existing_validators]
    // r[impl release_evidence.publish_materialization.composes_existing_validators.source_evidence]
    // r[impl release_evidence.publish_materialization.manifest_receipt]
    // r[impl release_evidence.publish_materialization.manifest_receipt.artifact_binding]
    // r[impl release_evidence.publish_materialization.manifest_receipt.metadata]
    // r[impl release_evidence.publish_materialization.fail_closed]
    // r[impl release_evidence.publish_materialization.fail_closed.path_or_hash]
    // r[impl release_evidence.publish_materialization.operator_docs]
    // r[impl release_evidence.publish_materialization.operator_docs.non_claims]
    let mut issues = Vec::new();
    let source_index_hash = read_file_hash(&options.index, &mut issues, "index", "source_index");
    let index_receipt_path = options.artifact_dir.join("release-evidence-index.json");
    let source_index_receipt: Option<Value> =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Release, &index_receipt_path, false)
            .and_then(|(index_root, relative)| {
                index_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
            })
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    if source_index_receipt.is_none() {
        issues.push(super::common::release_evidence_issue(
            "release_evidence_publish.source_index_receipt_unreadable",
            "artifact_dir/release-evidence-index.json",
            "readable evidence index receipt",
            "unreadable",
        ));
    }
    let index_verify = evidence_index_verify_receipt(&EvidenceIndexVerifyOptions {
        root: options.root.clone(),
        index: Some(index_receipt_path),
    });
    collect_invalid_receipt("publish_source_index_verify", &index_verify, &mut issues);
    validate_publish_label("channel", options.channel.as_deref(), &mut issues);
    validate_publish_label("release", options.release.as_deref(), &mut issues);
    validate_publish_label("retention", options.retention.as_deref(), &mut issues);

    let artifact_entries = source_index_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("artifacts"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut materialized = Vec::new();
    let publish_index_path = options.publish_dir.join("index.md");
    if issues.is_empty() {
        copy_publish_file(
            &options.index,
            &publish_index_path,
            "release_evidence_index",
            source_index_hash.as_deref(),
            &options.publish_dir,
            &mut materialized,
            &mut issues,
        );
    }
    let source_bundle_dir =
        evidence_publish_source_bundle_dir(&options.artifact_dir, source_index_receipt.as_ref());
    if issues.is_empty() {
        for artifact in &artifact_entries {
            copy_publish_artifact(
                artifact,
                &source_bundle_dir,
                &options.publish_dir,
                &mut materialized,
                &mut issues,
            );
        }
    }
    materialized.sort_by_key(canonical_json_hash);
    let payload_commit = git_head_state(&options.root, &mut Vec::new())
        .get("payload_commit")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let manifest = release_evidence_publish_manifest(
        options,
        payload_commit,
        source_index_hash.clone().unwrap_or_default(),
        source_index_receipt
            .as_ref()
            .and_then(|receipt| receipt.get("receipt_hash"))
            .cloned()
            .unwrap_or(Value::Null),
        materialized.clone(),
    );
    let mut manifest_hash = None;
    if issues.is_empty() {
        let manifest_path = options.publish_dir.join("manifest.json");
        super::common::write_json_file(&manifest_path, &manifest, &mut issues, "manifest");
        manifest_hash = read_file_hash(&manifest_path, &mut issues, "manifest", "manifest");
    }
    let receipt = release_evidence_publish_final_receipt(
        options,
        issues,
        source_index_hash,
        source_index_receipt
            .as_ref()
            .and_then(|receipt| receipt.get("receipt_hash"))
            .cloned()
            .unwrap_or(Value::Null),
        index_verify
            .get("receipt_hash")
            .cloned()
            .unwrap_or(Value::Null),
        manifest_hash,
        materialized,
    );
    write_release_evidence_publish_receipt(options, receipt)
}
