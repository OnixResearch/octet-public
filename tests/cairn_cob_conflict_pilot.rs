use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

const FIXTURE_DIRECTORY: &str = "fixtures/cairn-cob-conflict-pilot";
const MANIFEST_FILE: &str = "manifest.json";
const SUPPORTED_SCHEMA: &str = "octet.cairn-cob-conflict-pilot.v1";
const COB_BACKEND: &str = "radicle-cob.v1";
const PASS_VERDICT: &str = "PASS";
const FAIL_VERDICT: &str = "FAIL";
const IMPORT_PREVIEW: &str = "import-preview.json";
const IMPORT_EXECUTE: &str = "import-execute.json";
const CREATOR_UPDATE_PREVIEW: &str = "creator-update-preview.json";
const CREATOR_UPDATE_EXECUTE: &str = "creator-update-execute.json";
const DELEGATE_UPDATE_PREVIEW: &str = "delegate-update-preview.json";
const DELEGATE_UPDATE_EXECUTE: &str = "delegate-update-execute.json";
const CONFLICTS_METADATA: &str = "conflicts-metadata.json";
const CONFLICTS_SHOW_CREATOR: &str = "conflicts-show-creator.json";
const CONFLICTS_SHOW_DELEGATE: &str = "conflicts-show-delegate.json";
const UPDATE_CONFLICT_NEGATIVE: &str = "update-conflict-negative.json";
const RESOLVE_PREVIEW: &str = "resolve-preview.json";
const RESOLVE_EXECUTE: &str = "resolve-execute.json";
const STATUS_THIRD_BEFORE: &str = "status-third-before-checkout.json";
const CONFLICTS_RESOLVED_THIRD: &str = "conflicts-resolved-third.json";
const CHECKOUT_THIRD_PREVIEW: &str = "checkout-third-preview.json";
const CHECKOUT_THIRD_EXECUTE: &str = "checkout-third-execute.json";
const STATUS_THIRD_AFTER: &str = "status-third-after-checkout.json";
const TRANSPORT: &str = "transport.json";
const IMPORT_COMMAND: &str = "cob import";
const UPDATE_COMMAND: &str = "collaboration update";
const CONFLICTS_COMMAND: &str = "collaboration conflicts";
const RESOLVE_COMMAND: &str = "collaboration resolve";
const STATUS_COMMAND: &str = "collaboration status";
const CHECKOUT_COMMAND: &str = "collaboration checkout";
const TASKS_ARTIFACT: &str = "tasks";
const CREATOR_AUTHORITY: &str = "signed_creator";
const DELEGATE_AUTHORITY: &str = "embedded_delegate";
const STALE_LIFECYCLE: &str = "stale";
const EXPECTED_CONFLICT_HEAD_COUNT: usize = 2;
const EXPECTED_BUNDLE_HEAD_COUNT: usize = 3;
const TAMPERED_OPERATION_ID: &str = "0000000000000000000000000000000000000000";

// r[impl octet.cob_conflict_recovery.divergence]
// r[verify octet.cob_conflict_recovery.divergence.same_basis]
// r[verify octet.cob_conflict_recovery.divergence.transport]
// r[impl octet.cob_conflict_recovery.inspection]
// r[verify octet.cob_conflict_recovery.inspection.metadata]
// r[verify octet.cob_conflict_recovery.inspection.values]
// r[impl octet.cob_conflict_recovery.refusal]
// r[verify octet.cob_conflict_recovery.refusal.update]
// r[impl octet.cob_conflict_recovery.resolution]
// r[verify octet.cob_conflict_recovery.resolution.preview]
// r[verify octet.cob_conflict_recovery.resolution.execute]
// r[impl octet.cob_conflict_recovery.reload]
// r[verify octet.cob_conflict_recovery.reload.status]
// r[verify octet.cob_conflict_recovery.reload.checkout]
// r[impl octet.cob_conflict_recovery.evidence]
// r[verify octet.cob_conflict_recovery.evidence.positive]
// r[verify octet.cob_conflict_recovery.evidence.negative]
// r[impl octet.cob_conflict_recovery.boundary]
// r[verify octet.cob_conflict_recovery.boundary.non_claims]

#[derive(Clone, Debug, Deserialize)]
struct ConflictPilotManifest {
    schema: String,
    change: String,
    workspace_id: String,
    backend: String,
    cairn_commit: String,
    octet_base_git_oid: String,
    policy_hash: String,
    object_id: String,
    root_operation_id: String,
    root_content_root: String,
    root_state_hash: String,
    creator: ActorHead,
    delegate: ActorHead,
    conflict_state_hash: String,
    resolution: ResolutionManifest,
    resolved_lifecycle_condition: String,
    bundle: BundleManifest,
    receipts: Vec<ReceiptManifest>,
    required_non_claim_fragments: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ActorHead {
    actor: String,
    ref_name: String,
    operation_id: String,
    content_blake3: String,
    content_bytes: usize,
    content_root: String,
    state_hash: String,
    retained_path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct ResolutionManifest {
    operation_id: String,
    selected_revision: String,
    losing_revision: String,
    direct_parents: Vec<String>,
    content_root: String,
    state_hash: String,
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

#[derive(Clone, Debug)]
struct EvidenceFile {
    bytes: Vec<u8>,
    json: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConflictEvidenceError {
    Missing(String),
    Hash(String),
    Field { receipt: String, field: String },
    NonClaim(String),
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIRECTORY)
}

fn load_manifest() -> ConflictPilotManifest {
    let path = fixture_root().join(MANIFEST_FILE);
    let bytes = fs::read(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

fn load_evidence(manifest: &ConflictPilotManifest) -> BTreeMap<String, EvidenceFile> {
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

fn load_head_values(manifest: &ConflictPilotManifest) -> BTreeMap<String, Vec<u8>> {
    [&manifest.creator, &manifest.delegate]
        .into_iter()
        .map(|head| {
            let path = fixture_root().join(&head.retained_path);
            let bytes = fs::read(&path).unwrap_or_else(|error| {
                panic!("failed to read {}: {error}", path.display());
            });
            (head.retained_path.clone(), bytes)
        })
        .collect()
}

fn evidence<'a>(
    files: &'a BTreeMap<String, EvidenceFile>,
    name: &str,
) -> Result<&'a EvidenceFile, ConflictEvidenceError> {
    files.get(name).ok_or_else(|| ConflictEvidenceError::Missing(name.to_string()))
}

fn require_field(receipt: &str, value: &Value, field: &str, expected: Value) -> Result<(), ConflictEvidenceError> {
    if value.get(field).cloned() == Some(expected) {
        return Ok(());
    }
    Err(ConflictEvidenceError::Field {
        receipt: receipt.to_string(),
        field: field.to_string(),
    })
}

fn require_pointer(receipt: &str, value: &Value, pointer: &str, expected: Value) -> Result<(), ConflictEvidenceError> {
    if value.pointer(pointer).cloned() == Some(expected) {
        return Ok(());
    }
    Err(ConflictEvidenceError::Field {
        receipt: receipt.to_string(),
        field: pointer.to_string(),
    })
}

fn require_common(
    receipt: &str,
    value: &Value,
    manifest: &ConflictPilotManifest,
    command: &str,
    verdict: &str,
) -> Result<(), ConflictEvidenceError> {
    require_field(receipt, value, "command", json!(command))?;
    require_field(receipt, value, "backend", json!(manifest.backend))?;
    require_field(receipt, value, "change", json!(manifest.change))?;
    require_field(receipt, value, "verdict", json!(verdict))
}

fn validate_manifest_shape(manifest: &ConflictPilotManifest) -> Result<(), ConflictEvidenceError> {
    if manifest.schema != SUPPORTED_SCHEMA {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "schema".to_string(),
        });
    }
    if manifest.backend != COB_BACKEND {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "backend".to_string(),
        });
    }
    let expected_parents = vec![
        manifest.creator.operation_id.clone(),
        manifest.delegate.operation_id.clone(),
    ];
    if manifest.resolution.selected_revision != manifest.creator.operation_id
        || manifest.resolution.losing_revision != manifest.delegate.operation_id
        || manifest.resolution.direct_parents != expected_parents
    {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "resolution.direct_parents".to_string(),
        });
    }
    if manifest.bundle.heads.len() != EXPECTED_BUNDLE_HEAD_COUNT {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "bundle.heads".to_string(),
        });
    }
    Ok(())
}

fn validate_hashes(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    head_values: &BTreeMap<String, Vec<u8>>,
    bundle_bytes: &[u8],
) -> Result<(), ConflictEvidenceError> {
    for receipt in &manifest.receipts {
        let actual = blake3::hash(&evidence(files, &receipt.name)?.bytes).to_hex().to_string();
        if actual != receipt.blake3_hex {
            return Err(ConflictEvidenceError::Hash(receipt.name.clone()));
        }
    }
    for head in [&manifest.creator, &manifest.delegate] {
        let bytes = head_values
            .get(&head.retained_path)
            .ok_or_else(|| ConflictEvidenceError::Missing(head.retained_path.clone()))?;
        if blake3::hash(bytes).to_hex().as_str() != head.content_blake3 || bytes.len() != head.content_bytes {
            return Err(ConflictEvidenceError::Hash(head.retained_path.clone()));
        }
    }
    if blake3::hash(bundle_bytes).to_hex().as_str() != manifest.bundle.blake3_hex {
        return Err(ConflictEvidenceError::Hash(manifest.bundle.path.clone()));
    }
    Ok(())
}

fn validate_import(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let preview = &evidence(files, IMPORT_PREVIEW)?.json;
    let executed = &evidence(files, IMPORT_EXECUTE)?.json;
    require_common(IMPORT_PREVIEW, preview, manifest, IMPORT_COMMAND, PASS_VERDICT)?;
    require_common(IMPORT_EXECUTE, executed, manifest, IMPORT_COMMAND, PASS_VERDICT)?;
    require_field(IMPORT_PREVIEW, preview, "dry_run", json!(true))?;
    require_field(IMPORT_PREVIEW, preview, "mutated", json!(false))?;
    require_field(IMPORT_EXECUTE, executed, "dry_run", json!(false))?;
    require_field(IMPORT_EXECUTE, executed, "mutated", json!(true))?;
    require_field(IMPORT_EXECUTE, executed, "workspace_id", json!(manifest.workspace_id))?;
    require_field(IMPORT_EXECUTE, executed, "actor", json!(manifest.creator.actor))?;
    require_field(IMPORT_EXECUTE, executed, "delegates", json!([manifest.delegate.actor]))?;
    require_field(IMPORT_EXECUTE, executed, "object_id", json!(manifest.object_id))?;
    require_field(IMPORT_EXECUTE, executed, "operation_id", json!(manifest.root_operation_id))?;
    require_field(IMPORT_EXECUTE, executed, "content_root", json!(manifest.root_content_root))?;
    require_field(IMPORT_EXECUTE, executed, "state_hash", json!(manifest.root_state_hash))?;
    require_field(IMPORT_EXECUTE, executed, "repository_head_oid", json!(manifest.octet_base_git_oid))?;
    require_field(IMPORT_EXECUTE, executed, "policy_hash", json!(manifest.policy_hash))?;
    require_field(IMPORT_EXECUTE, executed, "plan_hash", preview["plan_hash"].clone())
}

fn validate_actor_update(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    head: &ActorHead,
    preview_name: &str,
    execute_name: &str,
    authority: &str,
) -> Result<(), ConflictEvidenceError> {
    let preview = &evidence(files, preview_name)?.json;
    let executed = &evidence(files, execute_name)?.json;
    require_common(preview_name, preview, manifest, UPDATE_COMMAND, PASS_VERDICT)?;
    require_common(execute_name, executed, manifest, UPDATE_COMMAND, PASS_VERDICT)?;
    require_field(preview_name, preview, "input_state_hash", json!(manifest.root_state_hash))?;
    require_field(preview_name, preview, "dry_run", json!(true))?;
    require_field(preview_name, preview, "mutated", json!(false))?;
    require_field(execute_name, executed, "input_state_hash", json!(manifest.root_state_hash))?;
    require_field(execute_name, executed, "result_operation_id", json!(head.operation_id))?;
    require_field(execute_name, executed, "result_content_root", json!(head.content_root))?;
    require_field(execute_name, executed, "result_state_hash", json!(head.state_hash))?;
    require_pointer(execute_name, executed, "/actor/id", json!(head.actor))?;
    require_pointer(execute_name, executed, "/actor/authority_class", json!(authority))?;
    require_pointer(execute_name, executed, "/updates/0/basis", json!([manifest.root_operation_id]))?;
    require_pointer(execute_name, executed, "/updates/0/content_blake3", json!(head.content_blake3))?;
    require_field(execute_name, executed, "plan_hash", preview["plan_hash"].clone())
}

fn conflict_heads(manifest: &ConflictPilotManifest) -> Value {
    json!([
        {
            "actor": manifest.creator.actor,
            "content_blake3": manifest.creator.content_blake3,
            "content_bytes": manifest.creator.content_bytes,
            "deleted": false,
            "revision_id": manifest.creator.operation_id,
        },
        {
            "actor": manifest.delegate.actor,
            "content_blake3": manifest.delegate.content_blake3,
            "content_bytes": manifest.delegate.content_bytes,
            "deleted": false,
            "revision_id": manifest.delegate.operation_id,
        }
    ])
}

fn validate_conflict_inspection(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    head_values: &BTreeMap<String, Vec<u8>>,
) -> Result<(), ConflictEvidenceError> {
    let metadata = &evidence(files, CONFLICTS_METADATA)?.json;
    require_common(CONFLICTS_METADATA, metadata, manifest, CONFLICTS_COMMAND, PASS_VERDICT)?;
    require_field(CONFLICTS_METADATA, metadata, "conflicted", json!(true))?;
    require_field(CONFLICTS_METADATA, metadata, "state_hash", json!(manifest.conflict_state_hash))?;
    require_field(CONFLICTS_METADATA, metadata, "shown_head", Value::Null)?;
    require_pointer(CONFLICTS_METADATA, metadata, "/artifact_conflicts/0/artifact", json!(TASKS_ARTIFACT))?;
    require_pointer(CONFLICTS_METADATA, metadata, "/artifact_conflicts/0/heads", conflict_heads(manifest))?;
    if metadata["artifact_conflicts"][0]["heads"].as_array().map(Vec::len) != Some(EXPECTED_CONFLICT_HEAD_COUNT) {
        return Err(ConflictEvidenceError::Field {
            receipt: CONFLICTS_METADATA.to_string(),
            field: "artifact_conflicts.heads".to_string(),
        });
    }
    validate_shown_head(manifest, files, head_values, &manifest.creator, CONFLICTS_SHOW_CREATOR)?;
    validate_shown_head(manifest, files, head_values, &manifest.delegate, CONFLICTS_SHOW_DELEGATE)
}

fn validate_shown_head(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    head_values: &BTreeMap<String, Vec<u8>>,
    head: &ActorHead,
    receipt_name: &str,
) -> Result<(), ConflictEvidenceError> {
    let receipt = &evidence(files, receipt_name)?.json;
    let bytes = head_values
        .get(&head.retained_path)
        .ok_or_else(|| ConflictEvidenceError::Missing(head.retained_path.clone()))?;
    let content = std::str::from_utf8(bytes).map_err(|_| ConflictEvidenceError::Field {
        receipt: head.retained_path.clone(),
        field: "utf8".to_string(),
    })?;
    require_common(receipt_name, receipt, manifest, CONFLICTS_COMMAND, PASS_VERDICT)?;
    require_field(receipt_name, receipt, "conflicted", json!(true))?;
    require_field(receipt_name, receipt, "state_hash", json!(manifest.conflict_state_hash))?;
    require_field(
        receipt_name,
        receipt,
        "shown_head",
        json!({
            "actor": head.actor,
            "artifact": TASKS_ARTIFACT,
            "content": content,
            "content_blake3": head.content_blake3,
            "content_bytes": head.content_bytes,
            "deleted": false,
            "revision_id": head.operation_id,
        }),
    )
}

fn validate_update_refusal(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let receipt = &evidence(files, UPDATE_CONFLICT_NEGATIVE)?.json;
    require_common(UPDATE_CONFLICT_NEGATIVE, receipt, manifest, UPDATE_COMMAND, FAIL_VERDICT)?;
    require_field(UPDATE_CONFLICT_NEGATIVE, receipt, "blocked", json!(true))?;
    require_field(UPDATE_CONFLICT_NEGATIVE, receipt, "dry_run", json!(true))?;
    require_field(UPDATE_CONFLICT_NEGATIVE, receipt, "mutated", json!(false))
}

fn validate_resolution(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let preview = &evidence(files, RESOLVE_PREVIEW)?.json;
    let executed = &evidence(files, RESOLVE_EXECUTE)?.json;
    let expected_selection = json!([{
        "artifact": TASKS_ARTIFACT,
        "revision_id": manifest.resolution.selected_revision,
    }]);
    let expected_basis = json!(manifest.resolution.direct_parents);
    require_common(RESOLVE_PREVIEW, preview, manifest, RESOLVE_COMMAND, PASS_VERDICT)?;
    require_common(RESOLVE_EXECUTE, executed, manifest, RESOLVE_COMMAND, PASS_VERDICT)?;
    require_field(RESOLVE_PREVIEW, preview, "input_state_hash", json!(manifest.conflict_state_hash))?;
    require_field(RESOLVE_PREVIEW, preview, "selected_artifacts", expected_selection.clone())?;
    require_pointer(RESOLVE_PREVIEW, preview, "/actions/0/basis", expected_basis.clone())?;
    require_field(RESOLVE_PREVIEW, preview, "dry_run", json!(true))?;
    require_field(RESOLVE_PREVIEW, preview, "mutated", json!(false))?;
    require_field(RESOLVE_EXECUTE, executed, "selected_artifacts", expected_selection)?;
    require_pointer(RESOLVE_EXECUTE, executed, "/actions/0/basis", expected_basis)?;
    require_field(RESOLVE_EXECUTE, executed, "dry_run", json!(false))?;
    require_field(RESOLVE_EXECUTE, executed, "mutated", json!(true))?;
    require_field(RESOLVE_EXECUTE, executed, "result_operation_id", json!(manifest.resolution.operation_id))?;
    require_field(RESOLVE_EXECUTE, executed, "result_content_root", json!(manifest.resolution.content_root))?;
    require_field(RESOLVE_EXECUTE, executed, "result_state_hash", json!(manifest.resolution.state_hash))?;
    require_field(RESOLVE_EXECUTE, executed, "plan_hash", preview["plan_hash"].clone())
}

fn validate_reload_and_checkout(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let conflicts = &evidence(files, CONFLICTS_RESOLVED_THIRD)?.json;
    let before = &evidence(files, STATUS_THIRD_BEFORE)?.json;
    let checkout_preview = &evidence(files, CHECKOUT_THIRD_PREVIEW)?.json;
    let checkout_execute = &evidence(files, CHECKOUT_THIRD_EXECUTE)?.json;
    let after = &evidence(files, STATUS_THIRD_AFTER)?.json;
    require_common(CONFLICTS_RESOLVED_THIRD, conflicts, manifest, CONFLICTS_COMMAND, PASS_VERDICT)?;
    require_field(CONFLICTS_RESOLVED_THIRD, conflicts, "conflicted", json!(false))?;
    require_field(CONFLICTS_RESOLVED_THIRD, conflicts, "state_hash", json!(manifest.resolution.state_hash))?;
    require_common(STATUS_THIRD_BEFORE, before, manifest, STATUS_COMMAND, PASS_VERDICT)?;
    require_field(STATUS_THIRD_BEFORE, before, "local_destination", json!("absent"))?;
    require_common(CHECKOUT_THIRD_PREVIEW, checkout_preview, manifest, CHECKOUT_COMMAND, PASS_VERDICT)?;
    require_field(CHECKOUT_THIRD_PREVIEW, checkout_preview, "dry_run", json!(true))?;
    require_field(CHECKOUT_THIRD_PREVIEW, checkout_preview, "mutated", json!(false))?;
    require_common(CHECKOUT_THIRD_EXECUTE, checkout_execute, manifest, CHECKOUT_COMMAND, PASS_VERDICT)?;
    require_field(CHECKOUT_THIRD_EXECUTE, checkout_execute, "dry_run", json!(false))?;
    require_field(CHECKOUT_THIRD_EXECUTE, checkout_execute, "mutated", json!(true))?;
    require_field(CHECKOUT_THIRD_EXECUTE, checkout_execute, "post_checkout_destination", json!("current"))?;
    require_pointer(
        CHECKOUT_THIRD_EXECUTE,
        checkout_execute,
        "/files/3/blake3",
        json!(manifest.creator.content_blake3),
    )?;
    require_common(STATUS_THIRD_AFTER, after, manifest, STATUS_COMMAND, PASS_VERDICT)?;
    require_field(STATUS_THIRD_AFTER, after, "state_hash", json!(manifest.resolution.state_hash))?;
    require_field(STATUS_THIRD_AFTER, after, "content_root", json!(manifest.resolution.content_root))?;
    require_pointer(STATUS_THIRD_AFTER, after, "/lifecycle/condition", json!(manifest.resolved_lifecycle_condition))
}

fn validate_transport_and_bundle(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let transport = &evidence(files, TRANSPORT)?.json;
    require_field(TRANSPORT, transport, "cairn_commit", json!(manifest.cairn_commit))?;
    require_field(TRANSPORT, transport, "octet_base_git_oid", json!(manifest.octet_base_git_oid))?;
    require_field(TRANSPORT, transport, "object_id", json!(manifest.object_id))?;
    require_pointer(TRANSPORT, transport, "/creator/actor", json!(manifest.creator.actor))?;
    require_pointer(TRANSPORT, transport, "/creator/ref_name", json!(manifest.creator.ref_name))?;
    require_pointer(TRANSPORT, transport, "/creator/resolved_target", json!(manifest.resolution.operation_id))?;
    require_pointer(TRANSPORT, transport, "/delegate/actor", json!(manifest.delegate.actor))?;
    require_pointer(TRANSPORT, transport, "/delegate/ref_name", json!(manifest.delegate.ref_name))?;
    require_pointer(TRANSPORT, transport, "/delegate/resolved_target", json!(manifest.delegate.operation_id))?;
    require_pointer(TRANSPORT, transport, "/resolution/direct_parents", json!(manifest.resolution.direct_parents))?;
    require_pointer(TRANSPORT, transport, "/resolution/losing_revision_reachable", json!(true))?;
    require_field(TRANSPORT, transport, "source_remotes", json!([]))?;
    require_field(TRANSPORT, transport, "convergence_remotes", json!([]))?;
    require_field(TRANSPORT, transport, "reload_remotes", json!([]))?;
    require_field(TRANSPORT, transport, "valid", json!(true))?;
    let bundle_heads = manifest
        .bundle
        .heads
        .iter()
        .map(|head| (head.ref_name.as_str(), head.target.as_str()))
        .collect::<BTreeMap<_, _>>();
    if bundle_heads.get("refs/heads/main") != Some(&manifest.octet_base_git_oid.as_str())
        || bundle_heads.get(manifest.creator.ref_name.as_str()) != Some(&manifest.resolution.operation_id.as_str())
        || bundle_heads.get(manifest.delegate.ref_name.as_str()) != Some(&manifest.delegate.operation_id.as_str())
    {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "bundle.heads".to_string(),
        });
    }
    Ok(())
}

fn validate_non_claims(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
) -> Result<(), ConflictEvidenceError> {
    let joined = files
        .values()
        .filter_map(|file| file.json.get("non_claims"))
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    for fragment in &manifest.required_non_claim_fragments {
        if !joined.contains(fragment) {
            return Err(ConflictEvidenceError::NonClaim(fragment.clone()));
        }
    }
    Ok(())
}

fn validate_conflict_pilot(
    manifest: &ConflictPilotManifest,
    files: &BTreeMap<String, EvidenceFile>,
    head_values: &BTreeMap<String, Vec<u8>>,
    bundle_bytes: &[u8],
) -> Result<(), ConflictEvidenceError> {
    validate_manifest_shape(manifest)?;
    validate_hashes(manifest, files, head_values, bundle_bytes)?;
    validate_import(manifest, files)?;
    validate_actor_update(
        manifest,
        files,
        &manifest.creator,
        CREATOR_UPDATE_PREVIEW,
        CREATOR_UPDATE_EXECUTE,
        CREATOR_AUTHORITY,
    )?;
    validate_actor_update(
        manifest,
        files,
        &manifest.delegate,
        DELEGATE_UPDATE_PREVIEW,
        DELEGATE_UPDATE_EXECUTE,
        DELEGATE_AUTHORITY,
    )?;
    validate_conflict_inspection(manifest, files, head_values)?;
    validate_update_refusal(manifest, files)?;
    validate_resolution(manifest, files)?;
    validate_reload_and_checkout(manifest, files)?;
    if manifest.resolved_lifecycle_condition != STALE_LIFECYCLE {
        return Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "resolved_lifecycle_condition".to_string(),
        });
    }
    validate_transport_and_bundle(manifest, files)?;
    validate_non_claims(manifest, files)
}

#[test]
fn checked_in_conflict_recovery_evidence_preserves_both_heads() {
    let manifest = load_manifest();
    let files = load_evidence(&manifest);
    let head_values = load_head_values(&manifest);
    let bundle_path = fixture_root().join(&manifest.bundle.path);
    let bundle_bytes = fs::read(&bundle_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", bundle_path.display());
    });
    assert_eq!(validate_conflict_pilot(&manifest, &files, &head_values, &bundle_bytes), Ok(()));
}

#[test]
fn tampered_complete_basis_fails_closed() {
    let mut manifest = load_manifest();
    let files = load_evidence(&manifest);
    let head_values = load_head_values(&manifest);
    let bundle_path = fixture_root().join(&manifest.bundle.path);
    let bundle_bytes = fs::read(&bundle_path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", bundle_path.display());
    });
    manifest.resolution.direct_parents = vec![TAMPERED_OPERATION_ID.to_string()];
    assert_eq!(
        validate_conflict_pilot(&manifest, &files, &head_values, &bundle_bytes),
        Err(ConflictEvidenceError::Field {
            receipt: MANIFEST_FILE.to_string(),
            field: "resolution.direct_parents".to_string(),
        })
    );
}
