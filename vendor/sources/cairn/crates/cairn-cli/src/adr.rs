use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, decode_base_revision, document_conflict_view, document_conflicted,
    document_content_root, document_state_hash, parse_document_portable,
    plan_document_materialization, plan_publication_number, serialize_document_portable,
    validate_document_init, DocumentAction, DocumentInit, DocumentKind, DocumentLifecycleStatus,
    DocumentOperation, DocumentState, ADR_SCHEMA_V1, BASE_REVISION_KIND_GIT_OID,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_string_option, with_receipt_hash,
    CapabilityRoot, CapabilityRootKind,
};

const PORTABLE_MAX_BYTES: u64 = 4 * 1024 * 1024;

// r[impl cairn.adr_cob.cli]
// r[impl cairn.adr_cob.cli.preview]
// r[impl cairn.adr_cob.cli.execute]
// r[impl cairn.adr_cob.profile]
// r[impl cairn.adr_cob.materialization]
// r[impl cairn.adr_cob.materialization.unsafe]
// r[impl cairn.adr_cob.receipts]
// r[impl cairn.adr_cob.receipts.acceptance]
// r[impl cairn.adr_cob.receipts.backend]
pub(crate) fn run_adr(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = args.first().cloned();
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_deref() {
        Some("import") => run_import(&rest),
        Some("status") => run_status(&rest),
        Some("update") => run_update(&rest),
        Some("accept") => run_accept(&rest),
        Some("deprecate") => run_deprecate(&rest),
        Some("supersede") => run_supersede(&rest),
        Some("conflict") => run_conflict(&rest),
        Some("resolve") => run_resolve(&rest),
        Some("number") => run_number(&rest),
        Some("publish") => run_publish(&rest),
        Some(unknown) => Err(format!("unknown adr command: {unknown}").into()),
        None => Err("adr requires import, status, update, accept, deprecate, supersede, conflict, resolve, number, or publish".into()),
    }
}

fn policy_hash() -> Result<String, Box<dyn Error>> {
    let policy = load_policy_file(PathBuf::from(default_policy_path()))?;
    Ok(canonical_json_hash(&policy.raw))
}

fn store_path(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(
        read_string_option(args, "--store").ok_or("adr requires --store DIR")?,
    ))
}

fn doc_id(args: &[String]) -> Result<String, Box<dyn Error>> {
    read_string_option(args, "--document-id")
        .ok_or_else(|| "adr requires --document-id ID".to_string())
        .map_err(Into::into)
}

fn read_portable(store: &Path, id: &str) -> Result<DocumentState, Box<dyn Error>> {
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, store)?;
    let relative = PathBuf::from(format!("{id}.portable.json"));
    let bytes = root.read_bounded(&relative, PORTABLE_MAX_BYTES)?;
    let text = String::from_utf8(bytes)?;
    parse_document_portable(&text).map_err(|e| e.to_string().into())
}

fn write_portable(store: &Path, state: &DocumentState) -> Result<(), Box<dyn Error>> {
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, store)?;
    let id = &state.document_id;
    let portable = serialize_document_portable(state).map_err(|e| e.to_string())?;
    root.write(
        &PathBuf::from(format!("{id}.portable.json")),
        portable.as_bytes(),
    )?;
    Ok(())
}

fn actor_set(value: Option<String>) -> BTreeSet<String> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

// r[impl cairn.adr_cob.profile]
// r[impl cairn.adr_cob.cli.preview]
fn run_import(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_hash = policy_hash()?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let workspace =
        read_string_option(args, "--workspace").unwrap_or_else(|| "workspace:adr".to_string());
    let base = read_string_option(args, "--base-revision")
        .ok_or("adr import requires --base-revision KIND:VALUE")?;
    let decoded = decode_base_revision(&base).map_err(|e| e.to_string())?;
    if decoded.kind != BASE_REVISION_KIND_GIT_OID {
        return Err("adr import COB requires a git_oid base revision".into());
    }
    let authors = actor_set(read_string_option(args, "--authors"));
    let reviewers = actor_set(read_string_option(args, "--reviewers"));
    let approvers = actor_set(read_string_option(args, "--approvers"));
    let publishers = actor_set(read_string_option(args, "--publishers"));
    let decision =
        read_string_option(args, "--decision").ok_or("adr import requires --decision FILE")?;
    let content = std::fs::read_to_string(&decision)
        .map_err(|e| format!("failed to read {decision}: {e}"))?;
    let init = DocumentInit {
        schema: ADR_SCHEMA_V1.to_string(),
        kind: "adr".to_string(),
        document_id: id,
        workspace_id: workspace,
        base_revision: base,
        policy_hash,
        authors,
        participants: BTreeSet::from(["participant".to_string()]),
        reviewers,
        approvers,
        publishers,
        artifacts: std::collections::BTreeMap::from([("decision.md".to_string(), content)]),
    };
    validate_document_init(&init).map_err(|e| e.to_string())?;
    let state = initialize_adr_state(init)?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.adr-import-receipt.v1",
        "command": "adr import",
        "document_id": state.document_id,
        "kind": state.kind,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
    })))
}

fn initialize_adr_state(init: DocumentInit) -> Result<DocumentState, Box<dyn Error>> {
    initialize_adr_state_inner(init).map_err(|e| e.to_string().into())
}

fn initialize_adr_state_inner(
    init: DocumentInit,
) -> Result<DocumentState, cairn_core::DocumentCobError> {
    cairn_core::initialize_document_cob(DocumentOperation {
        id: "init".to_string(),
        actor: String::new(),
        role: None,
        actions: vec![DocumentAction::Init { state: init }],
    })
}

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    cairn_core::assert_state_kind(&state, DocumentKind::Adr)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.adr-status-receipt.v1",
        "command": "adr status",
        "document_id": id,
        "kind": state.kind,
        "lifecycle": state.lifecycle,
        "conflicted": !document_conflict_view(&state).is_empty(),
        "conflicts": document_conflict_view(&state),
        "publication": state.publication,
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
    })))
}

fn run_conflict(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let conflicts = document_conflict_view(&state);
    print_json(with_receipt_hash(json!({
        "schema": "cairn.adr-conflict-receipt.v1",
        "command": "adr conflict",
        "document_id": id,
        "conflicted": !conflicts.is_empty(),
        "conflicts": conflicts,
    })))
}

fn apply(
    args: &[String],
    actions: Vec<DocumentAction>,
    actor: &str,
    receipt: &str,
) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let mut state = read_portable(&store, &id)?;
    cairn_core::assert_state_kind(&state, DocumentKind::Adr)?;
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("adr-{}", state.operations.len()));
    cairn_core::apply_document_cob_operation(
        &mut state,
        DocumentOperation {
            id: op_id,
            actor: actor.to_string(),
            role: None,
            actions,
        },
    )
    .map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": receipt,
        "command": format!("adr {}", args.first().map(String::as_str).unwrap_or("update")),
        "document_id": id,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
    })))
}

fn run_update(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("adr update requires --actor ACTOR")?;
    let text = read_string_option(args, "--text").ok_or("adr update requires --text")?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let basis = current_heads(&state);
    let actions = vec![DocumentAction::SetArtifact {
        artifact: "decision.md".to_string(),
        content: Some(text),
        basis,
    }];
    apply(args, actions, &actor, "cairn.adr-update-receipt.v1")
}

fn run_accept(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor =
        read_string_option(args, "--actor").ok_or("adr accept requires --actor APPROVER")?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    if document_conflicted(&state) {
        return Err("conflicted ADR cannot be accepted".into());
    }
    let subject_root = document_content_root(&state)?;
    let actions = vec![DocumentAction::SetLifecycle {
        status: DocumentLifecycleStatus::Accepted,
        subject_root,
        basis: state.lifecycle.basis,
    }];
    // Acceptance is preview-first and requires the approver role via the reducer.
    apply(args, actions, &actor, "cairn.adr-accept-receipt.v1")
}

fn run_deprecate(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("adr deprecate requires --actor")?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let subject_root = document_content_root(&state)?;
    let actions = vec![DocumentAction::SetLifecycle {
        status: DocumentLifecycleStatus::Deprecated,
        subject_root,
        basis: state.lifecycle.basis,
    }];
    apply(args, actions, &actor, "cairn.adr-deprecate-receipt.v1")
}

fn run_supersede(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("adr supersede requires --actor")?;
    let relation =
        read_string_option(args, "--replaces").ok_or("adr supersede requires --replaces ADR_ID")?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let subject_root = document_content_root(&state)?;
    let link = cairn_core::DocumentLink {
        id: format!("supersede:{relation}"),
        target_kind: "adr".to_string(),
        target_id: relation,
        relation: "supersedes".to_string(),
        expected_content_root: None,
    };
    let actions = vec![
        DocumentAction::AddLink {
            link,
            basis: BTreeSet::new(),
        },
        DocumentAction::SetLifecycle {
            status: DocumentLifecycleStatus::Superseded,
            subject_root,
            basis: state.lifecycle.basis,
        },
    ];
    apply(args, actions, &actor, "cairn.adr-supersede-receipt.v1")
}

fn run_resolve(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("adr resolve requires --actor")?;
    let text = read_string_option(args, "--text").ok_or("adr resolve requires --text")?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let heads = current_heads(&state);
    if heads.len() < 2 {
        return Err("adr resolve requires multiple current heads".into());
    }
    let actions = vec![DocumentAction::SetArtifact {
        artifact: "decision.md".to_string(),
        content: Some(text),
        basis: heads,
    }];
    apply(args, actions, &actor, "cairn.adr-resolve-receipt.v1")
}

fn run_number(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let number = read_string_option(args, "--number")
        .ok_or("adr number requires --number N")?
        .parse::<u64>()
        .map_err(|_| "number must be a positive integer")?;
    let registry = read_string_option(args, "--registry")
        .ok_or("adr number requires --registry SNAPSHOT_ID")?;
    let publisher =
        read_string_option(args, "--publisher").ok_or("adr number requires --publisher")?;
    let plan = plan_publication_number(&state, number, &registry, &publisher)
        .map_err(|e| e.to_string())?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.adr-number-receipt.v1",
        "command": "adr number",
        "document_id": id,
        "publication": plan,
        "non_claims": ["local registry comparison does not prove global uniqueness or remote reservation"],
    })))
}

fn run_publish(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let out_dir = read_string_option(args, "--out");
    let planned = plan_document_materialization(&state, ".", 4096).map_err(|e| e.to_string())?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.adr-publish-plan-receipt.v1",
        "command": "adr publish",
        "document_id": id,
        "out": out_dir,
        "content_identity": document_content_root(&state)?,
        "files": planned,
        "non_claims": ["publication plan preserves canonical content and does not prove remote publication"],
    })))
}

fn current_heads(state: &DocumentState) -> BTreeSet<String> {
    state
        .artifacts
        .get("decision.md")
        .map(cairn_core::DocumentRevisionSet::head_ids)
        .unwrap_or_default()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}
