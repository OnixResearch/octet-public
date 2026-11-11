use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    apply_workflow_cob_operation, assert_workflow_state_kind, canonical_json_hash,
    decode_base_revision, initialize_workflow_cob, parse_workflow_portable,
    serialize_workflow_portable, validate_workflow_init, workflow_conflict_view,
    workflow_content_root, workflow_state_hash, WorkflowAction, WorkflowEvidenceReference,
    WorkflowFinding, WorkflowInit, WorkflowKind, WorkflowOperation, WorkflowState,
    BASE_REVISION_KIND_GIT_OID, VERIFICATION_OBLIGATION_SCHEMA_V1,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_string_option, with_receipt_hash,
    CapabilityRoot, CapabilityRootKind,
};

const PORTABLE_MAX_BYTES: u64 = 4 * 1024 * 1024;

// r[impl cairn.verification_obligation_cob.cli]
// r[impl cairn.verification_obligation_cob.cli.preview]
// r[impl cairn.verification_obligation_cob.cli.execute]
// r[impl cairn.verification_obligation_cob.profile]
// r[impl cairn.verification_obligation_cob.authority]
// r[impl cairn.verification_obligation_cob.discharge]
// r[impl cairn.verification_obligation_cob.findings]
// r[impl cairn.verification_obligation_cob.claim_revision]
// r[impl cairn.verification_obligation_cob.receipts]
// r[impl cairn.workflow_collaboration.receipts]
// r[impl cairn.workflow_collaboration.validation]
// r[impl cairn.verification_obligation_cob.validation]
// r[impl cairn.verification_obligation_cob.validation.release]
pub(crate) fn run_vo(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = args.first().cloned();
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_deref() {
        Some("import") => run_import(&rest),
        Some("status") => run_status(&rest),
        Some("update") => run_update(&rest),
        Some("evidence") => run_evidence(&rest),
        Some("finding") => run_finding(&rest),
        Some("response") => run_response(&rest),
        Some("discharge") => run_decision(&rest, "discharge"),
        Some("reject") => run_decision(&rest, "reject"),
        Some("invalidate") => run_decision(&rest, "invalidate"),
        Some("conflict") => run_conflict(&rest),
        Some("resolve") => run_resolve(&rest),
        Some("export") => run_export(&rest),
        Some(unknown) => Err(format!("unknown verification-obligation command: {unknown}").into()),
        None => Err("verification-obligation requires import, status, update, evidence, finding, response, discharge, reject, invalidate, conflict, resolve, or export".into()),
    }
}

fn policy_hash() -> Result<String, Box<dyn Error>> {
    let policy = load_policy_file(PathBuf::from(default_policy_path()))?;
    Ok(canonical_json_hash(&policy.raw))
}

fn store_path(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(
        read_string_option(args, "--store")
            .ok_or("verification-obligation requires --store DIR")?,
    ))
}

fn obs_id(args: &[String]) -> Result<String, Box<dyn Error>> {
    read_string_option(args, "--obligation-id")
        .ok_or_else(|| "verification-obligation requires --obligation-id ID".to_string())
        .map_err(Into::into)
}

fn read_portable(store: &Path, id: &str) -> Result<WorkflowState, Box<dyn Error>> {
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, store)?;
    let relative = PathBuf::from(format!("{id}.portable.json"));
    let bytes = root.read_bounded(&relative, PORTABLE_MAX_BYTES)?;
    let text = String::from_utf8(bytes)?;
    parse_workflow_portable(&text).map_err(|e| e.to_string().into())
}

fn write_portable(store: &Path, state: &WorkflowState) -> Result<(), Box<dyn Error>> {
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, store)?;
    let id = &state.obligation_id;
    let portable = serialize_workflow_portable(state).map_err(|e| e.to_string())?;
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

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

// r[impl cairn.verification_obligation_cob.cli.preview]
fn run_import(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_hash = policy_hash()?;
    let store = store_path(args)?;
    let obligation_id = obs_id(args)?;
    let claim = read_string_option(args, "--claim").ok_or("requires --claim CLAIM_ID")?;
    let subject = read_string_option(args, "--subject-root").unwrap_or_else(|| ".".to_string());
    let base =
        read_string_option(args, "--base-revision").ok_or("requires --base-revision KIND:VALUE")?;
    let decoded = decode_base_revision(&base).map_err(|e| e.to_string())?;
    if decoded.kind != BASE_REVISION_KIND_GIT_OID {
        return Err("COB requires a git_oid base revision".into());
    }
    let claim_body = read_string_option(args, "--decision").ok_or("requires --decision FILE")?;
    let content = std::fs::read_to_string(&claim_body)
        .map_err(|e| format!("failed to read {claim_body}: {e}"))?;
    let init = WorkflowInit {
        schema: VERIFICATION_OBLIGATION_SCHEMA_V1.to_string(),
        kind: "verification-obligation".to_string(),
        obligation_id,
        workspace_id: "workspace:vo".to_string(),
        base_revision: base,
        policy_hash,
        claim_root: cairn_core::ClaimRoot {
            claim_identity: claim,
            subject_root: subject,
            revision: 0,
        },
        submitters: actor_set(read_string_option(args, "--submitters")),
        contributors: actor_set(read_string_option(args, "--contributors")),
        verifiers: actor_set(read_string_option(args, "--verifiers")),
        invalidators: actor_set(read_string_option(args, "--invalidators")),
        fields: std::collections::BTreeMap::from([("claim.md".to_string(), content)]),
    };
    validate_workflow_init(&init).map_err(|e| e.to_string())?;
    let state = initialize_workflow_cob(WorkflowOperation {
        id: "init".to_string(),
        actor: String::new(),
        role: None,
        actions: vec![WorkflowAction::Init { state: init }],
    })
    .map_err(|e| e.to_string())?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.vo-import-receipt.v1",
        "command": "verification-obligation import",
        "obligation_id": state.obligation_id,
        "kind": state.kind,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": workflow_state_hash(&state)?,
        "content_identity": workflow_content_root(&state)?,
    })))
}

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let state = read_portable(&store, &id)?;
    assert_workflow_state_kind(&state, WorkflowKind::VerificationObligation)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.vo-status-receipt.v1",
        "command": "verification-obligation status",
        "obligation_id": id,
        "kind": state.kind,
        "claim_root": state.claim_root,
        "lifecycle": state.lifecycle,
        "conflicted": !workflow_conflict_view(&state).is_empty(),
        "conflicts": workflow_conflict_view(&state),
        "findings": state.findings,
        "evidence": state.evidence,
        "state_identity": workflow_state_hash(&state)?,
        "content_identity": workflow_content_root(&state)?,
    })))
}

fn run_conflict(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let state = read_portable(&store, &id)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.vo-conflict-receipt.v1",
        "command": "verification-obligation conflict",
        "obligation_id": id,
        "conflicted": !workflow_conflict_view(&state).is_empty(),
        "conflicts": workflow_conflict_view(&state),
    })))
}

fn mutate(
    args: &[String],
    actions: Vec<WorkflowAction>,
    actor: &str,
    receipt: &str,
) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let mut state = read_portable(&store, &id)?;
    assert_workflow_state_kind(&state, WorkflowKind::VerificationObligation)?;
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("vo-{}", state.operations.len()));
    apply_workflow_cob_operation(
        &mut state,
        WorkflowOperation {
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
        "command": format!("verification-obligation {}", args.first().map(String::as_str).unwrap_or("update")),
        "obligation_id": id,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": workflow_state_hash(&state)?,
        "content_identity": workflow_content_root(&state)?,
    })))
}

fn run_update(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor CONTRIBUTOR")?;
    let text = read_string_option(args, "--text").ok_or("requires --text")?;
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let state = read_portable(&store, &id)?;
    let basis = state.fields["claim.md"].head_ids();
    let actions = vec![WorkflowAction::SetField {
        field: "claim.md".to_string(),
        content: Some(text),
        basis,
    }];
    mutate(args, actions, &actor, "cairn.vo-update-receipt.v1")
}

fn run_evidence(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor CONTRIBUTOR")?;
    let id = read_string_option(args, "--evidence-id").ok_or("requires --evidence-id")?;
    let kind = read_string_option(args, "--evidence-kind").ok_or("requires --evidence-kind")?;
    let role =
        read_string_option(args, "--verification-role").ok_or("requires --verification-role")?;
    let scope = read_string_option(args, "--claim-scope").ok_or("requires --claim-scope")?;
    let external =
        read_string_option(args, "--external-identity").ok_or("requires --external-identity")?;
    let actions = vec![WorkflowAction::AttachEvidence {
        evidence: WorkflowEvidenceReference {
            id,
            evidence_kind: kind,
            verification_role: role,
            claim_scope: scope,
            external_identity: external,
        },
    }];
    mutate(args, actions, &actor, "cairn.vo-evidence-receipt.v1")
}

fn run_finding(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let finding_id = read_string_option(args, "--finding-id").ok_or("requires --finding-id")?;
    let blocker =
        read_string_option(args, "--blocker-class").unwrap_or_else(|| "blocking".to_string());
    let span =
        read_string_option(args, "--source-span").unwrap_or_else(|| "claim.md:1".to_string());
    let actions = vec![WorkflowAction::AddFinding {
        finding: WorkflowFinding {
            id: finding_id,
            status: "open".to_string(),
            blocker_class: blocker,
            source_span: span,
            response: None,
            disposition: None,
        },
    }];
    mutate(args, actions, &actor, "cairn.vo-finding-receipt.v1")
}

fn run_response(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor VERIFIER")?;
    let finding_id = read_string_option(args, "--finding-id").ok_or("requires --finding-id")?;
    let response = read_string_option(args, "--response").ok_or("requires --response")?;
    let actions = vec![WorkflowAction::RespondFinding {
        finding_id,
        response,
    }];
    mutate(args, actions, &actor, "cairn.vo-response-receipt.v1")
}

fn run_decision(args: &[String], decision: &str) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor VERIFIER")?;
    let subject_root =
        read_string_option(args, "--subject-root").unwrap_or_else(|| ".".to_string());
    let actions = match decision {
        "discharge" => vec![WorkflowAction::Discharge {
            decision: "approve".to_string(),
            subject_root,
        }],
        "reject" => vec![WorkflowAction::Reject {
            decision: "reject".to_string(),
            subject_root,
        }],
        "invalidate" => vec![WorkflowAction::Invalidate {
            decision: "invalidate".to_string(),
            subject_root,
        }],
        _ => unreachable!(),
    };
    let receipt = match decision {
        "discharge" => "cairn.vo-discharge-receipt.v1",
        "reject" => "cairn.vo-reject-receipt.v1",
        "invalidate" => "cairn.vo-invalidate-receipt.v1",
        _ => unreachable!(),
    };
    mutate(args, actions, &actor, receipt)
}

fn run_resolve(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let text = read_string_option(args, "--text").ok_or("requires --text")?;
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let state = read_portable(&store, &id)?;
    let basis = state.fields["claim.md"].head_ids();
    let actions = vec![WorkflowAction::ResolveField {
        field: "claim.md".to_string(),
        content: text,
        basis,
    }];
    mutate(args, actions, &actor, "cairn.vo-resolve-receipt.v1")
}

fn run_export(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = obs_id(args)?;
    let state = read_portable(&store, &id)?;
    let portable = serialize_workflow_portable(&state).map_err(|e| e.to_string())?;
    println!("{portable}");
    Ok(())
}
