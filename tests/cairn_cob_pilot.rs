use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

const FIXTURE_DIRECTORY: &str = "fixtures/cairn-cob-pilot";
const MANIFEST_FILE: &str = "manifest.json";
const IMPORT_PREVIEW: &str = "import-preview.json";
const IMPORT_EXECUTE: &str = "import-execute.json";
const UPDATE_PREVIEW: &str = "update-preview.json";
const UPDATE_EXECUTE: &str = "update-execute.json";
const STATUS_SECONDARY: &str = "status-secondary.json";
const BUNDLE_RELOAD: &str = "bundle-reload.json";
const CHECKOUT_PREVIEW: &str = "checkout-preview.json";
const CHECKOUT_EXECUTE: &str = "checkout-execute.json";
const CONFLICTS_SECONDARY: &str = "conflicts-secondary.json";
const TRANSPORT: &str = "transport.json";
const SUPPORTED_SCHEMA: &str = "octet.cairn-cob-pilot.v1";
const COB_BACKEND: &str = "radicle-cob.v1";
const IMPORT_COMMAND: &str = "cob import";
const UPDATE_COMMAND: &str = "collaboration update";
const STATUS_COMMAND: &str = "collaboration status";
const CHECKOUT_COMMAND: &str = "collaboration checkout";
const CONFLICTS_COMMAND: &str = "collaboration conflicts";
const CHECKOUT_MISSING_CHANGES_NEGATIVE: &str = "checkout-missing-changes-negative.json";
const PASS_VERDICT: &str = "PASS";
const FAIL_VERDICT: &str = "FAIL";
const MISSING_CHANGES_BLOCKER: &str = "native cairn/changes directory is missing, unsafe, or not a directory";
const TAMPERED_OBJECT_ID: &str = "0000000000000000000000000000000000000000";

// r[impl octet.cob_collaboration.pilot]
// r[verify octet.cob_collaboration.pilot.import]
// r[verify octet.cob_collaboration.pilot.update]
// r[impl octet.cob_collaboration.transport]
// r[impl octet.cob_collaboration.checkout]
// r[impl octet.cob_collaboration.evidence]
// r[verify octet.cob_collaboration.validation]
// r[impl octet.cob_collaboration.boundary]

#[derive(Clone, Debug, Deserialize)]
struct PilotManifest {
    schema: String,
    change: String,
    workspace_id: String,
    backend: String,
    cairn_commit: String,
    base_git_oid: String,
    actor: String,
    object_id: String,
    import_operation_id: String,
    update_operation_id: String,
    final_content_root: String,
    final_state_hash: String,
    namespace_ref: String,
    bundle: BundleManifest,
    transport: TransportManifest,
    receipts: Vec<ReceiptManifest>,
    required_non_claim_fragments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ReceiptManifest {
    name: String,
    blake3_hex: String,
}

#[derive(Clone, Debug, Deserialize)]
struct BundleManifest {
    path: String,
    blake3_hex: String,
    heads: Vec<BundleHead>,
}

#[derive(Clone, Debug, Deserialize)]
struct BundleHead {
    ref_name: String,
    target: String,
}

#[derive(Clone, Debug, Deserialize)]
struct TransportManifest {
    source_ref_target: String,
    destination_ref_target: String,
    source_remotes: Vec<String>,
    destination_remotes: Vec<String>,
    artifact_tree_blake3: String,
}

#[derive(Clone, Debug)]
struct EvidenceFile {
    bytes: Vec<u8>,
    json: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum PilotEvidenceError {
    MissingReceipt(String),
    ReceiptHash { receipt: String },
    Field { receipt: String, field: String },
    NonClaim { fragment: String },
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIRECTORY)
}

fn load_manifest() -> PilotManifest {
    let path = fixture_root().join(MANIFEST_FILE);
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

fn load_evidence(manifest: &PilotManifest) -> BTreeMap<String, EvidenceFile> {
    let evidence_root = fixture_root().join("evidence");
    manifest
        .receipts
        .iter()
        .map(|receipt| {
            let path = evidence_root.join(&receipt.name);
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", path.display());
            });
            let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
                panic!("failed to parse {}: {error}", path.display());
            });
            (receipt.name.clone(), EvidenceFile { bytes, json })
        })
        .collect()
}

fn evidence<'a>(files: &'a BTreeMap<String, EvidenceFile>, name: &str) -> Result<&'a EvidenceFile, PilotEvidenceError> {
    files.get(name).ok_or_else(|| PilotEvidenceError::MissingReceipt(name.to_string()))
}

fn require_field(receipt: &str, value: &Value, field: &str, expected: Value) -> Result<(), PilotEvidenceError> {
    if value.get(field).cloned() == Some(expected) {
        return Ok(());
    }
    Err(PilotEvidenceError::Field {
        receipt: receipt.to_string(),
        field: field.to_string(),
    })
}

fn require_pointer(receipt: &str, value: &Value, pointer: &str, expected: Value) -> Result<(), PilotEvidenceError> {
    if value.pointer(pointer).cloned() == Some(expected) {
        return Ok(());
    }
    Err(PilotEvidenceError::Field {
        receipt: receipt.to_string(),
        field: pointer.to_string(),
    })
}

fn require_common(
    receipt: &str,
    value: &Value,
    manifest: &PilotManifest,
    command: &str,
) -> Result<(), PilotEvidenceError> {
    require_field(receipt, value, "command", Value::String(command.to_string()))?;
    require_field(receipt, value, "backend", Value::String(manifest.backend.clone()))?;
    require_field(receipt, value, "change", Value::String(manifest.change.clone()))?;
    require_field(receipt, value, "verdict", Value::String(PASS_VERDICT.to_string()))
}

fn validate_hashes(manifest: &PilotManifest, files: &BTreeMap<String, EvidenceFile>) -> Result<(), PilotEvidenceError> {
    for receipt in &manifest.receipts {
        let file = evidence(files, &receipt.name)?;
        let actual = blake3::hash(&file.bytes).to_hex().to_string();
        if actual != receipt.blake3_hex {
            return Err(PilotEvidenceError::ReceiptHash {
                receipt: receipt.name.clone(),
            });
        }
    }
    Ok(())
}

fn validate_import(manifest: &PilotManifest, files: &BTreeMap<String, EvidenceFile>) -> Result<(), PilotEvidenceError> {
    let preview = &evidence(files, IMPORT_PREVIEW)?.json;
    let executed = &evidence(files, IMPORT_EXECUTE)?.json;
    require_common(IMPORT_PREVIEW, preview, manifest, IMPORT_COMMAND)?;
    require_common(IMPORT_EXECUTE, executed, manifest, IMPORT_COMMAND)?;
    require_field(IMPORT_PREVIEW, preview, "dry_run", Value::Bool(true))?;
    require_field(IMPORT_PREVIEW, preview, "mutated", Value::Bool(false))?;
    require_field(IMPORT_PREVIEW, preview, "object_id", Value::Null)?;
    require_field(IMPORT_EXECUTE, executed, "dry_run", Value::Bool(false))?;
    require_field(IMPORT_EXECUTE, executed, "mutated", Value::Bool(true))?;
    require_field(IMPORT_EXECUTE, executed, "workspace_id", Value::String(manifest.workspace_id.clone()))?;
    require_field(IMPORT_EXECUTE, executed, "repository_head_oid", Value::String(manifest.base_git_oid.clone()))?;
    require_field(IMPORT_EXECUTE, executed, "actor", Value::String(manifest.actor.clone()))?;
    require_field(IMPORT_EXECUTE, executed, "object_id", Value::String(manifest.object_id.clone()))?;
    require_field(IMPORT_EXECUTE, executed, "operation_id", Value::String(manifest.import_operation_id.clone()))?;
    require_field(IMPORT_EXECUTE, executed, "content_root", preview["content_root"].clone())?;
    require_field(IMPORT_EXECUTE, executed, "plan_hash", preview["plan_hash"].clone())?;
    require_field(IMPORT_EXECUTE, executed, "input_hash", preview["input_hash"].clone())
}

fn validate_update(manifest: &PilotManifest, files: &BTreeMap<String, EvidenceFile>) -> Result<(), PilotEvidenceError> {
    let preview = &evidence(files, UPDATE_PREVIEW)?.json;
    let executed = &evidence(files, UPDATE_EXECUTE)?.json;
    require_common(UPDATE_PREVIEW, preview, manifest, UPDATE_COMMAND)?;
    require_common(UPDATE_EXECUTE, executed, manifest, UPDATE_COMMAND)?;
    require_field(UPDATE_PREVIEW, preview, "dry_run", Value::Bool(true))?;
    require_field(UPDATE_PREVIEW, preview, "mutated", Value::Bool(false))?;
    require_field(UPDATE_PREVIEW, preview, "noop", Value::Bool(false))?;
    require_field(UPDATE_EXECUTE, executed, "dry_run", Value::Bool(false))?;
    require_field(UPDATE_EXECUTE, executed, "mutated", Value::Bool(true))?;
    require_pointer(UPDATE_EXECUTE, executed, "/actor/id", Value::String(manifest.actor.clone()))?;
    require_field(UPDATE_EXECUTE, executed, "object_id", Value::String(manifest.object_id.clone()))?;
    require_field(
        UPDATE_EXECUTE,
        executed,
        "result_operation_id",
        Value::String(manifest.update_operation_id.clone()),
    )?;
    require_field(UPDATE_EXECUTE, executed, "result_content_root", Value::String(manifest.final_content_root.clone()))?;
    require_field(UPDATE_EXECUTE, executed, "result_state_hash", Value::String(manifest.final_state_hash.clone()))?;
    require_field(
        UPDATE_PREVIEW,
        preview,
        "expected_content_root",
        Value::String(manifest.final_content_root.clone()),
    )?;
    require_field(UPDATE_EXECUTE, executed, "plan_hash", preview["plan_hash"].clone())?;
    require_field(
        UPDATE_EXECUTE,
        executed,
        "changed_artifacts",
        Value::Array(vec![Value::String("tasks".to_string())]),
    )
}

fn validate_secondary(
    manifest: &PilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), PilotEvidenceError> {
    let status = &evidence(files, STATUS_SECONDARY)?.json;
    let bundle_reload = &evidence(files, BUNDLE_RELOAD)?.json;
    let checkout_preview = &evidence(files, CHECKOUT_PREVIEW)?.json;
    let checkout_execute = &evidence(files, CHECKOUT_EXECUTE)?.json;
    let conflicts = &evidence(files, CONFLICTS_SECONDARY)?.json;
    require_common(STATUS_SECONDARY, status, manifest, STATUS_COMMAND)?;
    require_common(BUNDLE_RELOAD, bundle_reload, manifest, STATUS_COMMAND)?;
    require_common(CHECKOUT_PREVIEW, checkout_preview, manifest, CHECKOUT_COMMAND)?;
    require_common(CHECKOUT_EXECUTE, checkout_execute, manifest, CHECKOUT_COMMAND)?;
    require_common(CONFLICTS_SECONDARY, conflicts, manifest, CONFLICTS_COMMAND)?;
    require_field(STATUS_SECONDARY, status, "object_id", Value::String(manifest.object_id.clone()))?;
    require_field(STATUS_SECONDARY, status, "state_hash", Value::String(manifest.final_state_hash.clone()))?;
    require_field(BUNDLE_RELOAD, bundle_reload, "object_id", Value::String(manifest.object_id.clone()))?;
    require_field(BUNDLE_RELOAD, bundle_reload, "state_hash", Value::String(manifest.final_state_hash.clone()))?;
    require_pointer(BUNDLE_RELOAD, bundle_reload, "/authority/creator", Value::String(manifest.actor.clone()))?;
    for (receipt, value) in [
        (STATUS_SECONDARY, status),
        (BUNDLE_RELOAD, bundle_reload),
        (CHECKOUT_PREVIEW, checkout_preview),
        (CHECKOUT_EXECUTE, checkout_execute),
        (CONFLICTS_SECONDARY, conflicts),
    ] {
        require_field(receipt, value, "object_id", Value::String(manifest.object_id.clone()))?;
        require_field(receipt, value, "state_hash", Value::String(manifest.final_state_hash.clone()))?;
    }
    require_field(CHECKOUT_PREVIEW, checkout_preview, "dry_run", Value::Bool(true))?;
    require_field(CHECKOUT_PREVIEW, checkout_preview, "mutated", Value::Bool(false))?;
    require_field(CHECKOUT_EXECUTE, checkout_execute, "dry_run", Value::Bool(false))?;
    require_field(CHECKOUT_EXECUTE, checkout_execute, "mutated", Value::Bool(true))?;
    require_field(
        CHECKOUT_EXECUTE,
        checkout_execute,
        "post_checkout_destination",
        Value::String("current".to_string()),
    )?;
    require_field(CHECKOUT_EXECUTE, checkout_execute, "files", status["files"].clone())?;
    require_field(CONFLICTS_SECONDARY, conflicts, "conflicted", Value::Bool(false))?;

    let negative = &evidence(files, CHECKOUT_MISSING_CHANGES_NEGATIVE)?.json;
    require_field(CHECKOUT_MISSING_CHANGES_NEGATIVE, negative, "command", Value::String(CHECKOUT_COMMAND.to_string()))?;
    require_field(CHECKOUT_MISSING_CHANGES_NEGATIVE, negative, "blocked", Value::Bool(true))?;
    require_field(CHECKOUT_MISSING_CHANGES_NEGATIVE, negative, "mutated", Value::Bool(false))?;
    require_field(CHECKOUT_MISSING_CHANGES_NEGATIVE, negative, "verdict", Value::String(FAIL_VERDICT.to_string()))?;
    require_field(
        CHECKOUT_MISSING_CHANGES_NEGATIVE,
        negative,
        "blockers",
        Value::Array(vec![Value::String(MISSING_CHANGES_BLOCKER.to_string())]),
    )
}

fn validate_bundle(manifest: &PilotManifest, bundle_bytes: &[u8]) -> Result<(), PilotEvidenceError> {
    let actual_hash = blake3::hash(bundle_bytes).to_hex().to_string();
    if actual_hash != manifest.bundle.blake3_hex {
        return Err(PilotEvidenceError::ReceiptHash {
            receipt: manifest.bundle.path.clone(),
        });
    }
    let main_matches = manifest
        .bundle
        .heads
        .iter()
        .any(|head| head.ref_name == "refs/heads/main" && head.target == manifest.base_git_oid);
    if !main_matches {
        return Err(PilotEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "bundle.heads.main".to_string(),
        });
    }
    let namespace_matches = manifest
        .bundle
        .heads
        .iter()
        .any(|head| head.ref_name == manifest.namespace_ref && head.target == manifest.update_operation_id);
    if !namespace_matches {
        return Err(PilotEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "bundle.heads.namespace".to_string(),
        });
    }
    Ok(())
}

fn validate_transport(
    manifest: &PilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), PilotEvidenceError> {
    let transport = &evidence(files, TRANSPORT)?.json;
    if manifest.transport.source_ref_target != manifest.update_operation_id
        || manifest.transport.destination_ref_target != manifest.update_operation_id
    {
        return Err(PilotEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "transport.ref_target".to_string(),
        });
    }
    require_field(TRANSPORT, transport, "cairn_commit", Value::String(manifest.cairn_commit.clone()))?;
    require_field(TRANSPORT, transport, "actor", Value::String(manifest.actor.clone()))?;
    require_field(TRANSPORT, transport, "object_id", Value::String(manifest.object_id.clone()))?;
    require_field(TRANSPORT, transport, "source_branch_target", Value::String(manifest.base_git_oid.clone()))?;
    require_field(TRANSPORT, transport, "destination_branch_target", Value::String(manifest.base_git_oid.clone()))?;
    require_field(TRANSPORT, transport, "destination_tags", Value::Array(Vec::new()))?;
    require_field(TRANSPORT, transport, "valid", Value::Bool(true))?;
    require_field(TRANSPORT, transport, "base_git_oid", Value::String(manifest.base_git_oid.clone()))?;
    require_field(TRANSPORT, transport, "namespace_ref", Value::String(manifest.namespace_ref.clone()))?;
    require_field(
        TRANSPORT,
        transport,
        "source_ref_target",
        Value::String(manifest.transport.source_ref_target.clone()),
    )?;
    require_field(
        TRANSPORT,
        transport,
        "destination_ref_target",
        Value::String(manifest.transport.destination_ref_target.clone()),
    )?;
    require_field(
        TRANSPORT,
        transport,
        "source_remotes",
        serde_json::to_value(&manifest.transport.source_remotes).unwrap_or(Value::Null),
    )?;
    require_field(
        TRANSPORT,
        transport,
        "destination_remotes",
        serde_json::to_value(&manifest.transport.destination_remotes).unwrap_or(Value::Null),
    )?;
    require_field(
        TRANSPORT,
        transport,
        "artifact_tree_blake3",
        Value::String(manifest.transport.artifact_tree_blake3.clone()),
    )
}

fn validate_non_claims(
    manifest: &PilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), PilotEvidenceError> {
    let mut joined = String::new();
    for file in files.values() {
        let Some(non_claims) = file.json.get("non_claims") else {
            continue;
        };
        joined.push_str(&non_claims.to_string());
    }
    for fragment in &manifest.required_non_claim_fragments {
        if !joined.contains(fragment) {
            return Err(PilotEvidenceError::NonClaim {
                fragment: fragment.clone(),
            });
        }
    }
    Ok(())
}

fn validate_pilot_evidence(
    manifest: &PilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    bundle_bytes: &[u8],
) -> Result<(), PilotEvidenceError> {
    if manifest.schema != SUPPORTED_SCHEMA {
        return Err(PilotEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "schema".to_string(),
        });
    }
    if manifest.backend != COB_BACKEND {
        return Err(PilotEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "backend".to_string(),
        });
    }
    validate_hashes(manifest, files)?;
    validate_bundle(manifest, bundle_bytes)?;
    validate_import(manifest, files)?;
    validate_update(manifest, files)?;
    validate_secondary(manifest, files)?;
    validate_transport(manifest, files)?;
    validate_non_claims(manifest, files)
}

#[test]
fn checked_in_cairn_cob_pilot_evidence_is_cross_linked() {
    let manifest = load_manifest();
    let files = load_evidence(&manifest);
    let bundle_path = fixture_root().join(&manifest.bundle.path);
    let bundle_bytes = fs::read(&bundle_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", bundle_path.display());
    });
    assert_eq!(validate_pilot_evidence(&manifest, &files, &bundle_bytes), Ok(()));
}

#[test]
fn tampered_cairn_cob_object_identity_fails_closed() {
    let mut manifest = load_manifest();
    let files = load_evidence(&manifest);
    let bundle_path = fixture_root().join(&manifest.bundle.path);
    let bundle_bytes = fs::read(&bundle_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", bundle_path.display());
    });
    manifest.object_id = TAMPERED_OBJECT_ID.to_string();
    assert_eq!(
        validate_pilot_evidence(&manifest, &files, &bundle_bytes),
        Err(PilotEvidenceError::Field {
            receipt: IMPORT_EXECUTE.to_string(),
            field: "object_id".to_string(),
        })
    );
}
