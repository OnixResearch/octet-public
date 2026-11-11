use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    apply_exception_operation, canonical_json_hash, classify_applicability, exception_content_root,
    exception_request_root, exception_state_hash, initialize_exception, parse_exception_portable,
    serialize_exception_portable, validate_exception_init, validate_scope, ExceptionRequest,
    PolicyExceptionAction, PolicyExceptionInit, PolicyExceptionState, PolicyScope,
    ValidityObservation, POLICY_EXCEPTION_SCHEMA_V1,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_string_option, with_receipt_hash,
    CapabilityRoot, CapabilityRootKind,
};

const PORTABLE_MAX_BYTES: u64 = 4 * 1024 * 1024;

// r[impl cairn.policy_exception_cob.cli]
// r[impl cairn.policy_exception_cob.cli.preview]
// r[impl cairn.policy_exception_cob.cli.execute]
// r[impl cairn.policy_exception_cob.authority]
// r[impl cairn.policy_exception_cob.approval]
// r[impl cairn.policy_exception_cob.validity]
// r[impl cairn.policy_exception_cob.applicability]
// r[impl cairn.policy_exception_cob.receipts]
// r[impl cairn.policy_exception_cob.receipts.backend]
// r[impl cairn.policy_exception_cob.validation]
// r[impl cairn.policy_exception_cob.validation.release]
pub(crate) fn run_policy_exception(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = args.first().cloned();
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_deref() {
        Some("request") => run_request(&rest),
        Some("status") => run_status(&rest),
        Some("approve") => run_decision(&rest, "approve"),
        Some("reject") => run_decision(&rest, "reject"),
        Some("observe-validity") => run_observe(&rest),
        Some("revoke") => run_decision(&rest, "revoke"),
        Some("expire") => run_decision(&rest, "expire"),
        Some("close") => run_decision(&rest, "close"),
        Some("supersede") => run_decision(&rest, "supersede"),
        Some("conflict") => run_conflict(&rest),
        Some("export") => run_export(&rest),
        Some(unknown) => Err(format!("unknown policy-exception command: {unknown}").into()),
        None => Err("policy-exception requires request, status, approve, reject, observe-validity, revoke, expire, close, supersede, conflict, or export".into()),
    }
}

fn policy_hash() -> Result<String, Box<dyn Error>> {
    let policy = load_policy_file(PathBuf::from(default_policy_path()))?;
    Ok(canonical_json_hash(&policy.raw))
}

fn store_path(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(
        read_string_option(args, "--store").ok_or("policy-exception requires --store DIR")?,
    ))
}

fn exc_id(args: &[String]) -> Result<String, Box<dyn Error>> {
    read_string_option(args, "--exception-id")
        .ok_or_else(|| "policy-exception requires --exception-id ID".to_string())
        .map_err(Into::into)
}

fn read_portable(store: &Path, id: &str) -> Result<PolicyExceptionState, Box<dyn Error>> {
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, store)?;
    let relative = PathBuf::from(format!("{id}.portable.json"));
    let bytes = root.read_bounded(&relative, PORTABLE_MAX_BYTES)?;
    let text = String::from_utf8(bytes)?;
    parse_exception_portable(&text).map_err(|e| e.to_string().into())
}

fn write_portable(store: &Path, state: &PolicyExceptionState) -> Result<(), Box<dyn Error>> {
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, store)?;
    let id = &state.exception_id;
    let portable = serialize_exception_portable(state).map_err(|e| e.to_string())?;
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

// r[impl cairn.policy_exception_cob.cli.preview]
fn run_request(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_identity = policy_hash()?;
    let store = store_path(args)?;
    let exception_id = exc_id(args)?;
    let workspace = read_string_option(args, "--base-revision")
        .ok_or("requires --base-revision git_oid:OID")?;
    if !workspace.starts_with("git_oid:") {
        return Err("policy-exception COB requires a git_oid base revision".into());
    }
    let rules = read_string_option(args, "--rules")
        .ok_or("requires --rules rule1,rule2")?
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let targets = read_string_option(args, "--targets")
        .ok_or("requires --targets t1,t2")?
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let scope = PolicyScope {
        rule_ids: rules,
        targets,
    };
    validate_scope(&scope).map_err(|e| e.to_string())?;
    let rationale = read_string_option(args, "--rationale").ok_or("requires --rationale")?;
    let controls = read_string_option(args, "--controls")
        .map(|c| {
            c.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .ok_or("requires --controls control1,control2")?;
    let request = ExceptionRequest {
        policy_identity,
        scope,
        rationale,
        compensating_controls: controls,
        request_root: String::new(),
    };
    let request_root = exception_request_root(&request)?;
    let mut request = request;
    request.request_root = request_root;
    let init = PolicyExceptionInit {
        schema: POLICY_EXCEPTION_SCHEMA_V1.to_string(),
        kind: "policy-exception".to_string(),
        exception_id,
        workspace_id: workspace,
        request,
        requesters: actor_set(read_string_option(args, "--requesters")),
        reviewers: actor_set(read_string_option(args, "--reviewers")),
        approvers: actor_set(read_string_option(args, "--approvers")),
        revokers: actor_set(read_string_option(args, "--revokers")),
        observers: actor_set(read_string_option(args, "--observers")),
    };
    validate_exception_init(&init).map_err(|e| e.to_string())?;
    let state = initialize_exception(cairn_core::PolicyExceptionOperation {
        id: "init".to_string(),
        actor: String::new(),
        role: None,
        actions: vec![PolicyExceptionAction::Init {
            state: Box::new(init),
        }],
    })
    .map_err(|e| e.to_string())?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.policy-exception-request-receipt.v1",
        "command": "policy-exception request",
        "exception_id": state.exception_id,
        "request_root": state.request.request_root,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": exception_state_hash(&state)?,
        "content_identity": exception_content_root(&state)?,
    })))
}

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = exc_id(args)?;
    let state = read_portable(&store, &id)?;
    let applicability = classify_applicability(&state).unwrap_or_else(|e| e.to_string());
    print_json(with_receipt_hash(json!({
        "schema": "cairn.policy-exception-status-receipt.v1",
        "command": "policy-exception status",
        "exception_id": id,
        "request_root": state.request.request_root,
        "request_revision": state.request_revision,
        "lifecycle": state.lifecycle,
        "applicability": applicability,
        "approvals": state.approvals,
        "observations": state.observations,
        "state_identity": exception_state_hash(&state)?,
        "content_identity": exception_content_root(&state)?,
    })))
}

fn mutate(
    args: &[String],
    actions: Vec<PolicyExceptionAction>,
    actor: &str,
    receipt: &str,
) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = exc_id(args)?;
    let mut state = read_portable(&store, &id)?;
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("pe-{}", state.operations.len()));
    apply_exception_operation(
        &mut state,
        cairn_core::PolicyExceptionOperation {
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
        "command": format!("policy-exception {}", args.first().map(String::as_str).unwrap_or("update")),
        "exception_id": id,
        "lifecycle": state.lifecycle,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": exception_state_hash(&state)?,
        "content_identity": exception_content_root(&state)?,
    })))
}

fn run_decision(args: &[String], decision: &str) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let reason =
        read_string_option(args, "--reason").unwrap_or_else(|| "bounded reason".to_string());
    let actions = match decision {
        "approve" => vec![PolicyExceptionAction::Approve {
            scope_root: scope_root_arg(args)?,
        }],
        "reject" => vec![PolicyExceptionAction::Reject { reason }],
        "revoke" => vec![PolicyExceptionAction::Revoke { reason }],
        "expire" => vec![PolicyExceptionAction::Expire { reason }],
        "close" => vec![PolicyExceptionAction::Close { reason }],
        "supersede" => vec![PolicyExceptionAction::Supersede {
            successor: read_string_option(args, "--successor")
                .ok_or("supersede requires --successor")?,
        }],
        _ => unreachable!(),
    };
    let receipt = match decision {
        "approve" => "cairn.policy-exception-approve-receipt.v1",
        "reject" => "cairn.policy-exception-reject-receipt.v1",
        "revoke" => "cairn.policy-exception-revoke-receipt.v1",
        "expire" => "cairn.policy-exception-expire-receipt.v1",
        "close" => "cairn.policy-exception-close-receipt.v1",
        "supersede" => "cairn.policy-exception-supersede-receipt.v1",
        _ => unreachable!(),
    };
    mutate(args, actions, &actor, receipt)
}

fn scope_root_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    let store = store_path(args)?;
    let id = exc_id(args)?;
    let state = read_portable(&store, &id)?;
    exception_request_root(&state.request).map_err(|e| e.to_string().into())
}

fn run_observe(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor OBSERVER")?;
    let observed = read_string_option(args, "--observed-epoch")
        .ok_or("requires --observed-epoch")?
        .parse::<u64>()
        .map_err(|_| "observed-epoch must be a number")?;
    let until = read_string_option(args, "--finite-valid-until")
        .ok_or("requires --finite-valid-until")?
        .parse::<u64>()
        .map_err(|_| "finite-valid-until must be a number")?;
    let actions = vec![PolicyExceptionAction::ObserveValidity {
        observation: ValidityObservation {
            id: read_string_option(args, "--observation-id").unwrap_or_else(|| "obs".to_string()),
            observed_epoch: observed,
            finite_valid_until: until,
            source: actor.clone(),
        },
    }];
    mutate(
        args,
        actions,
        &actor,
        "cairn.policy-exception-observation-receipt.v1",
    )
}

fn run_conflict(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = exc_id(args)?;
    let state = read_portable(&store, &id)?;
    let applicability = classify_applicability(&state).unwrap_or_else(|e| e.to_string());
    print_json(with_receipt_hash(json!({
        "schema": "cairn.policy-exception-conflict-receipt.v1",
        "command": "policy-exception conflict",
        "exception_id": id,
        "lifecycle": state.lifecycle,
        "applicability": applicability,
        "approvals": state.approvals,
    })))
}

fn run_export(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = exc_id(args)?;
    let state = read_portable(&store, &id)?;
    let portable = serialize_exception_portable(&state).map_err(|e| e.to_string())?;
    println!("{portable}");
    Ok(())
}
