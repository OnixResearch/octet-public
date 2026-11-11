use std::{
    collections::BTreeSet,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, derive_review_basis, evaluate_decision, parse_release_portable,
    release_candidate_root, release_content_root, release_state_hash, serialize_release_portable,
    validate_release_init, ReleaseCandidate, ReleaseDecisionAction, ReleaseDecisionInit,
    ReleaseDecisionOperation, ReleaseDecisionState, ReleaseEvidenceSummary, ReleaseFinding,
    ReleaseVote, RELEASE_DECISION_SCHEMA_V1,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_string_option, with_receipt_hash,
    CapabilityRoot, CapabilityRootKind,
};

const PORTABLE_MAX_BYTES: u64 = 4 * 1024 * 1024;

// r[impl cairn.release_decision_cob.cli]
// r[impl cairn.release_decision_cob.cli.preview]
// r[impl cairn.release_decision_cob.cli.execute]
// r[impl cairn.release_decision_cob.profile]
// r[impl cairn.release_decision_cob.evidence]
// r[impl cairn.release_decision_cob.blockers]
// r[impl cairn.release_decision_cob.decision]
// r[impl cairn.release_decision_cob.lifecycle]
// r[impl cairn.release_decision_cob.receipts]
// r[impl cairn.release_decision_cob.receipts.approval]
// r[impl cairn.release_decision_cob.receipts.backend]
// r[impl cairn.release_decision_cob.validation]
// r[impl cairn.release_decision_cob.validation.release]
pub(crate) fn run_release(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let command = args.first().cloned();
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    match command.as_deref() {
        Some("create") => run_create(&rest),
        Some("status") => run_status(&rest),
        Some("evidence") => run_evidence(&rest),
        Some("finding") => run_finding(&rest),
        Some("vote") => run_vote(&rest),
        Some("withdraw") => run_decision(&rest, "withdraw"),
        Some("invalidate") => run_decision(&rest, "invalidate"),
        Some("supersede") => run_decision(&rest, "supersede"),
        Some("conflict") => run_conflict(&rest),
        Some("export") => run_export(&rest),
        Some(unknown) => Err(format!("unknown release-decision command: {unknown}").into()),
        None => Err("release-decision requires create, status, evidence, finding, vote, withdraw, invalidate, supersede, conflict, or export".into()),
    }
}

fn policy_hash() -> Result<String, Box<dyn Error>> {
    let policy = load_policy_file(PathBuf::from(default_policy_path()))?;
    Ok(canonical_json_hash(&policy.raw))
}

fn store_path(args: &[String]) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(
        read_string_option(args, "--store").ok_or("release-decision requires --store DIR")?,
    ))
}

fn decision_id(args: &[String]) -> Result<String, Box<dyn Error>> {
    read_string_option(args, "--decision-id")
        .ok_or_else(|| "release-decision requires --decision-id ID".to_string())
        .map_err(Into::into)
}

fn read_portable(store: &Path, id: &str) -> Result<ReleaseDecisionState, Box<dyn Error>> {
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, store)?;
    let relative = PathBuf::from(format!("{id}.portable.json"));
    let bytes = root.read_bounded(&relative, PORTABLE_MAX_BYTES)?;
    let text = String::from_utf8(bytes)?;
    parse_release_portable(&text).map_err(|e| e.to_string().into())
}

fn write_portable(store: &Path, state: &ReleaseDecisionState) -> Result<(), Box<dyn Error>> {
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, store)?;
    let id = &state.decision_id;
    let portable = serialize_release_portable(state).map_err(|e| e.to_string())?;
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

// r[impl cairn.release_decision_cob.cli.preview]
fn run_create(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_identity = policy_hash()?;
    let store = store_path(args)?;
    let decision_id = decision_id(args)?;
    let workspace = read_string_option(args, "--base-revision")
        .ok_or("requires --base-revision git_oid:OID")?;
    if !workspace.starts_with("git_oid:") {
        return Err("release-decision COB requires a git_oid base revision".into());
    }
    let candidate = ReleaseCandidate {
        source_revision: read_string_option(args, "--source-revision")
            .ok_or("requires --source-revision")?,
        artifact_manifest: read_string_option(args, "--artifact-manifest")
            .ok_or("requires --artifact-manifest")?,
        artifact_identity: read_string_option(args, "--artifact-identity")
            .ok_or("requires --artifact-identity")?,
        target_label: read_string_option(args, "--target-label")
            .ok_or("requires --target-label")?,
        release_label: read_string_option(args, "--release-label")
            .ok_or("requires --release-label")?,
    };
    let candidate_root = release_candidate_root(&candidate, &policy_identity)?;
    let init = ReleaseDecisionInit {
        schema: RELEASE_DECISION_SCHEMA_V1.to_string(),
        kind: "release-decision".to_string(),
        decision_id,
        workspace_id: workspace,
        candidate_root,
        policy_identity,
        candidate,
        proposers: actor_set(read_string_option(args, "--proposers")),
        reviewers: actor_set(read_string_option(args, "--reviewers")),
        approvers: actor_set(read_string_option(args, "--approvers")),
        withdrawers: actor_set(read_string_option(args, "--withdrawers")),
        invalidators: actor_set(read_string_option(args, "--invalidators")),
    };
    validate_release_init(&init).map_err(|e| e.to_string())?;
    let state = initialize(init)?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.release-decision-create-receipt.v1",
        "command": "release-decision create",
        "decision_id": state.decision_id,
        "candidate_root": state.candidate_root,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": release_state_hash(&state)?,
        "content_identity": release_content_root(&state)?,
    })))
}

fn initialize(init: ReleaseDecisionInit) -> Result<ReleaseDecisionState, Box<dyn Error>> {
    cairn_core::initialize_release_decision(ReleaseDecisionOperation {
        id: "init".to_string(),
        actor: String::new(),
        role: None,
        actions: vec![ReleaseDecisionAction::Init { state: init }],
    })
    .map_err(|e| -> Box<dyn Error> { e.to_string().into() })
}

fn run_status(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = decision_id(args)?;
    let state = read_portable(&store, &id)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.release-decision-status-receipt.v1",
        "command": "release-decision status",
        "decision_id": id,
        "candidate_root": state.candidate_root,
        "lifecycle": state.lifecycle,
        "review_basis": state.review_basis,
        "votes": state.votes,
        "findings": state.findings,
        "evidence": state.evidence,
        "state_identity": release_state_hash(&state)?,
        "content_identity": release_content_root(&state)?,
    })))
}

fn mutate(
    args: &[String],
    actions: Vec<ReleaseDecisionAction>,
    actor: &str,
    receipt: &str,
) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = decision_id(args)?;
    let mut state = read_portable(&store, &id)?;
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("rd-{}", state.operations.len()));
    cairn_core::apply_release_operation(
        &mut state,
        ReleaseDecisionOperation {
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
        "command": format!("release-decision {}", args.first().map(String::as_str).unwrap_or("update")),
        "decision_id": id,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": release_state_hash(&state)?,
        "content_identity": release_content_root(&state)?,
    })))
}

fn run_evidence(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let id = read_string_option(args, "--evidence-id").ok_or("requires --evidence-id")?;
    let schema = read_string_option(args, "--schema").ok_or("requires --schema")?;
    let role =
        read_string_option(args, "--verification-role").ok_or("requires --verification-role")?;
    let verdict = read_string_option(args, "--verdict").ok_or("requires --verdict")?;
    let external =
        read_string_option(args, "--external-identity").ok_or("requires --external-identity")?;
    let actions = vec![ReleaseDecisionAction::AttachEvidence {
        evidence: ReleaseEvidenceSummary {
            id,
            evidence_identity: external,
            schema,
            verification_role: role,
            claim_scope: read_string_option(args, "--claim-scope")
                .unwrap_or_else(|| "release".to_string()),
            validator_identity: read_string_option(args, "--validator-identity")
                .unwrap_or_else(|| "cairn".to_string()),
            verdict,
            policy_identity: policy_hash()?,
            receipt_identity: read_string_option(args, "--receipt-identity").unwrap_or_default(),
        },
    }];
    mutate(
        args,
        actions,
        &actor,
        "cairn.release-decision-evidence-receipt.v1",
    )
}

fn run_finding(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let finding_id = read_string_option(args, "--finding-id").ok_or("requires --finding-id")?;
    let blocker =
        read_string_option(args, "--blocker-class").unwrap_or_else(|| "blocking".to_string());
    let actions = vec![ReleaseDecisionAction::AddFinding {
        finding: ReleaseFinding {
            id: finding_id,
            category: read_string_option(args, "--category")
                .unwrap_or_else(|| "correctness".to_string()),
            severity: read_string_option(args, "--severity").unwrap_or_else(|| "high".to_string()),
            blocker_class: blocker,
            description: read_string_option(args, "--description")
                .unwrap_or_else(|| "finding".to_string()),
            source_span: read_string_option(args, "--source-span")
                .unwrap_or_else(|| "src:1".to_string()),
            dispositions: Vec::new(),
        },
    }];
    mutate(
        args,
        actions,
        &actor,
        "cairn.release-decision-finding-receipt.v1",
    )
}

fn run_vote(args: &[String]) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let kind = read_string_option(args, "--kind").ok_or("requires --kind approve|reject")?;
    let store = store_path(args)?;
    let id = decision_id(args)?;
    let mut state = read_portable(&store, &id)?;
    let basis = derive_review_basis(&state)?;
    let actions = vec![ReleaseDecisionAction::Vote {
        vote: ReleaseVote {
            id: read_string_option(args, "--vote-id")
                .unwrap_or_else(|| format!("v-{}", state.votes.len())),
            actor: actor.clone(),
            kind,
            review_basis: basis,
        },
    }];
    let op_id = read_string_option(args, "--operation-id")
        .unwrap_or_else(|| format!("rd-{}", state.operations.len()));
    cairn_core::apply_release_operation(
        &mut state,
        ReleaseDecisionOperation {
            id: op_id,
            actor: actor.clone(),
            role: None,
            actions,
        },
    )
    .map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;
    // evaluate the decision (may approve/reject/conflict) as part of the vote
    let verdict =
        evaluate_decision(&mut state).map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;
    let execute = has_flag(args, "--execute");
    if execute {
        write_portable(&store, &state)?;
    }
    print_json(with_receipt_hash(json!({
        "schema": "cairn.release-decision-vote-receipt.v1",
        "command": "release-decision vote",
        "decision_id": id,
        "review_basis": state.review_basis,
        "verdict": verdict,
        "lifecycle": state.lifecycle,
        "dry_run": !execute,
        "executed": execute,
        "state_identity": release_state_hash(&state)?,
    })))
}

fn run_decision(args: &[String], decision: &str) -> Result<(), Box<dyn Error>> {
    let actor = read_string_option(args, "--actor").ok_or("requires --actor")?;
    let reason =
        read_string_option(args, "--reason").unwrap_or_else(|| "bounded reason".to_string());
    let actions = match decision {
        "withdraw" => vec![ReleaseDecisionAction::Withdraw { reason }],
        "invalidate" => vec![ReleaseDecisionAction::Invalidate {
            reason,
            evidence_id: read_string_option(args, "--evidence-id").unwrap_or_default(),
        }],
        "supersede" => vec![ReleaseDecisionAction::Supersede {
            successor: read_string_option(args, "--successor").unwrap_or_default(),
        }],
        _ => unreachable!(),
    };
    let receipt = match decision {
        "withdraw" => "cairn.release-decision-withdraw-receipt.v1",
        "invalidate" => "cairn.release-decision-invalidate-receipt.v1",
        "supersede" => "cairn.release-decision-supersede-receipt.v1",
        _ => unreachable!(),
    };
    mutate(args, actions, &actor, receipt)
}

fn run_conflict(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = decision_id(args)?;
    let state = read_portable(&store, &id)?;
    print_json(with_receipt_hash(json!({
        "schema": "cairn.release-decision-conflict-receipt.v1",
        "command": "release-decision conflict",
        "decision_id": id,
        "lifecycle": state.lifecycle,
        "votes": state.votes,
    })))
}

fn run_export(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store = store_path(args)?;
    let id = decision_id(args)?;
    let state = read_portable(&store, &id)?;
    let portable = serialize_release_portable(&state).map_err(|e| e.to_string())?;
    println!("{portable}");
    Ok(())
}
