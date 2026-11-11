use super::*;
use crate::support::{CapabilityRoot, CapabilityRootKind};

mod agent_receipt;
mod artifacts;
mod issue;
mod manifest;
mod options;
mod receipt;

pub(crate) use options::EvidenceExportOptions;
pub(crate) use receipt::evidence_export_final_receipt;
pub(crate) use receipt::write_evidence_json;

// r[impl cli_maintainability.evidence_export_submodule_split.focused_modules]
// r[impl cli_maintainability.evidence_export_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_export_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.evidence_export_submodule_split.behavior_preserved]
// r[impl cli_maintainability.evidence_export_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.evidence_export_submodule_split.behavior_preserved.repository_checks]

use agent_receipt::export_agent_receipt_bound_to_replay;
use artifacts::evidence_export_artifacts;
use issue::{evidence_export_issue, evidence_export_nested_issue};
use manifest::evidence_manifest_with_relative_paths;
use options::evidence_export_options;

pub(super) fn run_evidence_export(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.evidence_export_bundle.command]
    // r[impl cairn.supersede.evidence_export_bundle.artifacts]
    // r[impl cairn.supersede.evidence_export_bundle.failures]
    let options = evidence_export_options(&args)?;
    let receipt = evidence_export_receipt(&options);
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

pub(crate) fn evidence_export_receipt(options: &EvidenceExportOptions) -> Value {
    let root = options.root.clone();
    let out_dir = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let output_root =
        match CapabilityRoot::open_or_create(CapabilityRootKind::EvidenceBundle, &out_dir) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(evidence_export_issue(
                    "export.out_dir_create_failed",
                    "out_dir",
                    "writable output directory",
                    "failed",
                ));
                return evidence_export_final_receipt(
                    &root,
                    &out_dir,
                    issues,
                    Vec::new(),
                    Value::Null,
                    Value::Null,
                );
            }
        };
    let repository_root = match CapabilityRoot::open(CapabilityRootKind::Repository, &root) {
        Ok(capability) => capability,
        Err(_) => {
            issues.push(evidence_export_issue(
                "export.repository_root_open_failed",
                "root",
                "readable repository root",
                "failed",
            ));
            return evidence_export_final_receipt(
                &root,
                &out_dir,
                issues,
                Vec::new(),
                Value::Null,
                Value::Null,
            );
        }
    };

    let policy = match load_policy_file(default_policy_path().into()) {
        Ok(policy) => policy,
        Err(_) => {
            issues.push(evidence_export_issue(
                "export.policy_load_failed",
                "policy",
                "generated policy loads",
                "failed",
            ));
            return evidence_export_final_receipt(
                &root,
                &out_dir,
                issues,
                Vec::new(),
                Value::Null,
                Value::Null,
            );
        }
    };

    let agent_path = options
        .agent_receipt
        .clone()
        .unwrap_or_else(|| root.join("fixtures/agent-receipts/positive.json"));
    let coverage_path = out_dir.join("coverage-audit.json");
    let replay_path = out_dir.join("replay-agent.json");
    let replay_verification_path = out_dir.join("replay-verification.json");
    let release_path = out_dir.join("release-readiness.json");
    let runtime_chain_path = out_dir.join("runtime-chain.json");
    let witness_checkpoint_path = out_dir.join("witness-checkpoint.json");
    let runtime_evidence_path = out_dir.join("runtime-evidence.json");
    let exported_agent_path = out_dir.join("agent-receipt.json");
    let bundle_path = out_dir.join("agent-evidence-bundle.json");
    let bundle_verification_path = out_dir.join("agent-evidence-bundle-verification.json");
    let manifest_path = out_dir.join("evidence-manifest.json");
    let manifest_verification_path = out_dir.join("evidence-manifest-verification.json");

    let coverage = determinism_coverage_audit_receipt(&root, &policy);
    write_evidence_json(
        &output_root,
        &coverage_path,
        &coverage,
        &mut issues,
        "coverage_audit",
    );

    let replay =
        match determinism_replay_receipt(root.clone(), None, Some(REQUIRED_AGENT_REPLAY_GROUP)) {
            Ok(receipt) => receipt,
            Err(_) => {
                issues.push(evidence_export_issue(
                    "export.replay_failed",
                    "replay_receipt",
                    "valid replay receipt",
                    "failed",
                ));
                Value::Null
            }
        };
    if replay.is_object() {
        write_evidence_json(
            &output_root,
            &replay_path,
            &replay,
            &mut issues,
            "replay_receipt",
        );
    }

    let replay_verification = if replay.is_object() {
        write_evidence_json(
            &output_root,
            &replay_path,
            &replay,
            &mut issues,
            "replay_receipt",
        );
        determinism_verify_replay_receipt(&root, &replay_path)
    } else {
        Value::Null
    };
    if replay_verification.is_object() {
        write_evidence_json(
            &output_root,
            &replay_verification_path,
            &replay_verification,
            &mut issues,
            "replay_verification",
        );
    }

    let release = release_readiness_receipt(root.clone(), default_policy_path().into());
    write_evidence_json(
        &output_root,
        &release_path,
        &release,
        &mut issues,
        "release_readiness",
    );

    let exported_runtime_chain = copy_optional_runtime_artifact(
        &repository_root,
        &output_root,
        options.runtime_chain.as_ref(),
        &runtime_chain_path,
        &mut issues,
        "runtime_chain",
    );
    let exported_witness_checkpoint = copy_optional_runtime_artifact(
        &repository_root,
        &output_root,
        options.witness_checkpoint.as_ref(),
        &witness_checkpoint_path,
        &mut issues,
        "witness_checkpoint",
    );
    let runtime_evidence = if exported_runtime_chain.is_some() {
        let receipt = runtime_evidence_verification_receipt(&RuntimeEvidenceVerifyOptions {
            root: root.clone(),
            policy: default_policy_path().into(),
            chain: Some(runtime_chain_path.clone()),
            checkpoint: exported_witness_checkpoint
                .as_ref()
                .map(|_| witness_checkpoint_path.clone()),
        });
        write_evidence_json(
            &output_root,
            &runtime_evidence_path,
            &receipt,
            &mut issues,
            "runtime_evidence",
        );
        receipt
    } else {
        Value::Null
    };

    let exported_agent_receipt = if options.agent_receipt.is_some() {
        match repository_root.copy_path_to(&agent_path, &output_root, &exported_agent_path) {
            Ok(_) => output_root
                .read_path(&exported_agent_path)
                .ok()
                .filter(|bytes| {
                    u64::try_from(bytes.len()).unwrap_or(u64::MAX) <= AGENT_RECEIPT_MAX_BYTES
                })
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or(Value::Null),
            Err(_) => {
                issues.push(evidence_export_issue(
                    "export.agent_receipt_copy_failed",
                    "agent_receipt",
                    "readable agent receipt fixture",
                    "failed",
                ));
                Value::Null
            }
        }
    } else {
        match export_agent_receipt_bound_to_replay(
            &repository_root,
            &output_root,
            &agent_path,
            &exported_agent_path,
            &replay_verification,
            &policy,
        ) {
            Ok(receipt) => receipt,
            Err(_) => {
                issues.push(evidence_export_issue(
                    "export.agent_receipt_generate_failed",
                    "agent_receipt",
                    "agent receipt bound to exported replay evidence",
                    "failed",
                ));
                Value::Null
            }
        }
    };

    let mut bundle = json!({
        "agent_receipt_path": "agent-receipt.json",
        "replay_receipt_path": "replay-agent.json",
        "replay_verification_path": "replay-verification.json",
        "release_readiness_path": "release-readiness.json",
        "coverage_audit_path": "coverage-audit.json",
        "agent_receipt_hash": receipt_stated_or_computed_hash(&root, &exported_agent_receipt),
        "replay_receipt_hash": receipt_stated_or_computed_hash(&root, &replay),
        "replay_verification_receipt_hash": receipt_stated_or_computed_hash(&root, &replay_verification),
        "release_readiness_receipt_hash": receipt_stated_or_computed_hash(&root, &release),
        "coverage_audit_receipt_hash": receipt_stated_or_computed_hash(&root, &coverage),
    });
    if runtime_evidence.is_object() {
        bundle["runtime_evidence_path"] = json!("runtime-evidence.json");
        bundle["runtime_evidence_receipt_hash"] =
            receipt_stated_or_computed_hash(&root, &runtime_evidence);
    }
    write_evidence_json(
        &output_root,
        &bundle_path,
        &bundle,
        &mut issues,
        "agent_evidence_bundle",
    );

    let bundle_verification = agent_evidence_bundle_verification_receipt(&root, &bundle_path);
    write_evidence_json(
        &output_root,
        &bundle_verification_path,
        &bundle_verification,
        &mut issues,
        "agent_evidence_bundle_verification",
    );

    let manifest_options = EvidenceManifestGenerateOptions {
        root: root.clone(),
        bundle: Some(bundle_path.clone()),
        agent_receipt: Some(exported_agent_path.clone()),
        replay: Some(replay_path.clone()),
        replay_verification: Some(replay_verification_path.clone()),
        release_readiness: Some(release_path.clone()),
        coverage_audit: Some(coverage_path.clone()),
        requirement_registry: None,
        registry: None,
        registry_verification: None,
        registry_selection: None,
        registry_fetch_plan: None,
        runtime_evidence: runtime_evidence
            .is_object()
            .then_some(runtime_evidence_path.clone()),
        runtime_chain: exported_runtime_chain
            .as_ref()
            .map(|_| runtime_chain_path.clone()),
        witness_checkpoint: exported_witness_checkpoint
            .as_ref()
            .map(|_| witness_checkpoint_path.clone()),
        signer_metadata: None,
        anchor_metadata: None,
        out: None,
    };
    let (manifest, manifest_issues) = evidence_manifest_generate_manifest(&manifest_options);
    issues.extend(
        manifest_issues
            .into_iter()
            .map(|issue| evidence_export_nested_issue("export.manifest_generate_failed", issue)),
    );
    let manifest = manifest.unwrap_or(Value::Null);
    if manifest.is_object() {
        let relative_manifest = evidence_manifest_with_relative_paths(&manifest, &out_dir);
        write_evidence_json(
            &output_root,
            &manifest_path,
            &relative_manifest,
            &mut issues,
            "evidence_manifest",
        );
    }

    let manifest_verification = evidence_manifest_verification_receipt(&root, &manifest_path);
    write_evidence_json(
        &output_root,
        &manifest_verification_path,
        &manifest_verification,
        &mut issues,
        "evidence_manifest_verification",
    );

    if bundle_verification.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_export_issue(
            "export.bundle_verification_failed",
            "agent_evidence_bundle_verification",
            "valid bundle verification receipt",
            "invalid",
        ));
    }
    if manifest_verification.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_export_issue(
            "export.manifest_verification_failed",
            "evidence_manifest_verification",
            "valid manifest verification receipt",
            "invalid",
        ));
    }

    let mut artifacts = evidence_export_artifacts(&output_root, &out_dir, &mut issues);
    artifacts.sort_by_key(|artifact| {
        artifact
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    let manifest_hash = manifest_verification
        .get("manifest_hash")
        .cloned()
        .unwrap_or(Value::Null);
    let manifest_verification_hash = receipt_stated_or_computed_hash(&root, &manifest_verification);
    let receipt = evidence_export_final_receipt(
        &root,
        &out_dir,
        issues,
        artifacts,
        manifest_hash,
        manifest_verification_hash,
    );
    write_evidence_json(
        &output_root,
        &out_dir.join("evidence-export-receipt.json"),
        &receipt,
        &mut Vec::new(),
        "evidence_export_receipt",
    );
    receipt
}

fn copy_optional_runtime_artifact(
    repository_root: &CapabilityRoot,
    output_root: &CapabilityRoot,
    source: Option<&PathBuf>,
    destination: &Path,
    issues: &mut Vec<Value>,
    role: &str,
) -> Option<()> {
    let source = source?;
    match repository_root.copy_path_to(source, output_root, destination) {
        Ok(_) => Some(()),
        Err(_) => {
            issues.push(evidence_export_issue(
                "export.runtime_artifact_copy_failed",
                role,
                "readable runtime evidence artifact",
                "failed",
            ));
            None
        }
    }
}
