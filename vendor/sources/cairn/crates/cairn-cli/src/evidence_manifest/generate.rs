use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::canonical_json_hash;
use serde_json::{json, Value};

use crate::support::{
    path_string, portable_json_hash, print_json, read_option, with_portable_receipt_hash,
};

use super::artifact::{manifest_artifact_entry, read_manifest_artifact};
use super::issues::evidence_manifest_issue;
use super::metadata::read_optional_manifest_metadata;

#[derive(Default)]
pub(crate) struct EvidenceManifestGenerateOptions {
    pub(crate) root: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) agent_receipt: Option<PathBuf>,
    pub(crate) replay: Option<PathBuf>,
    pub(crate) replay_verification: Option<PathBuf>,
    pub(crate) release_readiness: Option<PathBuf>,
    pub(crate) coverage_audit: Option<PathBuf>,
    pub(crate) requirement_registry: Option<PathBuf>,
    pub(crate) registry: Option<PathBuf>,
    pub(crate) registry_verification: Option<PathBuf>,
    pub(crate) registry_selection: Option<PathBuf>,
    pub(crate) registry_fetch_plan: Option<PathBuf>,
    pub(crate) runtime_evidence: Option<PathBuf>,
    pub(crate) runtime_chain: Option<PathBuf>,
    pub(crate) witness_checkpoint: Option<PathBuf>,
    pub(crate) signer_metadata: Option<PathBuf>,
    pub(crate) anchor_metadata: Option<PathBuf>,
    pub(crate) out: Option<String>,
}

pub(super) fn run_evidence_manifest_generate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.evidence_manifest_generate.cli]
    // r[impl cairn.supersede.evidence_manifest_generate.canonical]
    // r[impl cairn.supersede.evidence_manifest_generate.metadata]
    // r[impl cairn.supersede.evidence_manifest_generate.failures]
    // r[impl cairn.supersede.registry_manifest_binding.generate]
    // r[impl cairn.supersede.registry_manifest_binding.failure]
    // r[impl cairn.supersede.registry_manifest_binding.policy]
    let options = evidence_manifest_generate_options(&args);
    let (manifest, issues) = evidence_manifest_generate_manifest(&options);
    let valid = issues.is_empty();
    if !valid {
        print_json(evidence_manifest_generate_receipt(
            &options.root,
            Value::Null,
            false,
            Value::Null,
            issues,
        ))?;
        std::process::exit(1);
    }
    let manifest = manifest.expect("manifest exists when generation issues are empty");
    let manifest_hash = Value::String(portable_json_hash(&options.root, &manifest));
    match options.out.as_deref() {
        Some("-") | None => print_json(manifest)?,
        Some(out) => {
            let path = Path::new(out);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(
                path,
                format!("{}\n", serde_json::to_string_pretty(&manifest)?),
            )?;
            print_json(evidence_manifest_generate_receipt(
                &options.root,
                manifest_hash,
                true,
                Value::String("written".to_string()),
                Vec::new(),
            ))?;
        }
    }
    Ok(())
}

fn evidence_manifest_generate_options(args: &[String]) -> EvidenceManifestGenerateOptions {
    let mut options = EvidenceManifestGenerateOptions {
        root: read_option(args.to_vec(), "--root", "."),
        ..EvidenceManifestGenerateOptions::default()
    };
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let value = match arg.as_str() {
            "--bundle"
            | "--agent-receipt"
            | "--replay"
            | "--replay-verification"
            | "--release-readiness"
            | "--coverage-audit"
            | "--requirement-registry"
            | "--registry"
            | "--registry-verification"
            | "--registry-selection"
            | "--registry-fetch-plan"
            | "--runtime-evidence"
            | "--runtime-chain"
            | "--witness-checkpoint"
            | "--signer-metadata"
            | "--anchor-metadata"
            | "--out"
            | "--root" => iter.next(),
            _ => None,
        };
        match (arg.as_str(), value) {
            ("--bundle", Some(value)) => options.bundle = Some(PathBuf::from(value)),
            ("--agent-receipt", Some(value)) => options.agent_receipt = Some(PathBuf::from(value)),
            ("--replay", Some(value)) => options.replay = Some(PathBuf::from(value)),
            ("--replay-verification", Some(value)) => {
                options.replay_verification = Some(PathBuf::from(value));
            }
            ("--release-readiness", Some(value)) => {
                options.release_readiness = Some(PathBuf::from(value));
            }
            ("--coverage-audit", Some(value)) => {
                options.coverage_audit = Some(PathBuf::from(value));
            }
            ("--requirement-registry", Some(value)) => {
                options.requirement_registry = Some(PathBuf::from(value));
            }
            ("--registry", Some(value)) => {
                options.registry = Some(PathBuf::from(value));
            }
            ("--registry-verification", Some(value)) => {
                options.registry_verification = Some(PathBuf::from(value));
            }
            ("--registry-selection", Some(value)) => {
                options.registry_selection = Some(PathBuf::from(value));
            }
            ("--registry-fetch-plan", Some(value)) => {
                options.registry_fetch_plan = Some(PathBuf::from(value));
            }
            ("--runtime-evidence", Some(value)) => {
                options.runtime_evidence = Some(PathBuf::from(value));
            }
            ("--runtime-chain", Some(value)) => {
                options.runtime_chain = Some(PathBuf::from(value));
            }
            ("--witness-checkpoint", Some(value)) => {
                options.witness_checkpoint = Some(PathBuf::from(value));
            }
            ("--signer-metadata", Some(value)) => {
                options.signer_metadata = Some(PathBuf::from(value));
            }
            ("--anchor-metadata", Some(value)) => {
                options.anchor_metadata = Some(PathBuf::from(value));
            }
            ("--out", Some(value)) => options.out = Some(value.to_string()),
            ("--root", Some(_)) => {}
            _ => {}
        }
    }
    options
}

pub(crate) fn evidence_manifest_generate_manifest(
    options: &EvidenceManifestGenerateOptions,
) -> (Option<Value>, Vec<Value>) {
    let mut issues = Vec::new();
    let mut artifacts = Vec::new();
    for (role, path) in [
        ("agent_evidence_bundle", options.bundle.as_deref()),
        ("agent_receipt", options.agent_receipt.as_deref()),
        ("replay_receipt", options.replay.as_deref()),
        (
            "replay_verification",
            options.replay_verification.as_deref(),
        ),
        ("release_readiness", options.release_readiness.as_deref()),
        ("coverage_audit", options.coverage_audit.as_deref()),
        (
            "requirement_registry",
            options.requirement_registry.as_deref(),
        ),
        ("evidence_registry", options.registry.as_deref()),
        (
            "evidence_registry_verification",
            options.registry_verification.as_deref(),
        ),
        (
            "evidence_registry_selection",
            options.registry_selection.as_deref(),
        ),
        (
            "evidence_registry_fetch_plan",
            options.registry_fetch_plan.as_deref(),
        ),
        ("runtime_evidence", options.runtime_evidence.as_deref()),
        ("runtime_chain", options.runtime_chain.as_deref()),
        ("witness_checkpoint", options.witness_checkpoint.as_deref()),
    ] {
        if let Some(path) = path {
            match read_manifest_artifact(&options.root, role, path) {
                Ok((value, hash)) => match manifest_artifact_entry(role, path, &value, hash) {
                    Ok(entry) => artifacts.push(entry),
                    Err(_) => issues.push(evidence_manifest_issue(
                        "generate.artifact_identity_failed",
                        role,
                        "hash-bound JSON artifact with required receipt identity",
                        "invalid",
                    )),
                },
                Err(_) => issues.push(evidence_manifest_issue(
                    "generate.artifact_read_failed",
                    role,
                    "readable JSON artifact",
                    "unreadable",
                )),
            }
        } else if ![
            "release_readiness",
            "coverage_audit",
            "requirement_registry",
            "evidence_registry",
            "evidence_registry_verification",
            "evidence_registry_selection",
            "evidence_registry_fetch_plan",
            "runtime_evidence",
            "runtime_chain",
            "witness_checkpoint",
        ]
        .contains(&role)
        {
            issues.push(evidence_manifest_issue(
                "generate.required_artifact_missing",
                role,
                "artifact path flag",
                "missing",
            ));
        }
    }
    let signer =
        read_optional_manifest_metadata(options.signer_metadata.as_deref(), "signer", &mut issues);
    let anchor =
        read_optional_manifest_metadata(options.anchor_metadata.as_deref(), "anchor", &mut issues);
    issues.sort_by_key(canonical_json_hash);
    if !issues.is_empty() {
        return (None, issues);
    }
    let mut object = serde_json::Map::new();
    object.insert("artifacts".to_string(), Value::Array(artifacts));
    if let Some(value) = signer {
        object.insert("signer".to_string(), value);
    }
    if let Some(value) = anchor {
        object.insert("anchor".to_string(), value);
    }
    (Some(Value::Object(object)), issues)
}

fn evidence_manifest_generate_receipt(
    root: &Path,
    manifest_hash: Value,
    valid: bool,
    output: Value,
    issues: Vec<Value>,
) -> Value {
    with_portable_receipt_hash(
        root,
        json!({
            "command": "evidence-manifest generate",
            "root": path_string(root),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "manifest_hash": manifest_hash,
            "output": output,
        }),
    )
}
