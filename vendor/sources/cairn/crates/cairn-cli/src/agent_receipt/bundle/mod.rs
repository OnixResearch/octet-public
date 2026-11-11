use std::{error::Error, path::Path};

use cairn_core::canonical_json_hash;
use serde_json::Value;

use crate::support::{default_policy_path, load_policy_file, print_json};

use super::{AGENT_RECEIPT_MAX_BYTES, REQUIRED_AGENT_REPLAY_GROUP};

mod checks;
mod hashes;
mod issues;
mod options;
mod paths;
mod receipt;

pub(crate) use hashes::receipt_stated_or_computed_hash;

// r[impl cli_maintainability.agent_receipt_bundle_submodule_split.focused_modules]
// r[impl cli_maintainability.agent_receipt_bundle_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.agent_receipt_bundle_submodule_split.focused_modules.narrow_bridges]
// r[verify cli_maintainability.agent_receipt_bundle_submodule_split.behavior_preserved]
// r[verify cli_maintainability.agent_receipt_bundle_submodule_split.behavior_preserved.focused_checks]
// r[verify cli_maintainability.agent_receipt_bundle_submodule_split.behavior_preserved.repository_checks]

use checks::{
    compare_agent_replay_evidence, compare_replay_verification, verify_agent_receipt,
    verify_optional_receipt_artifact, verify_replay_receipt, verify_saved_replay_verification,
};
use issues::{agent_bundle_issue, verify_optional_bundle_hash};
use options::agent_evidence_bundle_options;
use paths::{bundle_artifact_path, optional_bundle_artifact_path};
use receipt::agent_bundle_receipt;

pub(super) fn run_agent_receipt_verify_bundle(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = agent_evidence_bundle_options(&args)?;
    let receipt = agent_evidence_bundle_verification_receipt(&options.root, &options.path);
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

pub(crate) fn agent_evidence_bundle_verification_receipt(root: &Path, bundle_path: &Path) -> Value {
    // r[impl cairn.supersede.agent_evidence_bundle_verification.cli]
    // r[impl cairn.supersede.agent_evidence_bundle_verification.validation]
    // r[impl cairn.supersede.agent_evidence_bundle_verification.receipt]
    // r[impl cairn.supersede.agent_evidence_bundle_verification.secret_safe]
    let policy = match load_policy_file(default_policy_path().into()) {
        Ok(policy) => policy,
        Err(_) => {
            return agent_bundle_receipt(
                root,
                vec![agent_bundle_issue(
                    "policy.load_failed",
                    "policy",
                    "generated policy loads",
                    "failed",
                )],
                String::new(),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        }
    };
    let current_policy_hash = canonical_json_hash(&policy.raw);
    let bundle = match super::io::read_json_artifact(bundle_path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(value) => value,
        Err(_) => {
            return agent_bundle_receipt(
                root,
                vec![agent_bundle_issue(
                    "bundle.read_failed",
                    "bundle",
                    "readable JSON object",
                    "unreadable",
                )],
                current_policy_hash,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        }
    };
    let mut issues = Vec::new();
    if !bundle.is_object() {
        issues.push(agent_bundle_issue(
            "bundle.not_object",
            "bundle",
            "object",
            "non-object",
        ));
    }
    let bundle_base = bundle_path.parent().unwrap_or_else(|| Path::new("."));
    let agent_path = bundle_artifact_path(&bundle, bundle_base, "agent_receipt_path", &mut issues);
    let replay_path =
        bundle_artifact_path(&bundle, bundle_base, "replay_receipt_path", &mut issues);
    let replay_verify_path = bundle_artifact_path(
        &bundle,
        bundle_base,
        "replay_verification_path",
        &mut issues,
    );
    let release_path =
        optional_bundle_artifact_path(&bundle, bundle_base, "release_readiness_path", &mut issues);
    let coverage_audit_path =
        optional_bundle_artifact_path(&bundle, bundle_base, "coverage_audit_path", &mut issues);
    let runtime_evidence_path =
        optional_bundle_artifact_path(&bundle, bundle_base, "runtime_evidence_path", &mut issues);

    let mut agent_receipt_hash = Value::Null;
    let mut replay_receipt_hash = Value::Null;
    let mut replay_verification_receipt_hash = Value::Null;
    let mut environment_attestation_hash = Value::Null;
    let mut release_readiness_receipt_hash = Value::Null;
    let mut coverage_audit_receipt_hash = Value::Null;
    let mut runtime_evidence_receipt_hash = Value::Null;

    let mut agent_receipt = Value::Null;
    if let Some(path) = agent_path.as_deref() {
        let (hash, receipt) = verify_agent_receipt(path, &policy, &mut issues);
        agent_receipt_hash = hash;
        agent_receipt = receipt;
    }

    let mut replay_verifier = Value::Null;
    if let Some(path) = replay_path.as_deref() {
        let (hash, attestation_hash, verifier) = verify_replay_receipt(root, path, &mut issues);
        replay_receipt_hash = hash;
        environment_attestation_hash = attestation_hash;
        replay_verifier = verifier;
    }

    let mut saved_replay_verification = Value::Null;
    if let Some(path) = replay_verify_path.as_deref() {
        let (hash, verification) = verify_saved_replay_verification(root, path, &mut issues);
        replay_verification_receipt_hash = hash;
        saved_replay_verification = verification;
    }

    verify_optional_bundle_hash(
        &mut issues,
        &bundle,
        "agent_receipt_hash",
        &agent_receipt_hash,
    );
    verify_optional_bundle_hash(
        &mut issues,
        &bundle,
        "replay_receipt_hash",
        &replay_receipt_hash,
    );
    verify_optional_bundle_hash(
        &mut issues,
        &bundle,
        "replay_verification_receipt_hash",
        &replay_verification_receipt_hash,
    );

    compare_replay_verification(&mut issues, &saved_replay_verification, &replay_verifier);
    compare_agent_replay_evidence(
        &mut issues,
        &agent_receipt,
        &replay_verifier,
        &current_policy_hash,
    );

    if let Some(path) = release_path.as_deref() {
        release_readiness_receipt_hash = verify_optional_receipt_artifact(
            root,
            path,
            &mut issues,
            "release_readiness",
            "valid release-readiness receipt",
        );
        verify_optional_bundle_hash(
            &mut issues,
            &bundle,
            "release_readiness_receipt_hash",
            &release_readiness_receipt_hash,
        );
    }

    if let Some(path) = coverage_audit_path.as_deref() {
        coverage_audit_receipt_hash = verify_optional_receipt_artifact(
            root,
            path,
            &mut issues,
            "coverage_audit",
            "valid coverage-audit receipt",
        );
        verify_optional_bundle_hash(
            &mut issues,
            &bundle,
            "coverage_audit_receipt_hash",
            &coverage_audit_receipt_hash,
        );
    }

    if let Some(path) = runtime_evidence_path.as_deref() {
        runtime_evidence_receipt_hash = verify_optional_receipt_artifact(
            root,
            path,
            &mut issues,
            "runtime_evidence",
            "valid runtime-evidence receipt",
        );
        verify_optional_bundle_hash(
            &mut issues,
            &bundle,
            "runtime_evidence_receipt_hash",
            &runtime_evidence_receipt_hash,
        );
    }

    issues.sort_by_key(canonical_json_hash);
    agent_bundle_receipt(
        root,
        issues,
        current_policy_hash,
        agent_receipt_hash,
        replay_receipt_hash,
        replay_verification_receipt_hash,
        environment_attestation_hash,
        release_readiness_receipt_hash,
        coverage_audit_receipt_hash,
        runtime_evidence_receipt_hash,
    )
}
