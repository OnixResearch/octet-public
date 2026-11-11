use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, canonical_json_hash, verified::verification_role::VerificationRoleSummary, Policy,
};
use serde_json::{json, Value};

use crate::agent_receipt::{
    agent_evidence_bundle_verification_receipt, agent_receipt_validation_receipt,
    receipt_stated_or_computed_hash, AGENT_RECEIPT_MAX_BYTES, REQUIRED_AGENT_REPLAY_GROUP,
};
use crate::content_addressed_evidence::{
    run_evidence_circuit_primitive, run_evidence_downstream_pilot,
    run_evidence_export_bundle_receipt, run_evidence_function_object, run_evidence_name_pointer,
};
use crate::determinism::{
    determinism_coverage_audit_receipt, determinism_replay_receipt,
    determinism_verify_replay_receipt,
};
use crate::evidence_manifest::{
    evidence_manifest_generate_manifest, evidence_manifest_verification_receipt,
    EvidenceManifestGenerateOptions,
};
use crate::evidence_registry::run_evidence_registry;
use crate::release_readiness::release_readiness_receipt;
use crate::runtime_evidence::{
    runtime_evidence_verification_receipt, RuntimeEvidenceVerifyOptions,
};
use crate::support::{
    default_policy_path, load_policy_file, path_string, print_json, read_option,
    read_string_option, with_portable_content_hash_receipt,
};

mod export;
mod import;
mod index;
mod republish;
mod verify;

pub(crate) use export::{evidence_export_receipt, write_evidence_json, EvidenceExportOptions};
pub(crate) use import::{evidence_import_receipt, EvidenceImportOptions};
pub(crate) use index::{
    evidence_index_receipt, evidence_index_verify_receipt, EvidenceIndexOptions,
    EvidenceIndexVerifyOptions,
};
pub(crate) use republish::{evidence_republish_receipt, EvidenceRepublishOptions};
pub(crate) use verify::{evidence_verify_receipt, EvidenceVerifyOptions};

pub(crate) fn run_evidence(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence requires a subcommand: export".into());
    };
    match subcommand.as_str() {
        "export" => export::run_evidence_export(iter.collect()),
        "verify" => verify::run_evidence_verify(iter.collect()),
        "import" => import::run_evidence_import(iter.collect()),
        "republish" => republish::run_evidence_republish(iter.collect()),
        "index" => index::run_evidence_index(iter.collect()),
        "registry" => run_evidence_registry(iter.collect()),
        "function-object" => run_evidence_function_object(iter.collect()),
        "name-pointer" => run_evidence_name_pointer(iter.collect()),
        "export-bundle" => run_evidence_export_bundle_receipt(iter.collect()),
        "downstream-pilot" => run_evidence_downstream_pilot(iter.collect()),
        "circuit-primitive" => run_evidence_circuit_primitive(iter.collect()),
        unknown => Err(format!("unknown evidence subcommand: {unknown}").into()),
    }
}

fn safe_bundle_artifact_path(bundle_dir: &Path, relative: &str) -> Option<PathBuf> {
    let path = PathBuf::from(relative);
    if relative.is_empty() || path.is_absolute() {
        return None;
    }
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }
    Some(bundle_dir.join(path))
}
