use std::{
    collections::BTreeMap,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, plan_change_cob_checkout, validate_change_name, ChangeArtifactId,
    ChangeCheckoutDestination, ChangeCobCheckoutPlan, ChangeCobLocalSnapshot, ChangeLifecycleView,
    MAX_ARTIFACTS, MAX_ARTIFACT_BYTES, RADICLE_CHANGE_COB_TYPE_NAME,
};
use onix_cairn_radicle_cob::{inspect_change, CobChangeInspection, CobMutationLock};
use serde_json::{json, Value};

use crate::{
    collaboration::materialize::RepositoryCheckoutLock,
    support::{
        default_policy_path, load_policy_file, path_string, print_json, with_portable_receipt_hash,
        CapabilityRoot, CapabilityRootKind,
    },
};

use super::{
    format_library_versions, read_unique_option_value, COB_BACKEND_V1, OPTION_TOKEN_WIDTH,
};

const STANDARD_ROOT_ENTRY_LIMIT: usize = 5;
const LOCAL_ROOT_ENTRY_LIMIT: usize = MAX_ARTIFACTS + STANDARD_ROOT_ENTRY_LIMIT;
const SPEC_DIRECTORY_ENTRY_LIMIT: usize = 1;
const CHANGES_DIRECTORY: &str = ".cairn/changes";
const CHECKOUT_PREPARED_DIRECTORY: &str = ".cairn/.cob-checkout-prepared";
const STATUS_NON_CLAIM_LOCAL: &str =
    "typed status proves only reduction and observations in the selected local Git COB store";
const CHECKOUT_NON_CLAIM_LOCAL: &str =
    "checkout proves only local materialization from the stated reduced object";
const NON_CLAIM_LIFECYCLE: &str =
    "status and checkout do not prove gate validity, evidence truth, dependency readiness, source correctness, archive execution, or release eligibility";
const NON_CLAIM_TRANSPORT: &str =
    "status and checkout do not prove node participation, publication, peer availability, synchronization, convergence, or remote trust";
const NON_CLAIM_AUTHORITY: &str =
    "embedded creator and delegate keys are immutable COB authority, not Radicle identity-document or future authorization proof";
const NON_CLAIM_HOST: &str =
    "the cooperative checkout lock is not a sandbox against unrelated processes with equivalent host filesystem authority";
const BLOCKER_INPUT_DRIFT: &str =
    "Git repository head, object, typed state, policy, or local destination changed after checkout preview";
const BLOCKER_CHANGES_DIRECTORY: &str =
    "native .cairn/changes directory is missing, unsafe, or not a directory";
const BLOCKER_PREPARED_EXISTS: &str =
    "prepared checkout directory already exists; inspect interrupted local state before recovery";
const BLOCKER_DESTINATION_APPEARED: &str =
    "active-change destination appeared before atomic checkout exposure";
const BLOCKER_POST_VERIFICATION: &str =
    "materialized active change did not match the typed state after checkout";
const BLOCKER_POST_VERIFICATION_UNAVAILABLE: &str =
    "materialized active change could not be inspected after checkout";

// r[impl cairn.radicle_cob_only.cli]
// r[impl cairn.radicle_cob_only.docs]

#[derive(Debug, Clone, PartialEq, Eq)]
struct CobObjectOptions {
    change: String,
    root: PathBuf,
    git_repository: PathBuf,
    policy: PathBuf,
    execute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckoutExecutionOutcome {
    mutated: bool,
    final_destination: Option<ChangeCheckoutDestination>,
    blockers: Vec<String>,
}

pub(super) fn run_status(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_object_options(&args, "status", false)?;
    let inspection = inspect_change(&options.git_repository, &options.change)?;
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let local = read_local_change_snapshot(&options.root, &options.change)?;
    let plan = plan_change_cob_checkout(&inspection.state, local)?;
    let receipt = status_receipt(
        &options,
        &inspection,
        &policy.name,
        &observed_policy_hash,
        &plan,
    );
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

pub(super) fn run_checkout(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_object_options(&args, "checkout", true)?;
    let initial_inspection = inspect_change(&options.git_repository, &options.change)?;
    let policy = load_policy_file(options.policy.clone())?;
    let observed_policy_hash = canonical_json_hash(&policy.raw);
    let local = read_local_change_snapshot(&options.root, &options.change)?;
    let initial_plan = plan_change_cob_checkout(&initial_inspection.state, local)?;
    let mut blockers = checkout_blockers(&initial_inspection, &initial_plan);
    let mut outcome = CheckoutExecutionOutcome {
        mutated: false,
        final_destination: None,
        blockers: Vec::new(),
    };
    let mut receipt_inspection = initial_inspection.clone();

    if options.execute && blockers.is_empty() && !initial_plan.already_current() {
        match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
            Ok(repository_guard) => {
                match CobMutationLock::acquire(&options.git_repository, &options.change) {
                    Ok(cob_guard) => {
                        let current = inspect_change(&options.git_repository, &options.change);
                        let current_policy = load_policy_file(options.policy.clone())
                            .map(|policy| canonical_json_hash(&policy.raw));
                        match (current, current_policy) {
                            (Ok(current), Ok(current_policy_hash))
                                if current == initial_inspection
                                    && current_policy_hash == observed_policy_hash =>
                            {
                                receipt_inspection = current.clone();
                                match execute_guarded_checkout(&options, &initial_plan, &current) {
                                    Ok(executed) => outcome = executed,
                                    Err(error) => blockers.push(error.to_string()),
                                }
                            }
                            (Ok(_), Ok(_)) => blockers.push(BLOCKER_INPUT_DRIFT.to_string()),
                            (Err(error), _) => blockers.push(error.to_string()),
                            (_, Err(error)) => blockers.push(error.to_string()),
                        }
                        if let Err(error) = cob_guard.release() {
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
    blockers.append(&mut outcome.blockers);
    blockers.sort();
    blockers.dedup();
    outcome.blockers = blockers;

    let receipt = checkout_receipt(
        &options,
        &receipt_inspection,
        &policy.name,
        &observed_policy_hash,
        &initial_plan,
        &outcome,
    );
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

fn execute_guarded_checkout(
    options: &CobObjectOptions,
    preview: &ChangeCobCheckoutPlan,
    inspection: &CobChangeInspection,
) -> Result<CheckoutExecutionOutcome, Box<dyn Error>> {
    let current_local = read_local_change_snapshot(&options.root, &options.change)?;
    let current = plan_change_cob_checkout(&inspection.state, current_local)?;
    if &current != preview {
        return Ok(CheckoutExecutionOutcome {
            mutated: false,
            final_destination: None,
            blockers: vec![BLOCKER_INPUT_DRIFT.to_string()],
        });
    }
    materialize_checkout(
        &options.root,
        &inspection.state.repository_id,
        &inspection.state,
        &current,
    )
}

fn materialize_checkout(
    root: &Path,
    repository_id: &str,
    state: &cairn_core::ChangeCobState,
    plan: &ChangeCobCheckoutPlan,
) -> Result<CheckoutExecutionOutcome, Box<dyn Error>> {
    debug_assert!(plan.checkout_ready());
    debug_assert_eq!(plan.destination, ChangeCheckoutDestination::Absent);
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    if !safe_existing_directory(&repository, Path::new(CHANGES_DIRECTORY))? {
        return Ok(blocked_execution(BLOCKER_CHANGES_DIRECTORY));
    }

    let repository_key = cairn_core::blake3_hex(repository_id.as_bytes());
    let prepared_parent = PathBuf::from(CHECKOUT_PREPARED_DIRECTORY).join(repository_key);
    let prepared = prepared_parent.join(format!("{}-{}", plan.change_id, plan.state_hash));
    if path_exists(&repository, &prepared)? {
        return Ok(blocked_execution(BLOCKER_PREPARED_EXISTS));
    }
    repository.create_dir_all(&prepared_parent)?;
    match repository.create_dir(&prepared) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Ok(blocked_execution(BLOCKER_PREPARED_EXISTS));
        }
        Err(error) => return Err(error.into()),
    }

    if let Err(error) = write_and_verify_prepared(&repository, &prepared, plan) {
        return Err(cleanup_after_prepare_error(&repository, &prepared, error));
    }

    let destination = PathBuf::from(CHANGES_DIRECTORY).join(&plan.change_id);
    if path_exists(&repository, &destination)? {
        repository.remove_dir_all(&prepared)?;
        return Ok(blocked_execution(BLOCKER_DESTINATION_APPEARED));
    }
    repository.rename(&prepared, &repository, &destination)?;

    let post_plan = read_local_change_snapshot(root, &plan.change_id)
        .and_then(|local| plan_change_cob_checkout(state, local).map_err(Into::into));
    match post_plan {
        Ok(post_plan) => {
            let blockers = if post_plan.already_current() {
                Vec::new()
            } else {
                vec![BLOCKER_POST_VERIFICATION.to_string()]
            };
            Ok(CheckoutExecutionOutcome {
                mutated: true,
                final_destination: Some(post_plan.destination),
                blockers,
            })
        }
        Err(_) => Ok(CheckoutExecutionOutcome {
            mutated: true,
            final_destination: None,
            blockers: vec![BLOCKER_POST_VERIFICATION_UNAVAILABLE.to_string()],
        }),
    }
}

fn write_and_verify_prepared(
    repository: &CapabilityRoot,
    prepared: &Path,
    plan: &ChangeCobCheckoutPlan,
) -> Result<(), Box<dyn Error>> {
    for file in &plan.files {
        let output = prepared.join(&file.path);
        let parent = output.parent().unwrap_or(prepared);
        repository.create_dir_all(parent)?;
        repository.write(&output, file.content.as_bytes())?;
        let max_bytes = u64::try_from(file.byte_length)?;
        let restored = repository.read_bounded(&output, max_bytes)?;
        if restored.len() != file.byte_length || cairn_core::blake3_hex(&restored) != file.blake3 {
            return Err(
                format!("prepared checkout file failed verification: {}", file.path).into(),
            );
        }
    }
    Ok(())
}

fn cleanup_after_prepare_error(
    repository: &CapabilityRoot,
    prepared: &Path,
    error: Box<dyn Error>,
) -> Box<dyn Error> {
    match repository.remove_dir_all(prepared) {
        Ok(()) => error,
        Err(cleanup) => {
            format!("checkout preparation failed ({error}) and prepared cleanup failed ({cleanup})")
                .into()
        }
    }
}

fn blocked_execution(blocker: &str) -> CheckoutExecutionOutcome {
    CheckoutExecutionOutcome {
        mutated: false,
        final_destination: None,
        blockers: vec![blocker.to_string()],
    }
}

fn parse_object_options(
    args: &[String],
    command: &str,
    allow_execute: bool,
) -> Result<CobObjectOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err(format!("cob {command} requires a change name").into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut git_repository = None;
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
            "--policy" => {
                policy = Some(
                    read_unique_option_value(args, index, "--policy", policy.is_some())?.into(),
                );
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--execute" if allow_execute => {
                if execute {
                    return Err("duplicate option: --execute".into());
                }
                execute = true;
                index = index.saturating_add(1);
            }
            "--rad-home" | "--rid" | "--control-socket" | "--peer-id" | "--peer-address" => {
                return Err(format!(
                    "unsupported Radicle runtime argument for COB-only {command}: {}",
                    args[index]
                )
                .into());
            }
            unknown => return Err(format!("unknown cob {command} argument: {unknown}").into()),
        }
    }
    Ok(CobObjectOptions {
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        git_repository: git_repository
            .ok_or_else(|| format!("cob {command} requires --git-repository PATH"))?,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        execute,
    })
}

pub(crate) fn read_local_change_snapshot(
    root: &Path,
    change: &str,
) -> Result<ChangeCobLocalSnapshot, Box<dyn Error>> {
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let change_directory = PathBuf::from(CHANGES_DIRECTORY).join(change);
    let metadata = match repository.symlink_metadata(&change_directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(ChangeCobLocalSnapshot::absent())
        }
        Err(error) => return Err(error.into()),
    };
    let mut snapshot = ChangeCobLocalSnapshot {
        exists: true,
        files: BTreeMap::new(),
        issues: Vec::new(),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        snapshot.issues.push(format!(
            "local active-change path is not a safe directory: {}",
            change_directory.display()
        ));
        return Ok(snapshot);
    }

    let mut entries = match repository.entries_bounded(&change_directory, LOCAL_ROOT_ENTRY_LIMIT) {
        Ok(entries) => entries,
        Err(error) => {
            snapshot.issues.push(format!(
                "local active-change directory could not be scanned within limits: {error}"
            ));
            return Ok(snapshot);
        }
    };
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        let name = entry.file_name().to_string_lossy();
        match name.as_ref() {
            "proposal.md" | "design.md" | "tasks.md" | "metadata.json" => {
                read_local_artifact(
                    &repository,
                    &change_directory.join(name.as_ref()),
                    name.as_ref(),
                    entry.file_type().is_file(),
                    entry.file_type().is_symlink(),
                    &mut snapshot,
                )?;
            }
            "specs" => {
                read_local_specs(
                    &repository,
                    &change_directory.join("specs"),
                    entry.file_type().is_dir(),
                    entry.file_type().is_symlink(),
                    &mut snapshot,
                )?;
            }
            _ => snapshot
                .issues
                .push(format!("unsupported local entry: {name}")),
        }
    }
    Ok(snapshot)
}

fn read_local_specs(
    repository: &CapabilityRoot,
    specs_directory: &Path,
    is_directory: bool,
    is_symlink: bool,
    snapshot: &mut ChangeCobLocalSnapshot,
) -> Result<(), Box<dyn Error>> {
    if is_symlink || !is_directory {
        snapshot.issues.push(format!(
            "local specs path is not a safe directory: {}",
            specs_directory.display()
        ));
        return Ok(());
    }
    let mut entries = match repository.entries_bounded(specs_directory, MAX_ARTIFACTS) {
        Ok(entries) => entries,
        Err(error) => {
            snapshot.issues.push(format!(
                "local specs directory could not be scanned within limits: {error}"
            ));
            return Ok(());
        }
    };
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        let spec_id = entry.file_name().to_string_lossy().to_string();
        let artifact = match ChangeArtifactId::delta_spec(&spec_id) {
            Ok(artifact) => artifact,
            Err(_) => {
                snapshot
                    .issues
                    .push(format!("unsupported local spec identifier: {spec_id}"));
                continue;
            }
        };
        let spec_directory = specs_directory.join(&spec_id);
        if entry.file_type().is_symlink() || !entry.file_type().is_dir() {
            snapshot.issues.push(format!(
                "local spec path is not a safe directory: {}",
                spec_directory.display()
            ));
            continue;
        }
        let spec_entries =
            match repository.entries_bounded(&spec_directory, SPEC_DIRECTORY_ENTRY_LIMIT) {
                Ok(entries) => entries,
                Err(error) => {
                    snapshot.issues.push(format!(
                        "local spec directory must contain only spec.md: {spec_id}: {error}"
                    ));
                    continue;
                }
            };
        let Some(spec_entry) = spec_entries.first() else {
            snapshot
                .issues
                .push(format!("local spec file is missing: {spec_id}/spec.md"));
            continue;
        };
        if spec_entry.file_name() != "spec.md" {
            snapshot.issues.push(format!(
                "unsupported local spec entry: {spec_id}/{}",
                spec_entry.file_name().to_string_lossy()
            ));
            continue;
        }
        let relative_path = artifact.relative_path()?;
        read_local_artifact(
            repository,
            &spec_directory.join("spec.md"),
            &relative_path,
            spec_entry.file_type().is_file(),
            spec_entry.file_type().is_symlink(),
            snapshot,
        )?;
    }
    Ok(())
}

fn read_local_artifact(
    repository: &CapabilityRoot,
    path: &Path,
    relative_path: &str,
    is_file: bool,
    is_symlink: bool,
    snapshot: &mut ChangeCobLocalSnapshot,
) -> Result<(), Box<dyn Error>> {
    if is_symlink || !is_file {
        snapshot.issues.push(format!(
            "local artifact is not a safe regular file: {relative_path}"
        ));
        return Ok(());
    }
    let max_bytes = u64::try_from(MAX_ARTIFACT_BYTES)?;
    let bytes = match repository.read_bounded(path, max_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            snapshot.issues.push(format!(
                "local artifact could not be read within limits: {relative_path}: {error}"
            ));
            return Ok(());
        }
    };
    match String::from_utf8(bytes) {
        Ok(content) => {
            snapshot.files.insert(relative_path.to_string(), content);
        }
        Err(_) => snapshot
            .issues
            .push(format!("local artifact is not UTF-8: {relative_path}")),
    }
    Ok(())
}

fn checkout_blockers(
    _inspection: &CobChangeInspection,
    plan: &ChangeCobCheckoutPlan,
) -> Vec<String> {
    let mut blockers = plan.blockers.clone();
    blockers.sort();
    blockers.dedup();
    blockers
}

fn status_receipt(
    options: &CobObjectOptions,
    inspection: &CobChangeInspection,
    policy_name: &str,
    observed_policy_hash: &str,
    plan: &ChangeCobCheckoutPlan,
) -> Value {
    let files = checkout_file_receipts(plan);
    let checkout_blockers = checkout_blockers(inspection, plan);
    let plan_hash = checkout_plan_hash(
        inspection,
        policy_name,
        observed_policy_hash,
        plan,
        &checkout_blockers,
    );
    json!({
        "command": "collaboration status",
        "compatibility_command": "cob status",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "type_name": RADICLE_CHANGE_COB_TYPE_NAME,
        "workspace_id": inspection.state.repository_id,
        "repository_head_oid": inspection.repository_head_oid,
        "object_id": inspection.object_id,
        "authority": {
            "creator": inspection.state.creator,
            "delegates": inspection.state.delegates,
            "source": "signed_root_embedded_authority",
        },
        "base_git_oid": plan.base_git_oid,
        "policy": policy_name,
        "object_policy_hash": plan.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": plan.policy_hash == observed_policy_hash,
        "state_hash": plan.state_hash,
        "content_root": plan.content_root,
        "content_conflicts": plan.content_conflicts,
        "lifecycle": lifecycle_value(&plan.lifecycle),
        "evidence_count": plan.evidence_count,
        "current_evidence_count": plan.current_evidence_count,
        "files": files,
        "file_count": plan.files.len(),
        "local_destination": plan.destination.as_str(),
        "checkout_ready": checkout_blockers.is_empty(),
        "checkout_blockers": checkout_blockers,
        "plan_hash": plan_hash,
        "mutated": false,
        "valid": true,
        "verdict": "PASS",
        "non_claims": [
            STATUS_NON_CLAIM_LOCAL,
            NON_CLAIM_AUTHORITY,
            NON_CLAIM_LIFECYCLE,
            NON_CLAIM_TRANSPORT,
            NON_CLAIM_HOST,
        ],
    })
}

fn checkout_receipt(
    options: &CobObjectOptions,
    inspection: &CobChangeInspection,
    policy_name: &str,
    observed_policy_hash: &str,
    plan: &ChangeCobCheckoutPlan,
    outcome: &CheckoutExecutionOutcome,
) -> Value {
    let blocked = !outcome.blockers.is_empty();
    let files = checkout_file_receipts(plan);
    let plan_hash = checkout_plan_hash(
        inspection,
        policy_name,
        observed_policy_hash,
        plan,
        &outcome.blockers,
    );
    json!({
        "command": "collaboration checkout",
        "compatibility_command": "cob checkout",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "type_name": RADICLE_CHANGE_COB_TYPE_NAME,
        "workspace_id": inspection.state.repository_id,
        "repository_head_oid": inspection.repository_head_oid,
        "object_id": inspection.object_id,
        "authority": {
            "creator": inspection.state.creator,
            "delegates": inspection.state.delegates,
            "source": "signed_root_embedded_authority",
        },
        "base_git_oid": plan.base_git_oid,
        "policy": policy_name,
        "object_policy_hash": plan.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "policy_current": plan.policy_hash == observed_policy_hash,
        "state_hash": plan.state_hash,
        "content_root": plan.content_root,
        "content_conflicts": plan.content_conflicts,
        "lifecycle": lifecycle_value(&plan.lifecycle),
        "evidence_count": plan.evidence_count,
        "current_evidence_count": plan.current_evidence_count,
        "files": files,
        "file_count": plan.files.len(),
        "local_destination": plan.destination.as_str(),
        "already_current": plan.already_current(),
        "post_checkout_destination": outcome.final_destination.map(ChangeCheckoutDestination::as_str),
        "plan_hash": plan_hash,
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": outcome.blockers,
        "mutated": outcome.mutated,
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": [
            CHECKOUT_NON_CLAIM_LOCAL,
            NON_CLAIM_AUTHORITY,
            NON_CLAIM_LIFECYCLE,
            NON_CLAIM_TRANSPORT,
            NON_CLAIM_HOST,
        ],
    })
}

fn checkout_file_receipts(plan: &ChangeCobCheckoutPlan) -> Vec<Value> {
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

fn checkout_plan_hash(
    inspection: &CobChangeInspection,
    policy_name: &str,
    observed_policy_hash: &str,
    plan: &ChangeCobCheckoutPlan,
    blockers: &[String],
) -> String {
    canonical_json_hash(&json!({
        "type_name": RADICLE_CHANGE_COB_TYPE_NAME,
        "workspace_id": inspection.state.repository_id,
        "object_id": inspection.object_id,
        "creator": inspection.state.creator,
        "delegates": inspection.state.delegates,
        "policy": policy_name,
        "object_policy_hash": plan.policy_hash,
        "observed_policy_hash": observed_policy_hash,
        "state_hash": plan.state_hash,
        "content_root": plan.content_root,
        "content_conflicts": plan.content_conflicts,
        "files": checkout_file_receipts(plan),
        "destination": plan.destination.as_str(),
        "blockers": blockers,
    }))
}

fn lifecycle_value(view: &ChangeLifecycleView) -> Value {
    match view {
        ChangeLifecycleView::Current(record) => json!({
            "condition": "current",
            "record": record,
        }),
        ChangeLifecycleView::Stale {
            record,
            current_content_root,
        } => json!({
            "condition": "stale",
            "record": record,
            "current_content_root": current_content_root,
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

pub(crate) fn safe_existing_directory(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<bool, io::Error> {
    match repository.symlink_metadata(relative) {
        Ok(metadata) => Ok(!metadata.file_type().is_symlink() && metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(crate) fn path_exists(repository: &CapabilityRoot, relative: &Path) -> Result<bool, io::Error> {
    match repository.symlink_metadata(relative) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs};

    use super::*;
    use cairn_core::{
        initialize_change_cob, ChangeArtifactId, ChangeCobAction, ChangeCobInit,
        ChangeCobOperation, RADICLE_CHANGE_COB_SCHEMA_V1,
    };
    use onix_cairn_radicle_cob::RADICLE_COB_LIBRARY_VERSION;

    const TEST_DIRECTORY_PREFIX: &str = "cairn-radicle-status-checkout-cli";
    const CHANGE_ID: &str = "demo";
    const REPOSITORY_ID: &str = "workspace:example";
    const BASE_GIT_OID: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const OBJECT_ID: &str = "dddddddddddddddddddddddddddddddddddddddd";
    const ROOT_OPERATION: &str = "1111111111111111111111111111111111111111";
    const ACTOR: &str = "z6Mkactor";
    const PROPOSAL: &str = "# Proposal\n\nExact.  \n";

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    fn remove_test_directory(path: &Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn state() -> cairn_core::ChangeCobState {
        initialize_change_cob(ChangeCobOperation {
            id: ROOT_OPERATION.to_string(),
            actor: ACTOR.to_string(),
            is_delegate: true,
            actions: vec![ChangeCobAction::Init {
                state: ChangeCobInit {
                    schema: RADICLE_CHANGE_COB_SCHEMA_V1.to_string(),
                    change_id: CHANGE_ID.to_string(),
                    repository_id: REPOSITORY_ID.to_string(),
                    base_git_oid: BASE_GIT_OID.to_string(),
                    policy_hash: POLICY_HASH.to_string(),
                    delegates: Default::default(),
                    artifacts: BTreeMap::from([(
                        ChangeArtifactId::proposal(),
                        PROPOSAL.to_string(),
                    )]),
                },
            }],
        })
        .expect("valid test state")
    }

    fn inspection() -> CobChangeInspection {
        CobChangeInspection {
            repository_head_oid: BASE_GIT_OID.to_string(),
            object_id: OBJECT_ID.to_string(),
            state: state(),
        }
    }

    fn options(root: &Path, execute: bool) -> CobObjectOptions {
        CobObjectOptions {
            change: CHANGE_ID.to_string(),
            root: root.to_path_buf(),
            git_repository: PathBuf::from("/explicit/git/repository"),
            policy: PathBuf::from(default_policy_path()),
            execute,
        }
    }

    // r[verify cairn.radicle_cob_only.cli.explicit]
    // r[verify cairn.radicle_cob_only.cli.reject_runtime]
    #[test]
    fn explicit_status_and_checkout_arguments_default_to_read_only() {
        let status = parse_object_options(
            &[
                CHANGE_ID.to_string(),
                "--git-repository".to_string(),
                "/tmp/repository".to_string(),
            ],
            "status",
            false,
        )
        .expect("status options");
        assert!(!status.execute);
        let checkout = parse_object_options(
            &[
                CHANGE_ID.to_string(),
                "--git-repository".to_string(),
                "/tmp/repository".to_string(),
            ],
            "checkout",
            true,
        )
        .expect("checkout options");
        assert!(!checkout.execute);
        assert!(parse_object_options(&[CHANGE_ID.to_string()], "status", false).is_err());
        assert!(parse_object_options(
            &[
                CHANGE_ID.to_string(),
                "--git-repository".to_string(),
                "/tmp/repository".to_string(),
                "--execute".to_string(),
            ],
            "status",
            false,
        )
        .is_err());
        assert!(parse_object_options(
            &[
                CHANGE_ID.to_string(),
                "--rad-home".to_string(),
                "/tmp/radicle".to_string(),
            ],
            "status",
            false,
        )
        .is_err());
    }

    // r[verify cairn.radicle_cob_only.store.negative]
    #[test]
    fn local_scan_preserves_exact_bytes_and_reports_unsupported_entries(
    ) -> Result<(), Box<dyn Error>> {
        let root = test_directory("scan");
        remove_test_directory(&root)?;
        let change = root.join(CHANGES_DIRECTORY).join(CHANGE_ID);
        fs::create_dir_all(&change)?;
        fs::write(change.join("proposal.md"), PROPOSAL)?;

        let exact = read_local_change_snapshot(&root, CHANGE_ID)?;
        assert_eq!(
            exact.files.get("proposal.md").map(String::as_str),
            Some(PROPOSAL)
        );
        assert!(exact.issues.is_empty());

        fs::write(change.join("notes.txt"), "unsupported\n")?;
        let unsupported = read_local_change_snapshot(&root, CHANGE_ID)?;
        assert!(unsupported
            .issues
            .iter()
            .any(|issue| issue.contains("unsupported local entry")));

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            fs::remove_file(change.join("proposal.md"))?;
            let outside = root.join("outside.md");
            fs::write(&outside, "outside\n")?;
            symlink(&outside, change.join("proposal.md"))?;
            let symlinked = read_local_change_snapshot(&root, CHANGE_ID)?;
            assert!(symlinked
                .issues
                .iter()
                .any(|issue| issue.contains("not a safe regular file")));
        }

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // r[verify cairn.radicle_cob_only.cli.preview]
    #[test]
    fn checkout_materializes_exact_bytes_and_refuses_existing_destination(
    ) -> Result<(), Box<dyn Error>> {
        let root = test_directory("execute");
        remove_test_directory(&root)?;
        fs::create_dir_all(root.join(CHANGES_DIRECTORY))?;
        let state = state();
        let plan = plan_change_cob_checkout(&state, ChangeCobLocalSnapshot::absent())?;

        let outcome = materialize_checkout(&root, REPOSITORY_ID, &state, &plan)?;
        assert!(outcome.mutated);
        assert!(outcome.blockers.is_empty());
        assert_eq!(
            fs::read_to_string(
                root.join(CHANGES_DIRECTORY)
                    .join(CHANGE_ID)
                    .join("proposal.md")
            )?,
            PROPOSAL
        );
        let post = read_local_change_snapshot(&root, CHANGE_ID)?;
        assert!(plan_change_cob_checkout(&state, post)?.already_current());

        let blocked = materialize_checkout(&root, REPOSITORY_ID, &state, &plan)?;
        assert!(!blocked.mutated);
        assert!(blocked
            .blockers
            .iter()
            .any(|blocker| blocker == BLOCKER_DESTINATION_APPEARED));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // r[verify cairn.radicle_cob_only.docs.non_claims]
    #[test]
    fn receipts_bind_plan_without_disclosing_repository_path() -> Result<(), Box<dyn Error>> {
        let root = PathBuf::from(".");
        let inspection = inspection();
        let plan = plan_change_cob_checkout(&inspection.state, ChangeCobLocalSnapshot::absent())?;
        let receipt = with_portable_receipt_hash(
            &root,
            status_receipt(
                &options(&root, false),
                &inspection,
                "cairn-default",
                POLICY_HASH,
                &plan,
            ),
        );
        let rendered = serde_json::to_string(&receipt)?;
        assert!(!rendered.contains("/explicit/git/repository"));
        assert_eq!(receipt.get("mutated").and_then(Value::as_bool), Some(false));
        assert_eq!(
            receipt
                .get("format_libraries")
                .and_then(|libraries| libraries.get("radicle-cob"))
                .and_then(Value::as_str),
            Some(RADICLE_COB_LIBRARY_VERSION)
        );
        assert_eq!(
            receipt
                .get("receipt_hash")
                .and_then(Value::as_str)
                .map(str::len),
            Some(cairn_core::BLAKE3_HEX_LENGTH)
        );
        assert!(receipt
            .get("non_claims")
            .and_then(Value::as_array)
            .is_some_and(|claims| claims.iter().any(|claim| claim == NON_CLAIM_TRANSPORT)));

        let blocked = checkout_receipt(
            &options(&root, true),
            &inspection,
            "cairn-default",
            POLICY_HASH,
            &plan,
            &CheckoutExecutionOutcome {
                mutated: false,
                final_destination: None,
                blockers: vec![BLOCKER_INPUT_DRIFT.to_string()],
            },
        );
        assert_eq!(blocked.get("blocked").and_then(Value::as_bool), Some(true));
        assert_eq!(blocked.get("mutated").and_then(Value::as_bool), Some(false));

        let executed = checkout_receipt(
            &options(&root, true),
            &inspection,
            "cairn-default",
            POLICY_HASH,
            &plan,
            &CheckoutExecutionOutcome {
                mutated: true,
                final_destination: Some(ChangeCheckoutDestination::Current),
                blockers: Vec::new(),
            },
        );
        assert_eq!(executed.get("valid").and_then(Value::as_bool), Some(true));
        assert_eq!(executed.get("mutated").and_then(Value::as_bool), Some(true));
        assert_eq!(
            executed
                .get("post_checkout_destination")
                .and_then(Value::as_str),
            Some("current")
        );
        Ok(())
    }
}
