pub(crate) mod materialize;
pub(crate) mod portable_store;
pub(crate) mod resolution;

use std::{error::Error, path::PathBuf};

use cairn_core::{
    apply_planned_change_collaboration_update, build_portable_change_collaboration,
    canonical_json_hash, change_collaboration_update_plan_hash,
    initialize_change_collaboration_import, materialize_change_collaboration,
    plan_change_collaboration_checkout, plan_change_collaboration_import,
    plan_change_collaboration_update, validate_change_name, ChangeBaseRevision,
    ChangeCheckoutDestination, ChangeCollaborationCheckoutPlan, ChangeCollaborationImportSnapshot,
    ChangeCollaborationState, ChangeCollaborationUpdatePlan, ChangeCollaborationUpdateSnapshot,
    ChangeLifecycleView, PortableChangeCollaboration, PORTABLE_COLLABORATION_BACKEND_V1,
};
use serde_json::{json, Value};

use crate::{
    radicle_change::{run_cob, status_checkout::read_local_change_snapshot},
    support::{
        default_policy_path, load_policy_file, path_string, print_json, with_portable_receipt_hash,
    },
};

use self::{
    materialize::{
        materialize_collaboration_checkout, MaterializationOutcome, RepositoryCheckoutLock,
    },
    portable_store::{
        commit_store, load_store, read_active_change_artifacts, replace_store, store_exists,
        PortableStoreLock,
    },
};

const COB_BACKEND_V1: &str = "radicle-cob.v1";
const OPTION_TOKEN_WIDTH: usize = 2;
const NON_CLAIM_AUTHORITY: &str =
    "portable actor and workspace values are caller declarations, not cryptographic identity or remote authority proof";
const NON_CLAIM_LOCAL: &str =
    "portable collaboration proves bounded local envelope and filesystem observations only";
const NON_CLAIM_LIFECYCLE: &str =
    "collaboration import, status, checkout, and update do not prove gate validity, evidence truth, dependency readiness, source correctness, archive execution, or release eligibility";
const NON_CLAIM_TRANSPORT: &str =
    "a portable store does not prove network publication, peer synchronization, availability, or remote trust";
const NON_CLAIM_CONFLICT: &str =
    "ordinary collaboration update refuses conflicts and does not prove or perform conflict resolution";
const BLOCKER_STORE_EXISTS: &str = "portable collaboration store already exists";
const BLOCKER_INPUT_DRIFT: &str =
    "active change, policy, portable state, or local destination changed after preview";
const BLOCKER_CHANGE_MISMATCH: &str =
    "portable collaboration store contains a different change identifier";

// r[impl cairn.collaboration_backend.cli]
// r[impl cairn.collaboration_backend.portable_shell]
// r[impl cairn.collaboration_backend.receipts]
// r[impl cairn.collaboration_backend.non_claims]
// r[impl cairn.collaboration_update.non_claims]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollaborationCommand {
    Import,
    Status,
    Checkout,
    Update,
    Conflicts,
    Resolve,
}

impl CollaborationCommand {
    fn parse(value: &str) -> Result<Self, Box<dyn Error>> {
        match value {
            "import" => Ok(Self::Import),
            "status" => Ok(Self::Status),
            "checkout" => Ok(Self::Checkout),
            "update" => Ok(Self::Update),
            "conflicts" => Ok(Self::Conflicts),
            "resolve" => Ok(Self::Resolve),
            _ => Err(format!("unknown collaboration subcommand: {value}").into()),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::Status => "status",
            Self::Checkout => "checkout",
            Self::Update => "update",
            Self::Conflicts => "conflicts",
            Self::Resolve => "resolve",
        }
    }

    fn allows_execute(self) -> bool {
        matches!(
            self,
            Self::Import | Self::Checkout | Self::Update | Self::Resolve
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PortableOptions {
    command: CollaborationCommand,
    change: String,
    root: PathBuf,
    store: PathBuf,
    workspace_id: Option<String>,
    base_revision: Option<ChangeBaseRevision>,
    actor: Option<String>,
    policy: PathBuf,
    execute: bool,
}

pub(crate) fn run_collaboration(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command_text) = args.first() else {
        return Err(
            "collaboration requires a subcommand: import, status, checkout, update, conflicts, or resolve".into(),
        );
    };
    let command = CollaborationCommand::parse(command_text)?;
    let (backend, backend_args) = extract_backend(&args[1..])?;
    match backend.as_str() {
        PORTABLE_COLLABORATION_BACKEND_V1 => {
            let receipt = match command {
                CollaborationCommand::Conflicts => {
                    resolution::run_portable_conflicts(&backend_args)?
                }
                CollaborationCommand::Resolve => resolution::run_portable_resolve(&backend_args)?,
                CollaborationCommand::Import
                | CollaborationCommand::Status
                | CollaborationCommand::Checkout
                | CollaborationCommand::Update => {
                    let options = parse_portable_options(command, &backend_args)?;
                    match command {
                        CollaborationCommand::Import => portable_import(&options)?,
                        CollaborationCommand::Status => portable_status(&options)?,
                        CollaborationCommand::Checkout => portable_checkout(&options)?,
                        CollaborationCommand::Update => portable_update(&options)?,
                        CollaborationCommand::Conflicts | CollaborationCommand::Resolve => {
                            return Err(
                                "specialized command reached ordinary portable parser".into()
                            );
                        }
                    }
                }
            };
            let root = collaboration_receipt_root(&receipt);
            print_json(with_portable_receipt_hash(&root, receipt))
        }
        COB_BACKEND_V1 => {
            let mut forwarded = vec![command.as_str().to_string()];
            forwarded.extend(backend_args);
            run_cob(forwarded)
        }
        unsupported => Err(format!("unsupported collaboration backend: {unsupported}").into()),
    }
}

fn collaboration_receipt_root(receipt: &Value) -> PathBuf {
    receipt
        .get("root")
        .and_then(Value::as_str)
        .map_or_else(|| PathBuf::from("."), PathBuf::from)
}

fn extract_backend(args: &[String]) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let mut backend = None;
    let mut forwarded = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--backend" {
            if backend.is_some() {
                return Err("duplicate option: --backend".into());
            }
            let value_index = index.saturating_add(1);
            let Some(value) = args.get(value_index) else {
                return Err("--backend requires a value".into());
            };
            if value.starts_with("--") {
                return Err("--backend requires a value".into());
            }
            backend = Some(value.clone());
            index = index.saturating_add(OPTION_TOKEN_WIDTH);
            continue;
        }
        forwarded.push(args[index].clone());
        index = index.saturating_add(1);
    }
    Ok((
        backend.ok_or("collaboration requires --backend BACKEND")?,
        forwarded,
    ))
}

fn parse_portable_options(
    command: CollaborationCommand,
    args: &[String],
) -> Result<PortableOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err(format!("collaboration {} requires a change name", command.as_str()).into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut store = None;
    let mut workspace_id = None;
    let mut base_revision = None;
    let mut actor = None;
    let mut policy = None;
    let mut execute = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                root = Some(read_option(args, index, "--root", root.is_some())?.into());
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--store" => {
                store = Some(read_option(args, index, "--store", store.is_some())?.into());
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--workspace" if command == CollaborationCommand::Import => {
                workspace_id = Some(read_option(
                    args,
                    index,
                    "--workspace",
                    workspace_id.is_some(),
                )?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--base-revision" if command == CollaborationCommand::Import => {
                let encoded = read_option(args, index, "--base-revision", base_revision.is_some())?;
                base_revision = Some(cairn_core::decode_base_revision(&encoded)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--actor"
                if matches!(
                    command,
                    CollaborationCommand::Import | CollaborationCommand::Update
                ) =>
            {
                actor = Some(read_option(args, index, "--actor", actor.is_some())?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--policy" => {
                policy = Some(read_option(args, index, "--policy", policy.is_some())?.into());
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--execute" if command.allows_execute() => {
                if execute {
                    return Err("duplicate option: --execute".into());
                }
                execute = true;
                index = index.saturating_add(1);
            }
            unknown => {
                return Err(format!(
                    "unknown {} argument for {PORTABLE_COLLABORATION_BACKEND_V1}: {unknown}",
                    command.as_str()
                )
                .into())
            }
        }
    }
    let store = store.ok_or_else(|| {
        format!(
            "collaboration {} with {PORTABLE_COLLABORATION_BACKEND_V1} requires --store PATH",
            command.as_str()
        )
    })?;
    if command == CollaborationCommand::Import {
        if workspace_id.is_none() {
            return Err("portable collaboration import requires --workspace ID".into());
        }
        if base_revision.is_none() {
            return Err("portable collaboration import requires --base-revision KIND:VALUE".into());
        }
        if actor.is_none() {
            return Err("portable collaboration import requires --actor ID".into());
        }
    }
    if command == CollaborationCommand::Update && actor.is_none() {
        return Err("portable collaboration update requires --actor ID".into());
    }
    Ok(PortableOptions {
        command,
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        store,
        workspace_id,
        base_revision,
        actor,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        execute,
    })
}

fn read_option(
    args: &[String],
    index: usize,
    name: &str,
    already_present: bool,
) -> Result<String, Box<dyn Error>> {
    if already_present {
        return Err(format!("duplicate option: {name}").into());
    }
    let value_index = index.saturating_add(1);
    let Some(value) = args.get(value_index) else {
        return Err(format!("{name} requires a value").into());
    };
    if value.starts_with("--") {
        return Err(format!("{name} requires a value").into());
    }
    Ok(value.clone())
}

fn portable_import(options: &PortableOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, CollaborationCommand::Import);
    let policy = load_policy_file(options.policy.clone())?;
    let policy_hash = canonical_json_hash(&policy.raw);
    let preview = build_import_envelope(options, &policy_hash)?;
    let mut blockers = store_blockers(&options.store);
    let mut mutated = false;

    if options.execute && blockers.is_empty() {
        match PortableStoreLock::acquire(&options.store, true) {
            Ok(_guard) => {
                let current_policy = load_policy_file(options.policy.clone())?;
                let current_policy_hash = canonical_json_hash(&current_policy.raw);
                let current = build_import_envelope(options, &current_policy_hash)?;
                let confirmed = build_import_envelope(options, &current_policy_hash)?;
                if current != preview || confirmed != current {
                    blockers.push(BLOCKER_INPUT_DRIFT.to_string());
                } else if store_exists(&options.store)? {
                    blockers.push(BLOCKER_STORE_EXISTS.to_string());
                } else if let Err(error) = commit_store(&options.store, &current) {
                    blockers.push(error.to_string());
                } else {
                    mutated = true;
                }
            }
            Err(error) => blockers.push(error.to_string()),
        }
    }
    blockers.sort();
    blockers.dedup();
    import_receipt(options, &policy.name, &preview, &blockers, mutated)
}

fn portable_status(options: &PortableOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, CollaborationCommand::Status);
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let envelope = match load_store(&options.store) {
        Ok(envelope) => envelope,
        Err(error) => {
            return Ok(failure_receipt(options, "status", error.to_string()));
        }
    };
    if envelope.state.change_id != options.change {
        return Ok(failure_receipt(
            options,
            "status",
            BLOCKER_CHANGE_MISMATCH.to_string(),
        ));
    }
    let local = read_local_change_snapshot(&options.root, &options.change)?;
    let plan = plan_change_collaboration_checkout(&envelope.state, local)?;
    Ok(status_receipt(
        options,
        &policy.name,
        &observed_policy_hash,
        &envelope,
        &plan,
    ))
}

fn portable_checkout(options: &PortableOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, CollaborationCommand::Checkout);
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let envelope = match load_store(&options.store) {
        Ok(envelope) => envelope,
        Err(error) => {
            return Ok(failure_receipt(options, "checkout", error.to_string()));
        }
    };
    if envelope.state.change_id != options.change {
        return Ok(failure_receipt(
            options,
            "checkout",
            BLOCKER_CHANGE_MISMATCH.to_string(),
        ));
    }
    let local = read_local_change_snapshot(&options.root, &options.change)?;
    let preview = plan_change_collaboration_checkout(&envelope.state, local)?;
    let mut outcome = MaterializationOutcome {
        mutated: false,
        final_destination: None,
        blockers: preview.blockers.clone(),
    };

    if options.execute && outcome.blockers.is_empty() && !preview.already_current() {
        match PortableStoreLock::acquire(&options.store, false) {
            Ok(_store_guard) => {
                match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
                    Ok(repository_guard) => {
                        let current_envelope = load_store(&options.store)?;
                        let current_local =
                            read_local_change_snapshot(&options.root, &options.change)?;
                        let current_plan = plan_change_collaboration_checkout(
                            &current_envelope.state,
                            current_local,
                        )?;
                        if current_envelope != envelope || current_plan != preview {
                            outcome.blockers.push(BLOCKER_INPUT_DRIFT.to_string());
                        } else {
                            outcome = materialize_collaboration_checkout(
                                &options.root,
                                &current_envelope.state,
                                &current_plan,
                            )?;
                        }
                        if let Err(error) = repository_guard.release() {
                            outcome.blockers.push(error.to_string());
                        }
                    }
                    Err(error) => outcome.blockers.push(error.to_string()),
                }
            }
            Err(error) => outcome.blockers.push(error.to_string()),
        }
    }
    outcome.blockers.sort();
    outcome.blockers.dedup();
    Ok(checkout_receipt(
        options,
        &policy.name,
        &observed_policy_hash,
        &envelope,
        &preview,
        &outcome,
    ))
}

// r[impl cairn.collaboration_update.portable_execution]
// r[impl cairn.collaboration_update.receipts]
fn portable_update(options: &PortableOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, CollaborationCommand::Update);
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let envelope = match load_store(&options.store) {
        Ok(envelope) => envelope,
        Err(error) => {
            return Ok(failure_receipt(options, "update", error.to_string()));
        }
    };
    if envelope.state.change_id != options.change {
        return Ok(failure_receipt(
            options,
            "update",
            BLOCKER_CHANGE_MISMATCH.to_string(),
        ));
    }
    let (preview, candidate) =
        match build_update_candidate(options, &observed_policy_hash, &envelope) {
            Ok(value) => value,
            Err(error) => return Ok(failure_receipt(options, "update", error.to_string())),
        };
    let mut blockers = Vec::new();
    let mut mutated = false;

    if options.execute && !preview.is_noop() {
        match PortableStoreLock::acquire(&options.store, false) {
            Ok(_store_guard) => {
                match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
                    Ok(repository_guard) => {
                        let execution = (|| {
                            let current_policy = load_policy_file(options.policy.clone())?;
                            let current_policy_hash = canonical_json_hash(&current_policy.raw);
                            let current_envelope = load_store(&options.store)?;
                            let (current_plan, current_candidate) = build_update_candidate(
                                options,
                                &current_policy_hash,
                                &current_envelope,
                            )?;
                            if current_envelope != envelope
                                || current_plan != preview
                                || current_candidate != candidate
                            {
                                return Err(BLOCKER_INPUT_DRIFT.into());
                            }
                            replace_store(&options.store, &current_envelope, &current_candidate)?;
                            Ok::<(), Box<dyn Error>>(())
                        })();
                        match execution {
                            Ok(()) => mutated = true,
                            Err(error) => blockers.push(error.to_string()),
                        }
                        if let Err(error) = repository_guard.release() {
                            blockers.push(error.to_string());
                        }
                    }
                    Err(error) => blockers.push(error.to_string()),
                }
            }
            Err(error) => blockers.push(error.to_string()),
        }
    }
    blockers.sort();
    blockers.dedup();
    update_receipt(PortableUpdateReceiptInput {
        options,
        policy_name: &policy.name,
        observed_policy_hash: &observed_policy_hash,
        input: &envelope,
        candidate: &candidate,
        plan: &preview,
        blockers: &blockers,
        mutated,
    })
}

fn build_update_candidate(
    options: &PortableOptions,
    policy_hash: &str,
    envelope: &PortableChangeCollaboration,
) -> Result<(ChangeCollaborationUpdatePlan, PortableChangeCollaboration), Box<dyn Error>> {
    let artifacts = read_active_change_artifacts(&options.root, &options.change)?;
    let plan = plan_change_collaboration_update(
        &envelope.state,
        ChangeCollaborationUpdateSnapshot {
            change_id: options.change.clone(),
            policy_hash: policy_hash.to_string(),
            actor: options
                .actor
                .clone()
                .ok_or("portable update actor is missing")?,
            is_delegate: false,
            proposal: artifacts.proposal,
            design: artifacts.design,
            tasks: artifacts.tasks,
            metadata: artifacts.metadata,
            delta_specs: artifacts.delta_specs,
        },
    )?;
    let updated = apply_planned_change_collaboration_update(&envelope.state, &plan)?;
    let candidate = build_portable_change_collaboration(updated)?;
    Ok((plan, candidate))
}

fn build_import_envelope(
    options: &PortableOptions,
    policy_hash: &str,
) -> Result<PortableChangeCollaboration, Box<dyn Error>> {
    let artifacts = read_active_change_artifacts(&options.root, &options.change)?;
    let init = plan_change_collaboration_import(ChangeCollaborationImportSnapshot {
        change_id: options.change.clone(),
        workspace_id: options
            .workspace_id
            .clone()
            .ok_or("portable import workspace is missing")?,
        base_revision: options
            .base_revision
            .clone()
            .ok_or("portable import base revision is missing")?,
        policy_hash: policy_hash.to_string(),
        proposal: artifacts.proposal,
        design: artifacts.design,
        tasks: artifacts.tasks,
        metadata: artifacts.metadata,
        delta_specs: artifacts.delta_specs,
    })?;
    let state = initialize_change_collaboration_import(
        init,
        options
            .actor
            .clone()
            .ok_or("portable import actor is missing")?,
    )?;
    Ok(build_portable_change_collaboration(state)?)
}

fn store_blockers(store: &std::path::Path) -> Vec<String> {
    match store_exists(store) {
        Ok(true) => vec![BLOCKER_STORE_EXISTS.to_string()],
        Ok(false) => Vec::new(),
        Err(error) => vec![error.to_string()],
    }
}

fn import_receipt(
    options: &PortableOptions,
    policy_name: &str,
    envelope: &PortableChangeCollaboration,
    blockers: &[String],
    mutated: bool,
) -> Result<Value, Box<dyn Error>> {
    let artifacts = artifact_receipts(&envelope.state)?;
    let input_hash = envelope.state_hash.clone();
    let plan_hash = canonical_json_hash(&json!({
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "change": options.change,
        "workspace_id": envelope.state.workspace_id,
        "base_revision": envelope.state.base_revision,
        "state_hash": envelope.state_hash,
        "content_root": envelope.content_root,
        "policy_hash": envelope.state.policy_hash,
        "input_hash": input_hash,
    }));
    let blocked = !blockers.is_empty();
    Ok(json!({
        "command": "collaboration import",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": envelope.state.workspace_id,
        "base_revision": envelope.state.base_revision,
        "actor": actor_observation(&envelope.state),
        "policy": policy_name,
        "policy_hash": envelope.state.policy_hash,
        "artifacts": artifacts,
        "artifact_count": artifacts.len(),
        "input_hash": input_hash,
        "plan_hash": plan_hash,
        "content_root": envelope.content_root,
        "state_hash": envelope.state_hash,
        "envelope_hash": envelope.envelope_hash,
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": mutated,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    }))
}

fn status_receipt(
    options: &PortableOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    envelope: &PortableChangeCollaboration,
    plan: &ChangeCollaborationCheckoutPlan,
) -> Value {
    let blocked = !plan.blockers.is_empty();
    json!({
        "command": "collaboration status",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": plan.workspace_id,
        "base_revision": plan.base_revision,
        "actor": actor_observation(&envelope.state),
        "policy": policy_name,
        "policy_hash": plan.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": plan.policy_hash == observed_policy_hash,
        "state_hash": plan.state_hash,
        "content_root": plan.content_root,
        "envelope_hash": envelope.envelope_hash,
        "lifecycle": lifecycle_json(&plan.lifecycle),
        "evidence_count": plan.evidence_count,
        "current_evidence_count": plan.current_evidence_count,
        "content_conflicts": plan.content_conflicts,
        "files": checkout_file_receipts(plan),
        "local_destination": plan.destination.as_str(),
        "blocked": blocked,
        "blockers": plan.blockers,
        "mutated": false,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    })
}

fn checkout_receipt(
    options: &PortableOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    envelope: &PortableChangeCollaboration,
    plan: &ChangeCollaborationCheckoutPlan,
    outcome: &MaterializationOutcome,
) -> Value {
    let blocked = !outcome.blockers.is_empty();
    json!({
        "command": "collaboration checkout",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": plan.workspace_id,
        "base_revision": plan.base_revision,
        "actor": actor_observation(&envelope.state),
        "local_filesystem_capability_selected": true,
        "cryptographic_authority_proved": false,
        "policy": policy_name,
        "policy_hash": plan.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": plan.policy_hash == observed_policy_hash,
        "state_hash": plan.state_hash,
        "content_root": plan.content_root,
        "envelope_hash": envelope.envelope_hash,
        "lifecycle": lifecycle_json(&plan.lifecycle),
        "evidence_count": plan.evidence_count,
        "current_evidence_count": plan.current_evidence_count,
        "content_conflicts": plan.content_conflicts,
        "files": checkout_file_receipts(plan),
        "local_destination": plan.destination.as_str(),
        "post_checkout_destination": outcome.final_destination.map(ChangeCheckoutDestination::as_str),
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": outcome.blockers,
        "mutated": outcome.mutated,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    })
}

struct PortableUpdateReceiptInput<'a> {
    options: &'a PortableOptions,
    policy_name: &'a str,
    observed_policy_hash: &'a str,
    input: &'a PortableChangeCollaboration,
    candidate: &'a PortableChangeCollaboration,
    plan: &'a ChangeCollaborationUpdatePlan,
    blockers: &'a [String],
    mutated: bool,
}

fn update_receipt(receipt: PortableUpdateReceiptInput<'_>) -> Result<Value, Box<dyn Error>> {
    let PortableUpdateReceiptInput {
        options,
        policy_name,
        observed_policy_hash,
        input,
        candidate,
        plan,
        blockers,
        mutated,
    } = receipt;
    let plan_hash = change_collaboration_update_plan_hash(plan)?;
    let updates = plan
        .actions
        .iter()
        .filter_map(|action| match action {
            cairn_core::ChangeCollaborationAction::SetArtifact {
                artifact,
                content,
                basis,
            } => Some(json!({
                "artifact": artifact.as_str(),
                "basis": basis,
                "content_blake3": content
                    .as_ref()
                    .map(|value| cairn_core::blake3_hex(value.as_bytes())),
                "content_bytes": content.as_ref().map(String::len),
                "removed": content.is_none(),
            })),
            cairn_core::ChangeCollaborationAction::Init { .. }
            | cairn_core::ChangeCollaborationAction::AttachEvidence { .. }
            | cairn_core::ChangeCollaborationAction::SetLifecycle { .. } => None,
        })
        .collect::<Vec<_>>();
    let blocked = !blockers.is_empty();
    Ok(json!({
        "command": "collaboration update",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": input.state.workspace_id,
        "base_revision": input.state.base_revision,
        "actor": {
            "id": plan.actor,
            "source": "caller_declared",
            "cryptographically_verified": false,
            "currently_authorized": plan.actor == input.state.creator,
            "authority_class": "portable_creator_declaration",
        },
        "policy": policy_name,
        "policy_hash": input.state.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": input.state.policy_hash == observed_policy_hash,
        "input_hash": plan.input_state_hash,
        "plan_hash": plan_hash,
        "operation_id": plan.operation_id,
        "input_state_hash": plan.input_state_hash,
        "input_content_root": plan.input_content_root,
        "input_envelope_hash": input.envelope_hash,
        "expected_content_root": plan.expected_content_root,
        "expected_state_hash": candidate.state_hash,
        "expected_envelope_hash": candidate.envelope_hash,
        "changed_artifacts": plan.changed_artifacts,
        "updates": updates,
        "noop": plan.is_noop(),
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": mutated,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    }))
}

fn failure_receipt(options: &PortableOptions, command: &str, blocker: String) -> Value {
    json!({
        "command": format!("collaboration {command}"),
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "dry_run": !options.execute,
        "blocked": true,
        "blockers": [blocker],
        "mutated": false,
        "valid": false,
        "verdict": "FAIL",
        "non_claims": non_claims(),
    })
}

fn actor_observation(state: &ChangeCollaborationState) -> Value {
    json!({
        "id": state.creator,
        "source": "caller_declared",
        "cryptographically_verified": false,
        "currently_authorized": null,
    })
}

fn artifact_receipts(state: &ChangeCollaborationState) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(materialize_change_collaboration(state)?
        .into_iter()
        .map(|file| {
            json!({
                "path": file.path,
                "blake3": cairn_core::blake3_hex(file.content.as_bytes()),
                "bytes": file.content.len(),
            })
        })
        .collect())
}

fn checkout_file_receipts(plan: &ChangeCollaborationCheckoutPlan) -> Vec<Value> {
    plan.files
        .iter()
        .map(|file| {
            json!({
                "path": file.path,
                "blake3": file.blake3,
                "bytes": file.byte_length,
            })
        })
        .collect()
}

fn lifecycle_json(lifecycle: &ChangeLifecycleView) -> Value {
    match lifecycle {
        ChangeLifecycleView::Current(record) => json!({
            "condition": "current",
            "status": format!("{:?}", record.status).to_lowercase(),
            "subject_root": record.subject_root,
            "evidence": record.evidence,
        }),
        ChangeLifecycleView::Stale {
            record,
            current_content_root,
        } => json!({
            "condition": "stale",
            "status": format!("{:?}", record.status).to_lowercase(),
            "subject_root": record.subject_root,
            "current_content_root": current_content_root,
            "evidence": record.evidence,
        }),
        ChangeLifecycleView::Conflicted { revisions } => json!({
            "condition": "conflicted",
            "revisions": revisions,
        }),
        ChangeLifecycleView::ContentConflicted { artifacts } => json!({
            "condition": "content_conflicted",
            "artifacts": artifacts,
        }),
    }
}

fn non_claims() -> [&'static str; 5] {
    [
        NON_CLAIM_AUTHORITY,
        NON_CLAIM_LOCAL,
        NON_CLAIM_LIFECYCLE,
        NON_CLAIM_TRANSPORT,
        NON_CLAIM_CONFLICT,
    ]
}

#[cfg(test)]
mod tests {
    use std::{fs, io, path::Path};

    use super::*;

    const TEST_DIRECTORY_PREFIX: &str = "cairn-collaboration-portable";
    const CHANGE_ID: &str = "portable-demo";
    const PROPOSAL: &str = "# Proposal\n\nPortable.\n";
    const UPDATED_PROPOSAL: &str = "# Updated Proposal\n\nPortable.\n";
    const TEST_POLICY_RELATIVE: &str = "../../cairn-policy/generated/cairn-policy.json";

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    fn remove_directory(path: &Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn fixture(name: &str) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
        let root = test_directory(name);
        remove_directory(&root)?;
        let change = root.join(".cairn/changes").join(CHANGE_ID);
        fs::create_dir_all(change.join("specs"))?;
        fs::write(change.join("proposal.md"), PROPOSAL)?;
        Ok((root.clone(), root.join("portable-store.json")))
    }

    fn options(
        command: CollaborationCommand,
        root: &Path,
        store: &Path,
        execute: bool,
    ) -> PortableOptions {
        PortableOptions {
            command,
            change: CHANGE_ID.to_string(),
            root: root.to_path_buf(),
            store: store.to_path_buf(),
            workspace_id: (command == CollaborationCommand::Import)
                .then(|| "workspace:test".to_string()),
            base_revision: (command == CollaborationCommand::Import)
                .then(|| ChangeBaseRevision::new("opaque", "revision-one").unwrap()),
            actor: matches!(
                command,
                CollaborationCommand::Import | CollaborationCommand::Update
            )
            .then(|| "actor:test".to_string()),
            policy: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TEST_POLICY_RELATIVE),
            execute,
        }
    }

    // r[verify cairn.collaboration_backend.cli.portable]
    // r[verify cairn.collaboration_backend.cli.arguments]
    #[test]
    fn backend_dispatch_and_portable_arguments_are_explicit() {
        let (backend, args) = extract_backend(&[
            CHANGE_ID.to_string(),
            "--backend".to_string(),
            PORTABLE_COLLABORATION_BACKEND_V1.to_string(),
            "--store".to_string(),
            "store.json".to_string(),
            "--workspace".to_string(),
            "workspace:test".to_string(),
            "--base-revision".to_string(),
            "opaque:one".to_string(),
            "--actor".to_string(),
            "actor:test".to_string(),
        ])
        .unwrap();
        assert_eq!(backend, PORTABLE_COLLABORATION_BACKEND_V1);
        let parsed = parse_portable_options(CollaborationCommand::Import, &args).unwrap();
        assert_eq!(parsed.change, CHANGE_ID);
        assert!(args.iter().all(|arg| arg != "--rad-home" && arg != "--rid"));
        assert!(parse_portable_options(
            CollaborationCommand::Status,
            &[CHANGE_ID.to_string(), "--rad-home".to_string()]
        )
        .is_err());
        assert!(parse_portable_options(
            CollaborationCommand::Update,
            &[
                CHANGE_ID.to_string(),
                "--store".to_string(),
                "store.json".to_string(),
            ]
        )
        .is_err());
        assert!(extract_backend(&[CHANGE_ID.to_string()]).is_err());
        assert_eq!(
            CollaborationCommand::parse("conflicts").unwrap(),
            CollaborationCommand::Conflicts
        );
        assert_eq!(
            CollaborationCommand::parse("resolve").unwrap(),
            CollaborationCommand::Resolve
        );
        assert!(CollaborationCommand::parse("sync").is_err());
        assert!(run_collaboration(vec![
            "sync".to_string(),
            CHANGE_ID.to_string(),
            "--backend".to_string(),
            PORTABLE_COLLABORATION_BACKEND_V1.to_string(),
        ])
        .is_err());
    }

    // r[verify cairn.collaboration_backend.portable_shell.positive]
    // r[verify cairn.collaboration_backend.receipts.authority]
    // r[verify cairn.collaboration_update.portable_execution.success]
    // r[verify cairn.collaboration_update.receipts.modes]
    #[test]
    fn portable_import_status_checkout_round_trip_needs_no_radicle() -> Result<(), Box<dyn Error>> {
        let (root, store) = fixture("roundtrip")?;
        let preview =
            portable_import(&options(CollaborationCommand::Import, &root, &store, false))?;
        assert_eq!(preview["dry_run"], true);
        assert_eq!(preview["mutated"], false);
        assert!(!store.exists());

        let imported =
            portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(imported["valid"], true);
        assert_eq!(imported["mutated"], true);
        assert_eq!(imported["actor"]["cryptographically_verified"], false);
        assert!(store.is_file());

        let status = portable_status(&options(CollaborationCommand::Status, &root, &store, false))?;
        assert_eq!(status["local_destination"], "current");
        assert_eq!(status["backend"], PORTABLE_COLLABORATION_BACKEND_V1);

        fs::write(
            root.join(".cairn/changes")
                .join(CHANGE_ID)
                .join("proposal.md"),
            UPDATED_PROPOSAL,
        )?;
        let store_before_preview = fs::read(&store)?;
        let update_preview =
            portable_update(&options(CollaborationCommand::Update, &root, &store, false))?;
        assert_eq!(update_preview["dry_run"], true);
        assert_eq!(update_preview["mutated"], false);
        assert_eq!(update_preview["changed_artifacts"], json!(["proposal"]));
        assert_eq!(fs::read(&store)?, store_before_preview);

        let update = portable_update(&options(CollaborationCommand::Update, &root, &store, true))?;
        assert_eq!(update["valid"], true);
        assert_eq!(update["mutated"], true);
        assert_ne!(fs::read(&store)?, store_before_preview);
        let noop = portable_update(&options(CollaborationCommand::Update, &root, &store, true))?;
        assert_eq!(noop["noop"], true);
        assert_eq!(noop["mutated"], false);

        fs::remove_dir_all(root.join(".cairn/changes").join(CHANGE_ID))?;
        let checkout_preview = portable_checkout(&options(
            CollaborationCommand::Checkout,
            &root,
            &store,
            false,
        ))?;
        assert_eq!(checkout_preview["local_destination"], "absent");
        assert_eq!(checkout_preview["mutated"], false);

        let checkout = portable_checkout(&options(
            CollaborationCommand::Checkout,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(checkout["valid"], true);
        assert_eq!(checkout["mutated"], true);
        assert_eq!(
            fs::read_to_string(
                root.join(".cairn/changes")
                    .join(CHANGE_ID)
                    .join("proposal.md")
            )?,
            UPDATED_PROPOSAL
        );
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_backend.portable_shell.negative]
    // r[verify cairn.collaboration_backend.non_claims.provider]
    #[test]
    fn portable_store_tamper_existing_import_and_divergence_fail_closed(
    ) -> Result<(), Box<dyn Error>> {
        let (root, store) = fixture("negative")?;
        let imported =
            portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(imported["valid"], true);

        let repeated =
            portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(repeated["blocked"], true);
        assert_eq!(repeated["mutated"], false);

        fs::write(
            root.join(".cairn/changes")
                .join(CHANGE_ID)
                .join("proposal.md"),
            "# Divergent\n",
        )?;
        let divergent = portable_checkout(&options(
            CollaborationCommand::Checkout,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(divergent["blocked"], true);
        assert_eq!(divergent["mutated"], false);

        let mut envelope: Value = serde_json::from_slice(&fs::read(&store)?)?;
        envelope["content_root"] = Value::String("b".repeat(cairn_core::BLAKE3_HEX_LENGTH));
        fs::write(&store, serde_json::to_vec_pretty(&envelope)?)?;
        let tampered =
            portable_status(&options(CollaborationCommand::Status, &root, &store, false))?;
        assert_eq!(tampered["valid"], false);
        assert_eq!(tampered["mutated"], false);
        assert!(tampered["non_claims"]
            .as_array()
            .is_some_and(|claims| claims.iter().any(|claim| claim == NON_CLAIM_AUTHORITY)));
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_update.portable_execution.blocked]
    // r[verify cairn.collaboration_update.non_claims.local]
    #[test]
    fn portable_update_rejects_wrong_actor_stale_lock_and_prepared_state(
    ) -> Result<(), Box<dyn Error>> {
        let (root, store) = fixture("update-negative")?;
        let imported =
            portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(imported["valid"], true);
        fs::write(
            root.join(".cairn/changes")
                .join(CHANGE_ID)
                .join("proposal.md"),
            UPDATED_PROPOSAL,
        )?;
        let original_store = fs::read(&store)?;

        let mut unauthorized = options(CollaborationCommand::Update, &root, &store, true);
        unauthorized.actor = Some("actor:outsider".to_string());
        let denied = portable_update(&unauthorized)?;
        assert_eq!(denied["blocked"], true);
        assert_eq!(denied["mutated"], false);
        assert!(denied["non_claims"]
            .as_array()
            .is_some_and(|claims| claims.iter().any(|claim| claim == NON_CLAIM_CONFLICT)));
        assert_eq!(fs::read(&store)?, original_store);

        let lock = root.join(".portable-store.json.lock");
        fs::write(&lock, b"stale\n")?;
        let locked = portable_update(&options(CollaborationCommand::Update, &root, &store, true))?;
        assert_eq!(locked["blocked"], true);
        assert_eq!(locked["mutated"], false);
        assert_eq!(fs::read(&store)?, original_store);
        fs::remove_file(&lock)?;

        let preview =
            portable_update(&options(CollaborationCommand::Update, &root, &store, false))?;
        let expected_envelope_hash = preview["expected_envelope_hash"]
            .as_str()
            .ok_or("update preview is missing expected_envelope_hash")?;
        let prepared = root.join(format!(
            ".portable-store.json.{expected_envelope_hash}.prepared"
        ));
        fs::write(&prepared, b"stale\n")?;
        let blocked = portable_update(&options(CollaborationCommand::Update, &root, &store, true))?;
        assert_eq!(blocked["blocked"], true);
        assert_eq!(blocked["mutated"], false);
        assert_eq!(fs::read(&store)?, original_store);
        assert!(prepared.exists());
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_backend.portable_shell.negative]
    #[test]
    fn portable_import_rejects_stale_lock_without_creating_store() -> Result<(), Box<dyn Error>> {
        let (root, store) = fixture("stale-lock")?;
        let lock = root.join(".portable-store.json.lock");
        fs::write(&lock, b"stale\n")?;
        let receipt = portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(receipt["blocked"], true);
        assert_eq!(receipt["mutated"], false);
        assert!(!store.exists());
        assert!(lock.exists());
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_backend.portable_shell.negative]
    #[test]
    fn repository_checkout_lock_serializes_backends() -> Result<(), Box<dyn Error>> {
        let (root, store) = fixture("repository-lock")?;
        let imported =
            portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(imported["valid"], true);
        fs::remove_dir_all(root.join(".cairn/changes").join(CHANGE_ID))?;

        let guard = RepositoryCheckoutLock::acquire(&root, CHANGE_ID)?;
        let blocked = portable_checkout(&options(
            CollaborationCommand::Checkout,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(blocked["blocked"], true);
        assert_eq!(blocked["mutated"], false);
        assert!(!root.join(".cairn/changes").join(CHANGE_ID).exists());
        guard.release()?;
        remove_directory(&root)?;
        Ok(())
    }

    #[cfg(unix)]
    // r[verify cairn.collaboration_backend.portable_shell.negative]
    #[test]
    fn portable_import_rejects_symlink_store() -> Result<(), Box<dyn Error>> {
        use std::os::unix::fs::symlink;

        let (root, store) = fixture("symlink-store")?;
        let outside = root.join("outside.json");
        fs::write(&outside, b"unchanged\n")?;
        symlink(&outside, &store)?;
        let receipt = portable_import(&options(CollaborationCommand::Import, &root, &store, true))?;
        assert_eq!(receipt["blocked"], true);
        assert_eq!(receipt["mutated"], false);
        assert_eq!(fs::read(&outside)?, b"unchanged\n");
        remove_directory(&root)?;
        Ok(())
    }
}
