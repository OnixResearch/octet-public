use std::{error::Error, path::PathBuf};

use cairn_core::{
    change_cob_resolution_plan_hash, inspect_change_cob_conflict_value,
    inspect_change_cob_conflicts, plan_change_cob_resolution, validate_change_name,
    ChangeArtifactHeadSelection, ChangeArtifactId, ChangeCobAction, ChangeCobResolutionPlan,
    ChangeCollaborationConflictInspection, ChangeCollaborationResolutionSnapshot,
    ChangeLifecycleStatus, MAX_OPERATION_ID_BYTES,
};
use onix_cairn_radicle_cob::{
    embedded_authority, inspect_change, signer_actor, update_change, CobChangeInspection,
    CobMutationLock, CobMutationOutcome,
};
use serde_json::{json, Value};

use crate::{
    collaboration::materialize::RepositoryCheckoutLock,
    support::{
        default_policy_path, load_policy_file, path_string, print_json, with_portable_receipt_hash,
    },
};

use super::{
    format_library_versions, read_unique_option_value, COB_BACKEND_V1, OPTION_TOKEN_WIDTH,
};

const BLOCKER_INPUT_DRIFT: &str =
    "policy, Git repository head, object, signer, embedded authority, typed state, conflict heads, or resolution choices changed after preview";
const NON_CLAIM_AUTHORITY: &str =
    "embedded creator and delegate keys are immutable COB authority, not Radicle identity-document or future authorization proof";
const NON_CLAIM_LOCAL: &str =
    "signed conflict resolution proves bounded local Git COB observations only";
const NON_CLAIM_MERGE: &str =
    "selecting a conflict head does not prove semantic merge correctness or that losing heads are invalid";
const NON_CLAIM_LIFECYCLE: &str =
    "an explicit lifecycle reset to open does not prove review, approval, evidence truth, source correctness, archive execution, or release eligibility";
const NON_CLAIM_TRANSPORT: &str =
    "local signed COB resolution does not prove node participation, publication, synchronization, peer convergence, availability, or remote trust";
const NON_CLAIM_COUNT: usize = 5;

// r[impl cairn.radicle_cob_only.cli]
// r[impl cairn.radicle_cob_only.docs]
// r[impl cairn.collaboration_resolution.radicle_execution]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CobResolutionCommand {
    Conflicts,
    Resolve,
}

impl CobResolutionCommand {
    fn as_str(self) -> &'static str {
        match self {
            Self::Conflicts => "conflicts",
            Self::Resolve => "resolve",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CobResolutionOptions {
    command: CobResolutionCommand,
    change: String,
    root: PathBuf,
    git_repository: PathBuf,
    keystore: Option<PathBuf>,
    policy: PathBuf,
    show_head: Option<ChangeArtifactHeadSelection>,
    selections: Vec<ChangeArtifactHeadSelection>,
    lifecycle_status: Option<ChangeLifecycleStatus>,
    execute: bool,
}

pub(super) fn run_conflicts(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_options(CobResolutionCommand::Conflicts, &args)?;
    let inspection = inspect_change(&options.git_repository, &options.change)?;
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = cairn_core::canonical_json_hash(&policy.raw);
    let conflicts = inspect_change_cob_conflicts(&inspection.state)?;
    let shown_head = match &options.show_head {
        Some(selection) => match inspect_change_cob_conflict_value(
            &inspection.state,
            &selection.artifact,
            &selection.revision_id,
        ) {
            Ok(value) => Some(value),
            Err(error) => {
                return print_json(with_portable_receipt_hash(
                    &options.root,
                    failure_receipt(&options, error.to_string()),
                ));
            }
        },
        None => None,
    };
    let receipt = conflict_receipt(
        &options,
        &policy.name,
        &observed_policy_hash,
        &inspection,
        &conflicts,
        shown_head.map(serde_json::to_value).transpose()?,
    );
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

pub(super) fn run_resolve(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_options(CobResolutionCommand::Resolve, &args)?;
    let inspection = inspect_change(&options.git_repository, &options.change)?;
    let actor = resolution_actor(&options)?;
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = cairn_core::canonical_json_hash(&policy.raw);
    let (conflicts, preview) =
        match build_resolution_plan(&options, &inspection, &actor, &observed_policy_hash) {
            Ok(value) => value,
            Err(error) => {
                return print_json(with_portable_receipt_hash(
                    &options.root,
                    failure_receipt(&options, error.to_string()),
                ));
            }
        };
    let mut blockers = Vec::new();
    let mut outcome = None;

    if options.execute {
        match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
            Ok(repository_guard) => {
                match CobMutationLock::acquire(&options.git_repository, &options.change) {
                    Ok(cob_guard) => {
                        let execution = recheck_resolution_inputs(&options, &inspection, &preview)
                            .and_then(|(current, current_plan)| {
                                update_change(
                                    &options.git_repository,
                                    required_keystore(&options)?,
                                    &current.object_id,
                                    &options.change,
                                    &current_plan.actor,
                                    &current_plan.actions,
                                    &current_plan.expected_content_root,
                                )
                                .map_err(Into::into)
                            });
                        let cob_release = cob_guard.release();
                        match execution {
                            Ok(resolved) => outcome = Some(resolved),
                            Err(error) => blockers.push(error.to_string()),
                        }
                        if let Err(error) = cob_release {
                            blockers.push(error.to_string());
                        }
                    }
                    Err(error) => blockers.push(error.to_string()),
                }
                if let Err(error) = repository_guard.release() {
                    blockers.push(error.to_string());
                }
            }
            Err(error) => blockers.push(error.to_string()),
        }
    }
    blockers.sort();
    blockers.dedup();
    let receipt = resolution_receipt(
        &options,
        &policy.name,
        &observed_policy_hash,
        &inspection,
        &conflicts,
        &preview,
        &blockers,
        outcome.as_ref(),
    )?;
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

fn build_resolution_plan(
    options: &CobResolutionOptions,
    inspection: &CobChangeInspection,
    actor: &str,
    policy_hash: &str,
) -> Result<
    (
        ChangeCollaborationConflictInspection,
        ChangeCobResolutionPlan,
    ),
    Box<dyn Error>,
> {
    let conflicts = inspect_change_cob_conflicts(&inspection.state)?;
    let (_, is_delegate) = embedded_authority(&inspection.state, actor);
    let plan = plan_change_cob_resolution(
        &inspection.state,
        ChangeCollaborationResolutionSnapshot {
            change_id: options.change.clone(),
            policy_hash: policy_hash.to_string(),
            actor: actor.to_string(),
            is_delegate,
            artifact_selections: options.selections.clone(),
            lifecycle_status: options.lifecycle_status,
        },
    )?;
    Ok((conflicts, plan))
}

fn recheck_resolution_inputs(
    options: &CobResolutionOptions,
    expected_inspection: &CobChangeInspection,
    expected_plan: &ChangeCobResolutionPlan,
) -> Result<(CobChangeInspection, ChangeCobResolutionPlan), Box<dyn Error>> {
    let current_policy = load_policy_file(options.policy.clone())?;
    let current_policy_hash = cairn_core::canonical_json_hash(&current_policy.raw);
    let current = inspect_change(&options.git_repository, &options.change)?;
    let actor = resolution_actor(options)?;
    let (_, current_plan) = build_resolution_plan(options, &current, &actor, &current_policy_hash)?;
    if &current != expected_inspection || &current_plan != expected_plan {
        return Err(BLOCKER_INPUT_DRIFT.into());
    }
    Ok((current, current_plan))
}

fn conflict_receipt(
    options: &CobResolutionOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    inspection: &CobChangeInspection,
    conflicts: &ChangeCollaborationConflictInspection,
    shown_head: Option<Value>,
) -> Value {
    json!({
        "command": "collaboration conflicts",
        "compatibility_command": "cob conflicts",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": inspection.state.repository_id,
        "base_revision": {
            "kind": "git_oid",
            "value": inspection.repository_head_oid,
        },
        "object_id": inspection.object_id,
        "authority": {
            "creator": inspection.state.creator,
            "delegates": inspection.state.delegates,
            "source": "signed_root_embedded_authority",
        },
        "policy": policy_name,
        "policy_hash": inspection.state.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": inspection.state.policy_hash == observed_policy_hash,
        "state_hash": conflicts.state_hash,
        "conflicted": conflicts.has_conflicts(),
        "artifact_conflicts": conflicts.artifact_conflicts,
        "lifecycle_heads": conflicts.lifecycle_heads,
        "shown_head": shown_head,
        "blocked": false,
        "blockers": [],
        "mutated": false,
        "valid": true,
        "verdict": "PASS",
        "non_claims": non_claims(),
    })
}

#[allow(clippy::too_many_arguments)]
fn resolution_receipt(
    options: &CobResolutionOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    inspection: &CobChangeInspection,
    conflicts: &ChangeCollaborationConflictInspection,
    plan: &ChangeCobResolutionPlan,
    blockers: &[String],
    outcome: Option<&CobMutationOutcome>,
) -> Result<Value, Box<dyn Error>> {
    let blocked = !blockers.is_empty();
    let (is_creator, is_delegate) = embedded_authority(&inspection.state, &plan.actor);
    Ok(json!({
        "command": "collaboration resolve",
        "compatibility_command": "cob resolve",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": inspection.state.repository_id,
        "base_revision": {
            "kind": "git_oid",
            "value": inspection.repository_head_oid,
        },
        "object_id": inspection.object_id,
        "actor": {
            "id": plan.actor,
            "source": "explicit_cob_keystore_and_embedded_authority",
            "cryptographically_verified": outcome.is_some(),
            "currently_authorized": is_creator || is_delegate,
            "authority_class": if is_creator { "signed_creator" } else { "embedded_delegate" },
        },
        "delegates": inspection.state.delegates,
        "policy": policy_name,
        "policy_hash": inspection.state.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": inspection.state.policy_hash == observed_policy_hash,
        "input_hash": plan.input_state_hash,
        "plan_hash": change_cob_resolution_plan_hash(plan)?,
        "operation_plan_id": plan.operation_id,
        "input_state_hash": plan.input_state_hash,
        "expected_content_root": plan.expected_content_root,
        "result_operation_id": outcome.map(|value| value.operation_id.as_str()),
        "result_state_hash": outcome.map(|value| value.state_hash.as_str()),
        "result_content_root": outcome.map(|value| value.content_root.as_str()),
        "artifact_conflicts": conflicts.artifact_conflicts,
        "lifecycle_heads": conflicts.lifecycle_heads,
        "selected_artifacts": plan.selected_artifacts,
        "lifecycle_reset": plan.lifecycle_reset,
        "actions": resolution_actions(&plan.actions),
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": outcome.is_some(),
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    }))
}

fn resolution_actions(actions: &[ChangeCobAction]) -> Vec<Value> {
    actions
        .iter()
        .filter_map(|action| match action {
            ChangeCobAction::SetArtifact {
                artifact,
                content,
                basis,
            } => Some(json!({
                "kind": "set_artifact",
                "artifact": artifact.as_str(),
                "basis": basis,
                "content_blake3": content
                    .as_ref()
                    .map(|value| cairn_core::blake3_hex(value.as_bytes())),
                "content_bytes": content.as_ref().map(String::len),
                "removed": content.is_none(),
            })),
            ChangeCobAction::SetLifecycle {
                status,
                subject_root,
                evidence,
                basis,
            } => Some(json!({
                "kind": "set_lifecycle",
                "status": lifecycle_status_name(*status),
                "subject_root": subject_root,
                "evidence": evidence,
                "basis": basis,
            })),
            ChangeCobAction::Init { .. } | ChangeCobAction::AttachEvidence { .. } => None,
        })
        .collect()
}

fn failure_receipt(options: &CobResolutionOptions, blocker: String) -> Value {
    json!({
        "command": format!("collaboration {}", options.command.as_str()),
        "compatibility_command": format!("cob {}", options.command.as_str()),
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "dry_run": options.command == CobResolutionCommand::Resolve && !options.execute,
        "blocked": true,
        "blockers": [blocker],
        "mutated": false,
        "valid": false,
        "verdict": "FAIL",
        "non_claims": non_claims(),
    })
}

fn parse_options(
    command: CobResolutionCommand,
    args: &[String],
) -> Result<CobResolutionOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err(format!("cob {} requires a change name", command.as_str()).into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut git_repository = None;
    let mut keystore = None;
    let mut policy = None;
    let mut show_head = None;
    let mut selections = Vec::new();
    let mut lifecycle_status = None;
    let mut execute = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--root" => {
                root =
                    Some(read_unique_option_value(args, index, "--root", root.is_some())?.into());
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--git-repository" => {
                git_repository = Some(
                    read_unique_option_value(
                        args,
                        index,
                        "--git-repository",
                        git_repository.is_some(),
                    )?
                    .into(),
                );
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--keystore" if command == CobResolutionCommand::Resolve => {
                keystore = Some(
                    read_unique_option_value(args, index, "--keystore", keystore.is_some())?.into(),
                );
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--policy" => {
                policy = Some(
                    read_unique_option_value(args, index, "--policy", policy.is_some())?.into(),
                );
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--show-head" if command == CobResolutionCommand::Conflicts => {
                let value =
                    read_unique_option_value(args, index, "--show-head", show_head.is_some())?;
                show_head = Some(parse_head_selection(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--choose-head" if command == CobResolutionCommand::Resolve => {
                let value = read_unique_option_value(args, index, "--choose-head", false)?;
                selections.push(parse_head_selection(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--lifecycle" if command == CobResolutionCommand::Resolve => {
                let value = read_unique_option_value(
                    args,
                    index,
                    "--lifecycle",
                    lifecycle_status.is_some(),
                )?;
                lifecycle_status = Some(parse_lifecycle_status(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--execute" if command == CobResolutionCommand::Resolve => {
                if execute {
                    return Err("duplicate option: --execute".into());
                }
                execute = true;
                index = index.saturating_add(1);
            }
            "--rad-home" | "--rid" | "--control-socket" | "--peer-id" | "--peer-address" => {
                return Err(format!(
                    "unsupported Radicle runtime argument for COB-only {}: {}",
                    command.as_str(),
                    args[index]
                )
                .into());
            }
            unknown => {
                return Err(format!("unknown cob {} argument: {unknown}", command.as_str()).into());
            }
        }
    }
    if command == CobResolutionCommand::Resolve && keystore.is_none() {
        return Err("cob resolve requires --keystore PATH".into());
    }
    Ok(CobResolutionOptions {
        command,
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        git_repository: git_repository
            .ok_or_else(|| format!("cob {} requires --git-repository PATH", command.as_str()))?,
        keystore,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        show_head,
        selections,
        lifecycle_status,
        execute,
    })
}

fn required_keystore(options: &CobResolutionOptions) -> Result<&std::path::Path, Box<dyn Error>> {
    options
        .keystore
        .as_deref()
        .ok_or_else(|| "cob resolve requires --keystore PATH".into())
}

fn resolution_actor(options: &CobResolutionOptions) -> Result<String, Box<dyn Error>> {
    Ok(signer_actor(required_keystore(options)?)?)
}

fn parse_head_selection(value: &str) -> Result<ChangeArtifactHeadSelection, Box<dyn Error>> {
    let Some((artifact, revision_id)) = value.split_once('=') else {
        return Err("head selector must use ARTIFACT=REVISION".into());
    };
    if revision_id.is_empty() {
        return Err("head selector revision must not be empty".into());
    }
    if revision_id.len() > MAX_OPERATION_ID_BYTES {
        return Err(
            format!("head selector revision exceeds {MAX_OPERATION_ID_BYTES} bytes").into(),
        );
    }
    Ok(ChangeArtifactHeadSelection {
        artifact: ChangeArtifactId::parse(artifact.to_string())?,
        revision_id: revision_id.to_string(),
    })
}

fn parse_lifecycle_status(value: &str) -> Result<ChangeLifecycleStatus, Box<dyn Error>> {
    match value {
        "open" => Ok(ChangeLifecycleStatus::Open),
        _ => Err("conflict resolution supports only --lifecycle open".into()),
    }
}

fn lifecycle_status_name(status: ChangeLifecycleStatus) -> &'static str {
    match status {
        ChangeLifecycleStatus::Open => "open",
        ChangeLifecycleStatus::ReviewRequested => "review_requested",
        ChangeLifecycleStatus::Approved => "approved",
        ChangeLifecycleStatus::ArchiveRequested => "archive_requested",
        ChangeLifecycleStatus::Archived => "archived",
    }
}

fn non_claims() -> [&'static str; NON_CLAIM_COUNT] {
    [
        NON_CLAIM_AUTHORITY,
        NON_CLAIM_LOCAL,
        NON_CLAIM_MERGE,
        NON_CLAIM_LIFECYCLE,
        NON_CLAIM_TRANSPORT,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANGE_ID: &str = "cob-resolution-cli";

    // r[verify cairn.radicle_cob_only.cli.explicit]
    #[test]
    fn inspection_needs_only_repository_and_resolution_needs_keystore() {
        let conflicts = parse_options(
            CobResolutionCommand::Conflicts,
            &[
                CHANGE_ID.to_string(),
                "--git-repository".to_string(),
                "/tmp/repository".to_string(),
            ],
        )
        .expect("conflict inspection options");
        assert!(conflicts.keystore.is_none());
        assert!(parse_options(
            CobResolutionCommand::Resolve,
            &[
                CHANGE_ID.to_string(),
                "--git-repository".to_string(),
                "/tmp/repository".to_string(),
            ],
        )
        .is_err());
    }
}
