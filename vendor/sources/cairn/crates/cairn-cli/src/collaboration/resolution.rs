use std::{error::Error, path::PathBuf};

use cairn_core::{
    apply_planned_change_collaboration_resolution, build_portable_change_collaboration,
    canonical_json_hash, change_collaboration_resolution_plan_hash,
    inspect_change_collaboration_conflict_value, inspect_change_collaboration_conflicts,
    plan_change_collaboration_resolution, validate_change_name, ChangeArtifactHeadSelection,
    ChangeArtifactId, ChangeCollaborationAction, ChangeCollaborationConflictInspection,
    ChangeCollaborationResolutionPlan, ChangeCollaborationResolutionSnapshot,
    ChangeLifecycleStatus, PortableChangeCollaboration, MAX_OPERATION_ID_BYTES,
    PORTABLE_COLLABORATION_BACKEND_V1,
};
use serde_json::{json, Value};

use crate::support::{default_policy_path, load_policy_file, path_string};

use super::{
    materialize::RepositoryCheckoutLock,
    portable_store::{load_store, replace_store, PortableStoreLock},
};

const OPTION_TOKEN_WIDTH: usize = 2;
const BLOCKER_CHANGE_MISMATCH: &str =
    "portable collaboration store contains a different change identifier";
const BLOCKER_INPUT_DRIFT: &str =
    "policy, portable state, conflict heads, or resolution choices changed after preview";
const NON_CLAIM_AUTHORITY: &str =
    "portable actor and workspace values are caller declarations, not cryptographic identity or remote authority proof";
const NON_CLAIM_LOCAL: &str =
    "portable conflict resolution proves bounded local DAG and filesystem observations only";
const NON_CLAIM_MERGE: &str =
    "selecting a conflict head does not prove semantic merge correctness or that losing heads are invalid";
const NON_CLAIM_LIFECYCLE: &str =
    "an explicit lifecycle reset to open does not prove review, approval, evidence truth, source correctness, archive execution, or release eligibility";
const NON_CLAIM_TRANSPORT: &str =
    "local portable resolution does not prove network publication, peer convergence, availability, or remote trust";
const NON_CLAIM_COUNT: usize = 5;

// r[impl cairn.collaboration_resolution.portable_execution]
// r[impl cairn.collaboration_resolution.receipts]
// r[impl cairn.collaboration_resolution.non_claims]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PortableResolutionCommand {
    Conflicts,
    Resolve,
}

impl PortableResolutionCommand {
    fn as_str(self) -> &'static str {
        match self {
            Self::Conflicts => "conflicts",
            Self::Resolve => "resolve",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PortableResolutionOptions {
    command: PortableResolutionCommand,
    change: String,
    root: PathBuf,
    store: PathBuf,
    actor: Option<String>,
    policy: PathBuf,
    show_head: Option<ChangeArtifactHeadSelection>,
    selections: Vec<ChangeArtifactHeadSelection>,
    lifecycle_status: Option<ChangeLifecycleStatus>,
    execute: bool,
}

pub(super) fn run_portable_conflicts(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let options = parse_options(PortableResolutionCommand::Conflicts, args)?;
    portable_conflicts(&options)
}

pub(super) fn run_portable_resolve(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let options = parse_options(PortableResolutionCommand::Resolve, args)?;
    portable_resolve(&options)
}

fn portable_conflicts(options: &PortableResolutionOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, PortableResolutionCommand::Conflicts);
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let envelope = match load_store(&options.store) {
        Ok(envelope) => envelope,
        Err(error) => return Ok(failure_receipt(options, error.to_string())),
    };
    if envelope.state.change_id != options.change {
        return Ok(failure_receipt(
            options,
            BLOCKER_CHANGE_MISMATCH.to_string(),
        ));
    }
    let inspection = inspect_change_collaboration_conflicts(&envelope.state)?;
    let shown_head = match &options.show_head {
        Some(selection) => match inspect_change_collaboration_conflict_value(
            &envelope.state,
            &selection.artifact,
            &selection.revision_id,
        ) {
            Ok(value) => Some(value),
            Err(error) => return Ok(failure_receipt(options, error.to_string())),
        },
        None => None,
    };
    Ok(json!({
        "command": "collaboration conflicts",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": envelope.state.workspace_id,
        "base_revision": envelope.state.base_revision,
        "actor": portable_actor_observation(&envelope.state.creator),
        "policy": policy.name,
        "policy_hash": envelope.state.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": envelope.state.policy_hash == observed_policy_hash,
        "state_hash": inspection.state_hash,
        "envelope_hash": envelope.envelope_hash,
        "conflicted": inspection.has_conflicts(),
        "artifact_conflicts": inspection.artifact_conflicts,
        "lifecycle_heads": inspection.lifecycle_heads,
        "shown_head": shown_head,
        "blocked": false,
        "blockers": [],
        "mutated": false,
        "valid": true,
        "verdict": "PASS",
        "non_claims": non_claims(),
    }))
}

fn portable_resolve(options: &PortableResolutionOptions) -> Result<Value, Box<dyn Error>> {
    debug_assert_eq!(options.command, PortableResolutionCommand::Resolve);
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let envelope = match load_store(&options.store) {
        Ok(envelope) => envelope,
        Err(error) => return Ok(failure_receipt(options, error.to_string())),
    };
    if envelope.state.change_id != options.change {
        return Ok(failure_receipt(
            options,
            BLOCKER_CHANGE_MISMATCH.to_string(),
        ));
    }
    let (inspection, plan, candidate) =
        match build_resolution_candidate(options, &observed_policy_hash, &envelope) {
            Ok(value) => value,
            Err(error) => return Ok(failure_receipt(options, error.to_string())),
        };
    let mut blockers = Vec::new();
    let mut mutated = false;

    if options.execute {
        match PortableStoreLock::acquire(&options.store, false) {
            Ok(_store_guard) => {
                match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
                    Ok(repository_guard) => {
                        let execution = (|| {
                            let current_policy = load_policy_file(options.policy.clone())?;
                            let current_policy_hash = canonical_json_hash(&current_policy.raw);
                            let current_envelope = load_store(&options.store)?;
                            let (current_inspection, current_plan, current_candidate) =
                                build_resolution_candidate(
                                    options,
                                    &current_policy_hash,
                                    &current_envelope,
                                )?;
                            if current_envelope != envelope
                                || current_inspection != inspection
                                || current_plan != plan
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
    resolution_receipt(
        options,
        &policy.name,
        &observed_policy_hash,
        &envelope,
        &inspection,
        &plan,
        &candidate,
        &blockers,
        mutated,
    )
}

fn build_resolution_candidate(
    options: &PortableResolutionOptions,
    policy_hash: &str,
    envelope: &PortableChangeCollaboration,
) -> Result<
    (
        ChangeCollaborationConflictInspection,
        ChangeCollaborationResolutionPlan,
        PortableChangeCollaboration,
    ),
    Box<dyn Error>,
> {
    let inspection = inspect_change_collaboration_conflicts(&envelope.state)?;
    let plan = plan_change_collaboration_resolution(
        &envelope.state,
        ChangeCollaborationResolutionSnapshot {
            change_id: options.change.clone(),
            policy_hash: policy_hash.to_string(),
            actor: options
                .actor
                .clone()
                .ok_or("portable resolution actor is missing")?,
            is_delegate: false,
            artifact_selections: options.selections.clone(),
            lifecycle_status: options.lifecycle_status,
        },
    )?;
    let state = apply_planned_change_collaboration_resolution(&envelope.state, &plan)?;
    let candidate = build_portable_change_collaboration(state)?;
    if candidate.state_hash != plan.expected_state_hash
        || candidate.content_root.as_deref() != Some(plan.expected_content_root.as_str())
    {
        return Err("portable resolution candidate differs from pure plan".into());
    }
    Ok((inspection, plan, candidate))
}

#[allow(clippy::too_many_arguments)]
fn resolution_receipt(
    options: &PortableResolutionOptions,
    policy_name: &str,
    observed_policy_hash: &str,
    input: &PortableChangeCollaboration,
    inspection: &ChangeCollaborationConflictInspection,
    plan: &ChangeCollaborationResolutionPlan,
    candidate: &PortableChangeCollaboration,
    blockers: &[String],
    mutated: bool,
) -> Result<Value, Box<dyn Error>> {
    let blocked = !blockers.is_empty();
    Ok(json!({
        "command": "collaboration resolve",
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "workspace_id": input.state.workspace_id,
        "base_revision": input.state.base_revision,
        "actor": portable_actor_observation(&plan.actor),
        "policy": policy_name,
        "policy_hash": input.state.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": input.state.policy_hash == observed_policy_hash,
        "input_hash": plan.input_state_hash,
        "plan_hash": change_collaboration_resolution_plan_hash(plan)?,
        "operation_id": plan.operation_id,
        "input_state_hash": plan.input_state_hash,
        "input_envelope_hash": input.envelope_hash,
        "expected_state_hash": plan.expected_state_hash,
        "expected_content_root": plan.expected_content_root,
        "expected_envelope_hash": candidate.envelope_hash,
        "artifact_conflicts": inspection.artifact_conflicts,
        "lifecycle_heads": inspection.lifecycle_heads,
        "selected_artifacts": plan.selected_artifacts,
        "lifecycle_reset": plan.lifecycle_reset,
        "actions": resolution_actions(&plan.actions),
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": mutated,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": non_claims(),
    }))
}

fn resolution_actions(actions: &[ChangeCollaborationAction]) -> Vec<Value> {
    actions
        .iter()
        .filter_map(|action| match action {
            ChangeCollaborationAction::SetArtifact {
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
            ChangeCollaborationAction::SetLifecycle {
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
            ChangeCollaborationAction::Init { .. }
            | ChangeCollaborationAction::AttachEvidence { .. } => None,
        })
        .collect()
}

fn failure_receipt(options: &PortableResolutionOptions, blocker: String) -> Value {
    json!({
        "command": format!("collaboration {}", options.command.as_str()),
        "backend": PORTABLE_COLLABORATION_BACKEND_V1,
        "root": path_string(&options.root),
        "change": options.change,
        "dry_run": options.command == PortableResolutionCommand::Resolve && !options.execute,
        "blocked": true,
        "blockers": [blocker],
        "mutated": false,
        "valid": false,
        "verdict": "FAIL",
        "non_claims": non_claims(),
    })
}

fn portable_actor_observation(actor: &str) -> Value {
    json!({
        "id": actor,
        "source": "caller_declared",
        "cryptographically_verified": false,
        "authority_class": "portable_creator_declaration",
    })
}

pub(super) fn parse_options(
    command: PortableResolutionCommand,
    args: &[String],
) -> Result<PortableResolutionOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err(format!("collaboration {} requires a change name", command.as_str()).into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut store = None;
    let mut actor = None;
    let mut policy = None;
    let mut show_head = None;
    let mut selections = Vec::new();
    let mut lifecycle_status = None;
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
            "--policy" => {
                policy = Some(read_option(args, index, "--policy", policy.is_some())?.into());
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--show-head" if command == PortableResolutionCommand::Conflicts => {
                let value = read_option(args, index, "--show-head", show_head.is_some())?;
                show_head = Some(parse_head_selection(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--actor" if command == PortableResolutionCommand::Resolve => {
                actor = Some(read_option(args, index, "--actor", actor.is_some())?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--choose-head" if command == PortableResolutionCommand::Resolve => {
                let value = read_option(args, index, "--choose-head", false)?;
                selections.push(parse_head_selection(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--lifecycle" if command == PortableResolutionCommand::Resolve => {
                let value = read_option(args, index, "--lifecycle", lifecycle_status.is_some())?;
                lifecycle_status = Some(parse_lifecycle_status(&value)?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--execute" if command == PortableResolutionCommand::Resolve => {
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
    if command == PortableResolutionCommand::Resolve && actor.is_none() {
        return Err("portable collaboration resolve requires --actor ID".into());
    }
    Ok(PortableResolutionOptions {
        command,
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        store: store.ok_or_else(|| {
            format!(
                "collaboration {} with {PORTABLE_COLLABORATION_BACKEND_V1} requires --store PATH",
                command.as_str()
            )
        })?,
        actor,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        show_head,
        selections,
        lifecycle_status,
        execute,
    })
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
    use std::{collections::BTreeMap, fs, io, path::Path};

    use cairn_core::{
        apply_change_collaboration_operation, initialize_change_collaboration_import,
        plan_change_collaboration_import, ChangeBaseRevision, ChangeCollaborationImportSnapshot,
        ChangeCollaborationOperation,
    };

    use super::*;
    use crate::collaboration::portable_store::commit_store;

    const CHANGE_ID: &str = "portable-resolution";
    const ACTOR: &str = "actor:portable";
    const LEFT_OPERATION: &str = "operation:left";
    const RIGHT_OPERATION: &str = "operation:right";
    const TEST_POLICY_RELATIVE: &str = "../../cairn-policy/generated/cairn-policy.json";
    const CONFLICT_HEAD_COUNT: usize = 2;
    const TEST_DIRECTORY_PREFIX: &str = "cairn-portable-resolution";

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

    fn conflicted_store(name: &str) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
        let root = test_directory(name);
        remove_directory(&root)?;
        fs::create_dir_all(root.join(".cairn/changes").join(CHANGE_ID))?;
        let policy_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TEST_POLICY_RELATIVE);
        let policy = load_policy_file(policy_path)?;
        let policy_hash = canonical_json_hash(&policy.raw);
        let init = plan_change_collaboration_import(ChangeCollaborationImportSnapshot {
            change_id: CHANGE_ID.to_string(),
            workspace_id: "workspace:portable-resolution".to_string(),
            base_revision: ChangeBaseRevision::new("opaque", "revision-one")?,
            policy_hash,
            proposal: Some("# Proposal\n".to_string()),
            design: None,
            tasks: None,
            metadata: None,
            delta_specs: BTreeMap::new(),
        })?;
        let mut state = initialize_change_collaboration_import(init, ACTOR)?;
        let basis = state.artifacts[&ChangeArtifactId::proposal()].head_ids();
        for (operation_id, content) in
            [(LEFT_OPERATION, "# Left\n"), (RIGHT_OPERATION, "# Right\n")]
        {
            apply_change_collaboration_operation(
                &mut state,
                ChangeCollaborationOperation {
                    id: operation_id.to_string(),
                    actor: ACTOR.to_string(),
                    is_delegate: false,
                    actions: vec![ChangeCollaborationAction::SetArtifact {
                        artifact: ChangeArtifactId::proposal(),
                        content: Some(content.to_string()),
                        basis: basis.clone(),
                    }],
                },
            )?;
        }
        let store = root.join("portable.json");
        commit_store(&store, &build_portable_change_collaboration(state)?)?;
        Ok((root, store))
    }

    fn options(
        command: PortableResolutionCommand,
        root: &Path,
        store: &Path,
        execute: bool,
    ) -> PortableResolutionOptions {
        PortableResolutionOptions {
            command,
            change: CHANGE_ID.to_string(),
            root: root.to_path_buf(),
            store: store.to_path_buf(),
            actor: if command == PortableResolutionCommand::Resolve {
                Some(ACTOR.to_string())
            } else {
                None
            },
            policy: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(TEST_POLICY_RELATIVE),
            show_head: None,
            selections: if command == PortableResolutionCommand::Resolve {
                vec![ChangeArtifactHeadSelection {
                    artifact: ChangeArtifactId::proposal(),
                    revision_id: LEFT_OPERATION.to_string(),
                }]
            } else {
                Vec::new()
            },
            lifecycle_status: None,
            execute,
        }
    }

    // r[verify cairn.collaboration_resolution.inspection.complete]
    // r[verify cairn.collaboration_resolution.inspection.show]
    #[test]
    fn portable_inspection_lists_heads_and_shows_only_requested_value() -> Result<(), Box<dyn Error>>
    {
        let (root, store) = conflicted_store("inspection")?;
        let inspected = portable_conflicts(&options(
            PortableResolutionCommand::Conflicts,
            &root,
            &store,
            false,
        ))?;
        assert_eq!(inspected["conflicted"], true);
        assert_eq!(
            inspected["artifact_conflicts"][0]["heads"]
                .as_array()
                .map(Vec::len),
            Some(CONFLICT_HEAD_COUNT)
        );
        assert!(inspected["shown_head"].is_null());

        let mut show = options(PortableResolutionCommand::Conflicts, &root, &store, false);
        show.show_head = Some(ChangeArtifactHeadSelection {
            artifact: ChangeArtifactId::proposal(),
            revision_id: RIGHT_OPERATION.to_string(),
        });
        let shown = portable_conflicts(&show)?;
        assert_eq!(shown["shown_head"]["content"], "# Right\n");
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_resolution.portable_execution.success]
    // r[verify cairn.collaboration_resolution.receipts.modes]
    #[test]
    fn portable_resolution_previews_then_replaces_exact_state() -> Result<(), Box<dyn Error>> {
        let (root, store) = conflicted_store("execution")?;
        let before = fs::read(&store)?;
        let preview = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            false,
        ))?;
        assert_eq!(preview["valid"], true);
        assert_eq!(preview["mutated"], false);
        assert_eq!(fs::read(&store)?, before);

        let executed = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(executed["mutated"], true);
        assert_ne!(fs::read(&store)?, before);
        let reloaded = load_store(&store)?;
        assert!(!inspect_change_collaboration_conflicts(&reloaded.state)?.has_conflicts());
        assert_eq!(reloaded.state_hash, preview["expected_state_hash"]);
        remove_directory(&root)?;
        Ok(())
    }

    // r[verify cairn.collaboration_resolution.portable_execution.blocked]
    // r[verify cairn.collaboration_resolution.non_claims.local]
    #[test]
    fn portable_resolution_rejects_incomplete_choice_wrong_actor_and_lock(
    ) -> Result<(), Box<dyn Error>> {
        let (root, store) = conflicted_store("blocked")?;
        let before = fs::read(&store)?;
        let mut incomplete = options(PortableResolutionCommand::Resolve, &root, &store, true);
        incomplete.selections.clear();
        let missing = portable_resolve(&incomplete)?;
        assert_eq!(missing["blocked"], true);
        assert_eq!(fs::read(&store)?, before);

        let mut outsider = options(PortableResolutionCommand::Resolve, &root, &store, true);
        outsider.actor = Some("actor:outsider".to_string());
        let denied = portable_resolve(&outsider)?;
        assert_eq!(denied["blocked"], true);
        assert_eq!(fs::read(&store)?, before);

        let lock = root.join(".portable.json.lock");
        fs::write(&lock, b"stale\n")?;
        let locked = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(locked["blocked"], true);
        assert_eq!(fs::read(&store)?, before);
        assert!(locked["non_claims"]
            .as_array()
            .is_some_and(|claims| claims.iter().any(|claim| claim == NON_CLAIM_MERGE)));
        fs::remove_file(&lock)?;

        let preview = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            false,
        ))?;
        let envelope_hash = preview["expected_envelope_hash"]
            .as_str()
            .ok_or("resolution preview is missing expected envelope hash")?;
        let prepared = root.join(format!(".portable.json.{envelope_hash}.prepared"));
        fs::write(&prepared, b"stale\n")?;
        let stale_prepared = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(stale_prepared["blocked"], true);
        assert_eq!(fs::read(&store)?, before);
        assert!(prepared.exists());
        remove_directory(&root)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn portable_resolution_rejects_symlink_store_without_changing_target(
    ) -> Result<(), Box<dyn Error>> {
        use std::os::unix::fs::symlink;

        let (root, store) = conflicted_store("symlink")?;
        let outside = root.join("outside.json");
        let original = fs::read(&store)?;
        fs::rename(&store, &outside)?;
        symlink(&outside, &store)?;
        let blocked = portable_resolve(&options(
            PortableResolutionCommand::Resolve,
            &root,
            &store,
            true,
        ))?;
        assert_eq!(blocked["blocked"], true);
        assert_eq!(blocked["mutated"], false);
        assert_eq!(fs::read(&outside)?, original);
        remove_directory(&root)?;
        Ok(())
    }

    #[test]
    fn resolution_arguments_are_strict_and_explicit() {
        let args = [
            CHANGE_ID.to_string(),
            "--store".to_string(),
            "store.json".to_string(),
            "--actor".to_string(),
            ACTOR.to_string(),
            "--choose-head".to_string(),
            format!("proposal={LEFT_OPERATION}"),
        ];
        let parsed = parse_options(PortableResolutionCommand::Resolve, &args).unwrap();
        assert_eq!(parsed.selections.len(), 1);
        assert!(parse_options(
            PortableResolutionCommand::Resolve,
            &[
                CHANGE_ID.to_string(),
                "--store".to_string(),
                "store.json".to_string()
            ]
        )
        .is_err());
        assert!(parse_options(
            PortableResolutionCommand::Conflicts,
            &[
                CHANGE_ID.to_string(),
                "--store".to_string(),
                "store.json".to_string(),
                "--execute".to_string(),
            ]
        )
        .is_err());
        assert!(parse_head_selection("proposal").is_err());
        assert!(parse_lifecycle_status("approved").is_err());
    }
}
