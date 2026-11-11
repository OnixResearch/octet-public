use std::{error::Error, path::PathBuf};

use cairn_core::{
    change_cob_update_plan_hash, plan_change_cob_update, validate_change_name, ChangeCobAction,
    ChangeCobUpdatePlan, ChangeCollaborationUpdateSnapshot,
};
use onix_cairn_radicle_cob::{
    embedded_authority, inspect_change, signer_actor, update_change, CobChangeInspection,
    CobMutationLock, CobMutationOutcome,
};
use serde_json::{json, Value};

use crate::{
    collaboration::{
        materialize::RepositoryCheckoutLock, portable_store::read_active_change_artifacts,
    },
    support::{
        default_policy_path, load_policy_file, path_string, print_json, with_portable_receipt_hash,
    },
};

use super::{
    format_library_versions, read_unique_option_value, COB_BACKEND_V1, OPTION_TOKEN_WIDTH,
};

const UPDATE_NON_CLAIM_LOCAL: &str =
    "a signed update proves bounded local Git COB observations, not node participation, publication, peer availability, synchronization, convergence, or remote trust";
const UPDATE_NON_CLAIM_LIFECYCLE: &str =
    "collaboration update does not prove gate validity, evidence truth, dependency readiness, source correctness, archive execution, or release eligibility";
const UPDATE_NON_CLAIM_AUTHORITY: &str =
    "embedded creator and delegate keys are immutable COB authority, not Radicle identity-document or future authorization proof";
const UPDATE_NON_CLAIM_CONFLICT: &str =
    "ordinary collaboration update refuses conflicts and does not prove or perform conflict resolution";
const BLOCKER_INPUT_DRIFT: &str =
    "active change, policy, Git repository head, object, signer, embedded authority, or typed state changed after preview";

// r[impl cairn.radicle_cob_only.cli]
// r[impl cairn.radicle_cob_only.docs]
// r[impl cairn.collaboration_update.radicle_execution]

#[derive(Debug, Clone, PartialEq, Eq)]
struct CobUpdateOptions {
    change: String,
    root: PathBuf,
    git_repository: PathBuf,
    keystore: PathBuf,
    policy: PathBuf,
    execute: bool,
}

pub(super) fn run_update(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_update_options(&args)?;
    let inspection = inspect_change(&options.git_repository, &options.change)?;
    let actor = signer_actor(&options.keystore)?;
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = cairn_core::canonical_json_hash(&policy.raw);
    let preview = match build_update_plan(&options, &inspection, &actor, &observed_policy_hash) {
        Ok(plan) => plan,
        Err(error) => {
            return print_json(with_portable_receipt_hash(
                &options.root,
                failure_receipt(&options, error.to_string()),
            ));
        }
    };
    let mut blockers = Vec::new();
    let mut outcome = None;

    if options.execute && !preview.is_noop() {
        match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
            Ok(repository_guard) => {
                match CobMutationLock::acquire(&options.git_repository, &options.change) {
                    Ok(cob_guard) => {
                        let execution = recheck_update_inputs(&options, &inspection, &preview)
                            .and_then(|(current, current_plan)| {
                                update_change(
                                    &options.git_repository,
                                    &options.keystore,
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
                            Ok(updated) => outcome = Some(updated),
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
    let receipt = update_receipt(
        &options,
        &policy.name,
        &observed_policy_hash,
        &inspection,
        &preview,
        &blockers,
        outcome.as_ref(),
    )?;
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

fn parse_update_options(args: &[String]) -> Result<CobUpdateOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err("cob update requires a change name".into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut git_repository = None;
    let mut keystore = None;
    let mut policy = None;
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
            "--keystore" => {
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
            "--execute" => {
                if execute {
                    return Err("duplicate option: --execute".into());
                }
                execute = true;
                index = index.saturating_add(1);
            }
            "--rad-home" | "--rid" | "--control-socket" | "--peer-id" | "--peer-address" => {
                return Err(format!(
                    "unsupported Radicle runtime argument for COB-only update: {}",
                    args[index]
                )
                .into());
            }
            unknown => return Err(format!("unknown cob update argument: {unknown}").into()),
        }
    }
    Ok(CobUpdateOptions {
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        git_repository: git_repository.ok_or("cob update requires --git-repository PATH")?,
        keystore: keystore.ok_or("cob update requires --keystore PATH")?,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        execute,
    })
}

fn build_update_plan(
    options: &CobUpdateOptions,
    inspection: &CobChangeInspection,
    actor: &str,
    policy_hash: &str,
) -> Result<ChangeCobUpdatePlan, Box<dyn Error>> {
    let artifacts = read_active_change_artifacts(&options.root, &options.change)?;
    let (_, is_delegate) = embedded_authority(&inspection.state, actor);
    Ok(plan_change_cob_update(
        &inspection.state,
        ChangeCollaborationUpdateSnapshot {
            change_id: options.change.clone(),
            policy_hash: policy_hash.to_string(),
            actor: actor.to_string(),
            is_delegate,
            proposal: artifacts.proposal,
            design: artifacts.design,
            tasks: artifacts.tasks,
            metadata: artifacts.metadata,
            delta_specs: artifacts.delta_specs,
        },
    )?)
}

fn recheck_update_inputs(
    options: &CobUpdateOptions,
    expected_inspection: &CobChangeInspection,
    expected_plan: &ChangeCobUpdatePlan,
) -> Result<(CobChangeInspection, ChangeCobUpdatePlan), Box<dyn Error>> {
    let current_policy = load_policy_file(options.policy.clone())?;
    let current_policy_hash = cairn_core::canonical_json_hash(&current_policy.raw);
    let current = inspect_change(&options.git_repository, &options.change)?;
    let actor = signer_actor(&options.keystore)?;
    let current_plan = build_update_plan(options, &current, &actor, &current_policy_hash)?;
    if &current != expected_inspection || &current_plan != expected_plan {
        return Err(BLOCKER_INPUT_DRIFT.into());
    }
    Ok((current, current_plan))
}

fn update_receipt(
    options: &CobUpdateOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    inspection: &CobChangeInspection,
    plan: &ChangeCobUpdatePlan,
    blockers: &[String],
    outcome: Option<&CobMutationOutcome>,
) -> Result<Value, Box<dyn Error>> {
    let updates = plan
        .actions
        .iter()
        .filter_map(|action| match action {
            ChangeCobAction::SetArtifact {
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
            ChangeCobAction::Init { .. }
            | ChangeCobAction::AttachEvidence { .. }
            | ChangeCobAction::SetLifecycle { .. } => None,
        })
        .collect::<Vec<_>>();
    let blocked = !blockers.is_empty();
    let (is_creator, is_delegate) = embedded_authority(&inspection.state, &plan.actor);
    Ok(json!({
        "command": "collaboration update",
        "compatibility_command": "cob update",
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
        "plan_hash": change_cob_update_plan_hash(plan)?,
        "operation_plan_id": plan.operation_id,
        "input_state_hash": plan.input_state_hash,
        "input_content_root": plan.input_content_root,
        "expected_content_root": plan.expected_content_root,
        "result_operation_id": outcome.map(|value| value.operation_id.as_str()),
        "result_state_hash": outcome.map(|value| value.state_hash.as_str()),
        "result_content_root": outcome.map(|value| value.content_root.as_str()),
        "changed_artifacts": plan.changed_artifacts,
        "updates": updates,
        "noop": plan.is_noop(),
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": outcome.is_some(),
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    }))
}

fn failure_receipt(options: &CobUpdateOptions, blocker: String) -> Value {
    json!({
        "command": "collaboration update",
        "compatibility_command": "cob update",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
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

fn non_claims() -> [&'static str; 4] {
    [
        UPDATE_NON_CLAIM_AUTHORITY,
        UPDATE_NON_CLAIM_LOCAL,
        UPDATE_NON_CLAIM_LIFECYCLE,
        UPDATE_NON_CLAIM_CONFLICT,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANGE_ID: &str = "cob-update-preview";

    // r[verify cairn.radicle_cob_only.cli.explicit]
    // r[verify cairn.radicle_cob_only.cli.reject_runtime]
    #[test]
    fn update_arguments_require_repository_and_keystore_without_runtime_context() {
        assert!(parse_update_options(&[CHANGE_ID.to_string()]).is_err());
        let parsed = parse_update_options(&[
            CHANGE_ID.to_string(),
            "--git-repository".to_string(),
            "/tmp/repository".to_string(),
            "--keystore".to_string(),
            "/tmp/keys".to_string(),
        ])
        .expect("explicit COB options");
        assert_eq!(parsed.change, CHANGE_ID);
        assert!(!parsed.execute);
        assert!(parse_update_options(&[
            CHANGE_ID.to_string(),
            "--rad-home".to_string(),
            "/tmp/radicle".to_string(),
        ])
        .is_err());
    }
}
