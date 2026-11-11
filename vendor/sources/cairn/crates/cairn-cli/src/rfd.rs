use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    assert_state_kind, canonical_json_hash, decode_base_revision, document_conflict_view,
    document_content_root, document_state_hash, initialize_document_cob, parse_document_portable,
    plan_document_materialization, plan_publication_number, serialize_document_portable,
    validate_document_init, DocumentAction, DocumentInit, DocumentKind, DocumentOperation,
    DocumentRole, DocumentState, BASE_REVISION_KIND_GIT_OID, RFD_SCHEMA_V1,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_string_option, with_receipt_hash,
    CapabilityRoot, CapabilityRootKind,
};

const PORTABLE_MAX_BYTES: u64 = 4 * 1024 * 1024;

pub(crate) fn run_rfd(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = args.first().cloned();
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_deref() {
        Some("import") => run_import(&rest),
        Some("status") => run_status(&rest),
        Some("checkout") => run_checkout(&rest),
        Some("update") => run_update(&rest),
        Some("conflict") => run_conflict(&rest),
        Some("resolve") => run_resolve(&rest),
        Some("number") => run_number(&rest),
        Some("publish") => run_publish(&rest),
        Some(unknown) => Err(format!("unknown rfd command: {unknown}").into()),
        None => Err("rfd requires a command: import, status, checkout, update, conflict, resolve, number, or publish".into()),
    }
}

fn policy() -> Result<(cairn_core::Policy, String), Box<dyn Error>> {
    let path = PathBuf::from(default_policy_path());
    let policy = load_policy_file(path)?;
    let policy_hash = canonical_json_hash(&policy.raw);
    Ok((policy, policy_hash))
}

fn store_path(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(
        read_string_option(args, "--store").ok_or("rfd requires --store DIR")?,
    ))
}

fn doc_id(args: &[String]) -> Result<String, Box<dyn Error>> {
    read_string_option(args, "--document-id")
        .ok_or_else(|| "rfd requires --document-id ID".to_string())
        .map_err(Into::into)
}

fn read_portable(store: &Path, id: &str) -> Result<DocumentState, Box<dyn Error>> {
    let path = store.join(format!("{id}.portable.json"));
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, store)?;
    let relative = path.file_name().ok_or("invalid store path")?;
    let bytes = root.read_bounded(&PathBuf::from(relative), PORTABLE_MAX_BYTES)?;
    let text = String::from_utf8(bytes)?;
    parse_document_portable(&text).map_err(|e| e.to_string().into())
}

// r[impl cairn.rfd_cob.cli]
// r[impl cairn.rfd_cob.cli.preview]
// r[impl cairn.rfd_cob.cli.execute]
fn run_import(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_policy, policy_hash) = policy()?;
    let store = store_path(args)?;
    let id = doc_id(args)?;
    if id.is_empty() || id.len() > 256 {
        return Err("document id is empty or too long".into());
    }
    let workspace =
        read_string_option(args, "--workspace").unwrap_or_else(|| "workspace:rfd".to_string());
    let base = read_string_option(args, "--base-revision")
        .ok_or("rfd import requires --base-revision KIND:VALUE")?;
    let decoded = decode_base_revision(&base).map_err(|e| e.to_string())?;
    if decoded.kind != BASE_REVISION_KIND_GIT_OID {
        return Err("rfd import COB requires a git_oid base revision".into());
    }
    let authors = parse_actor_set(read_string_option(args, "--authors"));
    let participants = parse_actor_set(read_string_option(args, "--participants"));
    let reviewers = parse_actor_set(read_string_option(args, "--reviewers"));
    let approvers = parse_actor_set(read_string_option(args, "--approvers"));
    let publishers = parse_actor_set(read_string_option(args, "--publishers"));
    let spec = read_string_option(args, "--spec").ok_or("rfd import requires --spec PATH")?;
    let spec_content = read_option_path(&spec)?;
    let init = DocumentInit {
        schema: RFD_SCHEMA_V1.to_string(),
        kind: "rfd".to_string(),
        document_id: id.clone(),
        workspace_id: workspace,
        base_revision: base,
        policy_hash,
        authors,
        participants,
        reviewers,
        approvers,
        publishers,
        artifacts: std::collections::BTreeMap::from([("spec.md".to_string(), spec_content)]),
    };
    validate_document_init(&init).map_err(|e| e.to_string())?;
    let actor = read_string_option(args, "--actor")
        .unwrap_or_else(|| init.authors.iter().next().cloned().unwrap_or_default());
    let operation = DocumentOperation {
        id: "init".to_string(),
        actor,
        role: None,
        actions: vec![DocumentAction::Init { state: init }],
    };
    let state = initialize_document_cob(operation).map_err(|e| e.to_string())?;
    let portable = serialize_document_portable(&state).map_err(|e| e.to_string())?;
    let execute = has_flag(args, "--execute");
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, &store)?;
    let out = format!("{id}.portable.json");
    if execute {
        root.write_new(&PathBuf::from(&out), portable.as_bytes())?;
    }
    let value = with_receipt_hash(json!({
        "schema": "cairn.rfd-import-receipt.v1",
        "command": "rfd import",
        "document_id": state.document_id,
        "kind": state.kind,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
        "store_path": out,
        "non_claims": ["document import does not prove publication, synchronization, or remote uniqueness"],
    }));
    print_json(value)
}

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    assert_state_kind(&state, DocumentKind::Rfd)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-status-receipt.v1",
        "command": "rfd status",
        "document_id": id,
        "kind": state.kind,
        "lifecycle": state.lifecycle,
        "conflicted": !document_conflict_view(&state).is_empty(),
        "conflicts": document_conflict_view(&state),
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
        "publication": state.publication,
        "discussion_count": state.discussion.len(),
        "link_count": state.links.len(),
        "artifact_count": state.artifacts.len(),
    })))
}

fn run_conflict(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let conflicts = document_conflict_view(&state);
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-conflict-receipt.v1",
        "command": "rfd conflict",
        "document_id": id,
        "conflicted": !conflicts.is_empty(),
        "conflicts": conflicts,
    })))
}

fn run_checkout(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let state_dir = args
        .iter()
        .position(|a| a == "--out")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .ok_or("rfd checkout requires --out DIR")?;
    let planned = plan_document_materialization(&state, ".", 4096).map_err(|e| e.to_string())?;
    let execute = has_flag(args, "--execute");
    let root =
        CapabilityRoot::open_or_create(CapabilityRootKind::Repository, &PathBuf::from(&state_dir))?;
    if execute {
        for file in &planned {
            let path = Path::new(&file.destination);
            if let Some(parent) = path.parent() {
                if !parent.as_os_str().is_empty() && parent != Path::new(".") {
                    root.create_dir_all(parent)?;
                }
            }
            root.write(path, file.content.as_bytes())?;
        }
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-checkout-receipt.v1",
        "command": "rfd checkout",
        "document_id": id,
        "dry_run": !execute,
        "executed": execute,
        "files": planned,
        "non_claims": ["checkout preserves canonical publication content and is not a remote proof"],
    })))
}

fn run_update(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let mut state = read_portable(&store, &id)?;
    let actor = read_string_option(args, "--actor").ok_or("rfd update requires --actor ID")?;
    let kind = read_string_option(args, "--kind").unwrap_or_else(|| "discussion".to_string());
    let text = read_string_option(args, "--text");
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("update-{}", state.operations.len()));
    let actions = match kind.as_str() {
        "discussion" => {
            let text = text.ok_or("discussion update requires --text")?;
            vec![DocumentAction::AddDiscussion {
                record: cairn_core::DiscussionRecord {
                    id: format!("{op_id}:m"),
                    actor: actor.clone(),
                    text,
                    reply_to: read_string_option(args, "--reply-to"),
                },
                basis: BTreeSet::new(),
            }]
        }
        "artifact" => {
            author_role(&state, &actor).ok_or("actor is not an author")?;
            let content = text.ok_or("artifact update requires --text")?;
            let basis = current_artifact_heads(&state);
            vec![DocumentAction::SetArtifact {
                artifact: "spec.md".to_string(),
                content: Some(content),
                basis,
            }]
        }
        other => return Err(format!("unsupported rfd update kind: {other}").into()),
    };
    let operation = DocumentOperation {
        id: op_id,
        actor,
        role: role_for(&actions),
        actions,
    };
    apply_update(&mut state, operation)?;
    let portable = serialize_document_portable(&state).map_err(|e| e.to_string())?;
    let execute = has_flag(args, "--execute");
    if execute {
        let root = CapabilityRoot::open(CapabilityRootKind::LocalState, &store)?;
        root.write(
            &PathBuf::from(format!("{id}.portable.json")),
            portable.as_bytes(),
        )?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-update-receipt.v1",
        "command": "rfd update",
        "document_id": id,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": document_state_hash(&state)?,
        "content_identity": document_content_root(&state)?,
    })))
}

fn run_resolve(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let mut state = read_portable(&store, &id)?;
    let actor = read_string_option(args, "--actor").ok_or("rfd resolve requires --actor ID")?;
    if author_role(&state, &actor).is_none() {
        return Err("actor is not an author".into());
    }
    let heads = current_artifact_heads(&state);
    if heads.len() < 2 {
        return Err("rfd resolve requires multiple current artifact heads".into());
    }
    let content = read_string_option(args, "--text").ok_or("rfd resolve requires --text")?;
    let op_id = read_string_option(args, "--operation-id").unwrap_or_else(|| "resolve".to_string());
    let operation = DocumentOperation {
        id: op_id,
        actor,
        role: Some(DocumentRole::Author),
        actions: vec![DocumentAction::SetArtifact {
            artifact: "spec.md".to_string(),
            content: Some(content),
            basis: heads,
        }],
    };
    apply_update(&mut state, operation)?;
    if has_flag(args, "--execute") {
        let portable = serialize_document_portable(&state).map_err(|e| e.to_string())?;
        let root = CapabilityRoot::open(CapabilityRootKind::LocalState, &store)?;
        root.write(
            &PathBuf::from(format!("{id}.portable.json")),
            portable.as_bytes(),
        )?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-resolve-receipt.v1",
        "command": "rfd resolve",
        "document_id": id,
        "conflicted": !document_conflict_view(&state).is_empty(),
        "state_identity": document_state_hash(&state)?,
    })))
}

fn run_number(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = doc_id(args)?;
    let state = read_portable(&store, &id)?;
    let number = read_number(args, "--number")?;
    let registry = read_string_option(args, "--registry")
        .ok_or("rfd number requires --registry SNAPSHOT_IDENTITY")?;
    let publisher =
        read_string_option(args, "--publisher").ok_or("rfd number requires --publisher ACTOR")?;
    let plan = plan_publication_number(&state, number, &registry, &publisher)
        .map_err(|e| e.to_string())?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.rfd-number-receipt.v1",
        "command": "rfd number",
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
        "schema": "cairn.rfd-publish-plan-receipt.v1",
        "command": "rfd publish",
        "document_id": id,
        "content_identity": document_content_root(&state)?,
        "out": out_dir,
        "files": planned,
        "non_claims": ["publication plan preserves canonical content and does not prove remote publication"],
    })))
}

fn apply_update(
    state: &mut DocumentState,
    operation: DocumentOperation,
) -> Result<(), Box<dyn Error>> {
    cairn_core::apply_document_cob_operation(state, operation).map_err(|e| e.to_string().into())
}

fn current_artifact_heads(state: &DocumentState) -> BTreeSet<String> {
    state
        .artifacts
        .get("spec.md")
        .map(cairn_core::DocumentRevisionSet::head_ids)
        .unwrap_or_default()
}

fn author_role(state: &DocumentState, actor: &str) -> Option<DocumentRole> {
    if state.authors.contains(actor) {
        Some(DocumentRole::Author)
    } else {
        None
    }
}

fn role_for(actions: &[DocumentAction]) -> Option<DocumentRole> {
    actions.first().and_then(|action| match action {
        DocumentAction::SetArtifact { .. } => Some(DocumentRole::Author),
        DocumentAction::AddDiscussion { .. } => Some(DocumentRole::Participant),
        _ => None,
    })
}

fn parse_actor_set(value: Option<String>) -> BTreeSet<String> {
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

fn read_number(args: &[String], flag: &str) -> Result<u64, Box<dyn Error>> {
    read_string_option(args, flag)
        .ok_or_else(|| format!("rfd requires {flag} NUMBER"))?
        .parse::<u64>()
        .map_err(|_| "number must be a positive integer".into())
}

fn read_option_path(path: &str) -> Result<String, Box<dyn Error>> {
    let path = PathBuf::from(path);
    std::fs::read_to_string(&path).map_err(|e| format!("failed to read {path:?}: {e}").into())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}
