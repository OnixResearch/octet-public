pub(crate) mod resolution;
pub(crate) mod status_checkout;
pub(crate) mod update;

use std::{
    collections::BTreeMap,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, change_cob_import_content_root, plan_change_cob_import,
    validate_change_name, ChangeCobError, ChangeCobImportSnapshot, ChangeCobInit, MAX_ARTIFACTS,
    MAX_ARTIFACT_BYTES, RADICLE_CHANGE_COB_TYPE_NAME,
};
use onix_cairn_radicle_cob::{
    create_change, find_change, parse_delegate_ids, signer_actor, CobMutationLock,
    CobMutationOutcome, CobRepository, GIT2_LIBRARY_VERSION, GIT_REF_FORMAT_LIBRARY_VERSION,
    RADICLE_COB_LIBRARY_VERSION, RADICLE_CRYPTO_LIBRARY_VERSION, RADICLE_OID_LIBRARY_VERSION,
};
use serde_json::{json, Value};

use crate::{
    collaboration::materialize::RepositoryCheckoutLock,
    support::{
        default_policy_path, load_policy_file, path_string, print_json, with_portable_receipt_hash,
        CapabilityRoot, CapabilityRootKind,
    },
};

pub(super) const COB_BACKEND_V1: &str = "radicle-cob.v1";
pub(super) const OPTION_TOKEN_WIDTH: usize = 2;

pub(super) fn format_library_versions() -> Value {
    json!({
        "radicle-cob": RADICLE_COB_LIBRARY_VERSION,
        "radicle-crypto": RADICLE_CRYPTO_LIBRARY_VERSION,
        "radicle-oid": RADICLE_OID_LIBRARY_VERSION,
        "git2": GIT2_LIBRARY_VERSION,
        "git-ref-format-core": GIT_REF_FORMAT_LIBRARY_VERSION,
    })
}
const IMPORT_NON_CLAIM_LOCAL: &str =
    "signed COB creation proves bounded local Git object and reference observations only";
const IMPORT_NON_CLAIM_LIFECYCLE: &str =
    "signed import does not prove gate validity, evidence truth, dependency readiness, archive execution, or release eligibility";
const IMPORT_NON_CLAIM_AUTHORITY: &str =
    "embedded creator and delegate keys are immutable object authority, not Radicle identity-document or future authorization proof";
const IMPORT_NON_CLAIM_TRANSPORT: &str =
    "local Git COB creation does not prove node participation, publication, synchronization, peer availability, convergence, or remote trust";
const BLOCKER_INPUT_DRIFT: &str =
    "active change, policy, repository head, signer, embedded authority, or existing COB state changed after preview";

// r[impl cairn.radicle_cob_only.cli]
// r[impl cairn.radicle_cob_only.no_runtime]
// r[impl cairn.radicle_cob_only.docs]

#[derive(Debug, Clone, PartialEq, Eq)]
struct CobImportOptions {
    change: String,
    root: PathBuf,
    git_repository: PathBuf,
    keystore: PathBuf,
    workspace_id: String,
    delegates: Vec<String>,
    policy: PathBuf,
    execute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImportInspection {
    actor: String,
    repository_head_oid: String,
    existing_object_id: Option<String>,
}

pub(crate) fn run_cob(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first() else {
        return Err(
            "cob requires a subcommand: import, status, checkout, update, conflicts, or resolve"
                .into(),
        );
    };
    match subcommand.as_str() {
        "import" => run_cob_import(args.into_iter().skip(1).collect()),
        "status" => status_checkout::run_status(args.into_iter().skip(1).collect()),
        "checkout" => status_checkout::run_checkout(args.into_iter().skip(1).collect()),
        "update" => update::run_update(args.into_iter().skip(1).collect()),
        "conflicts" => resolution::run_conflicts(args.into_iter().skip(1).collect()),
        "resolve" => resolution::run_resolve(args.into_iter().skip(1).collect()),
        // r[impl cairn.collaboration_sync.retired]
        // r[verify cairn.collaboration_sync.retired.rejected]
        // r[verify cairn.radicle_cob_only.no_runtime.sync]
        "sync" => Err(
            "COB-only collaboration does not provide synchronization; manage Git transport outside Cairn"
                .into(),
        ),
        unknown => Err(format!("unknown cob subcommand: {unknown}").into()),
    }
}

fn run_cob_import(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_import_options(&args)?;
    let inspection = inspect_import(&options)?;
    let policy = load_policy_file(options.policy.clone())?;
    let policy_hash = canonical_json_hash(&policy.raw);
    let init = build_import_init(&options, &inspection, &policy_hash)?;
    let content_root = change_cob_import_content_root(&init, &inspection.actor)?;
    let mut blockers = existing_import_blockers(&inspection);
    let mut outcome = None;

    if options.execute && blockers.is_empty() {
        match RepositoryCheckoutLock::acquire(&options.root, &options.change) {
            Ok(repository_guard) => {
                match CobMutationLock::acquire(&options.git_repository, &options.change) {
                    Ok(cob_guard) => {
                        let execution = recheck_import(&options, &inspection, &init, &content_root)
                            .and_then(|current| {
                                create_change(
                                    &options.git_repository,
                                    &options.keystore,
                                    current,
                                    &content_root,
                                )
                                .map_err(Into::into)
                            });
                        let cob_release = cob_guard.release();
                        match execution {
                            Ok(created) => outcome = Some(created),
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

    let receipt = import_receipt(
        &options,
        &inspection,
        &policy.name,
        &init,
        &content_root,
        &blockers,
        outcome.as_ref(),
    )?;
    print_json(with_portable_receipt_hash(&options.root, receipt))
}

fn inspect_import(options: &CobImportOptions) -> Result<ImportInspection, Box<dyn Error>> {
    let repository = CobRepository::open(&options.git_repository)?;
    Ok(ImportInspection {
        actor: signer_actor(&options.keystore)?,
        repository_head_oid: repository.head_oid()?.to_string(),
        existing_object_id: find_change(&options.git_repository, &options.change)?
            .map(|inspection| inspection.object_id),
    })
}

fn build_import_init(
    options: &CobImportOptions,
    inspection: &ImportInspection,
    policy_hash: &str,
) -> Result<ChangeCobInit, Box<dyn Error>> {
    let snapshot = read_active_change_snapshot(
        &options.root,
        &options.change,
        &options.workspace_id,
        &inspection.repository_head_oid,
        policy_hash,
    )?;
    let mut init = plan_change_cob_import(snapshot)?;
    init.delegates = parse_delegate_ids(&inspection.actor, &options.delegates)?;
    cairn_core::validate_change_cob_init(&init)?;
    Ok(init)
}

fn recheck_import(
    options: &CobImportOptions,
    expected_inspection: &ImportInspection,
    expected_init: &ChangeCobInit,
    expected_content_root: &str,
) -> Result<ChangeCobInit, Box<dyn Error>> {
    let policy = load_policy_file(options.policy.clone())?;
    let current_inspection = inspect_import(options)?;
    let current_init = build_import_init(
        options,
        &current_inspection,
        &canonical_json_hash(&policy.raw),
    )?;
    let current_content_root =
        change_cob_import_content_root(&current_init, &current_inspection.actor)?;
    if &current_inspection != expected_inspection
        || &current_init != expected_init
        || current_content_root != expected_content_root
        || current_inspection.existing_object_id.is_some()
    {
        return Err(BLOCKER_INPUT_DRIFT.into());
    }
    Ok(current_init)
}

fn parse_import_options(args: &[String]) -> Result<CobImportOptions, Box<dyn Error>> {
    let Some(change) = args.first() else {
        return Err("cob import requires a change name".into());
    };
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let mut root = None;
    let mut git_repository = None;
    let mut keystore = None;
    let mut workspace_id = None;
    let mut delegates = Vec::new();
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
            "--workspace" => {
                workspace_id = Some(read_unique_option_value(
                    args,
                    index,
                    "--workspace",
                    workspace_id.is_some(),
                )?);
                index = index.saturating_add(OPTION_TOKEN_WIDTH);
            }
            "--delegate" => {
                delegates.push(read_unique_option_value(args, index, "--delegate", false)?);
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
                    "unsupported Radicle runtime argument for COB-only import: {}",
                    args[index]
                )
                .into());
            }
            unknown => return Err(format!("unknown cob import argument: {unknown}").into()),
        }
    }
    delegates.sort();
    delegates.dedup();
    Ok(CobImportOptions {
        change: change.clone(),
        root: root.unwrap_or_else(|| PathBuf::from(".")),
        git_repository: git_repository.ok_or("cob import requires --git-repository PATH")?,
        keystore: keystore.ok_or("cob import requires --keystore PATH")?,
        workspace_id: workspace_id.ok_or("cob import requires --workspace ID")?,
        delegates,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        execute,
    })
}

pub(super) fn read_unique_option_value(
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

fn read_active_change_snapshot(
    root: &Path,
    change: &str,
    workspace_id: &str,
    base_git_oid: &str,
    policy_hash: &str,
) -> Result<ChangeCobImportSnapshot, Box<dyn Error>> {
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let capability = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let change_dir = PathBuf::from(".cairn/changes").join(change);
    let change_metadata = capability.symlink_metadata(&change_dir).map_err(|error| {
        format!(
            "active native change {} is unavailable: {error}",
            change_dir.display()
        )
    })?;
    if change_metadata.file_type().is_symlink() || !change_metadata.is_dir() {
        return Err(format!(
            "active native change path is not a safe directory: {}",
            change_dir.display()
        )
        .into());
    }
    let proposal = read_optional_import_artifact(&capability, &change_dir.join("proposal.md"))?;
    let design = read_optional_import_artifact(&capability, &change_dir.join("design.md"))?;
    let tasks = read_optional_import_artifact(&capability, &change_dir.join("tasks.md"))?;
    let metadata = read_optional_import_artifact(&capability, &change_dir.join("metadata.json"))?;
    let delta_specs = read_delta_specs(&capability, &change_dir.join("specs"))?;
    Ok(ChangeCobImportSnapshot {
        change_id: change.to_string(),
        repository_id: workspace_id.to_string(),
        base_git_oid: base_git_oid.to_string(),
        policy_hash: policy_hash.to_string(),
        proposal,
        design,
        tasks,
        metadata,
        delta_specs,
    })
}

fn read_optional_import_artifact(
    capability: &CapabilityRoot,
    relative: &Path,
) -> Result<Option<String>, Box<dyn Error>> {
    let metadata = match capability.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "import artifact is not a safe regular file: {}",
            relative.display()
        )
        .into());
    }
    let max_bytes = u64::try_from(MAX_ARTIFACT_BYTES)?;
    let bytes = capability.read_bounded(relative, max_bytes)?;
    Ok(Some(String::from_utf8(bytes).map_err(|error| {
        format!(
            "import artifact is not UTF-8 at {}: {error}",
            relative.display()
        )
    })?))
}

fn read_delta_specs(
    capability: &CapabilityRoot,
    specs_dir: &Path,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let metadata = match capability.symlink_metadata(specs_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "change specs path is not a safe directory: {}",
            specs_dir.display()
        )
        .into());
    }
    let mut entries = capability.entries_bounded(specs_dir, MAX_ARTIFACTS)?;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    let mut delta_specs = BTreeMap::new();
    for entry in entries {
        if delta_specs.len() >= MAX_ARTIFACTS {
            return Err(format!(
                "change specs exceed the import artifact limit of {MAX_ARTIFACTS}"
            )
            .into());
        }
        if !entry.file_type().is_dir() {
            return Err(format!(
                "unsupported entry in change specs directory: {}",
                specs_dir.join(entry.file_name()).display()
            )
            .into());
        }
        let spec_id = entry
            .file_name()
            .to_str()
            .ok_or("delta-spec directory name is not UTF-8")?
            .to_string();
        let spec_path = specs_dir.join(entry.file_name()).join("spec.md");
        let content = read_optional_import_artifact(capability, &spec_path)?
            .ok_or_else(|| format!("delta-spec file is missing: {}", spec_path.display()))?;
        delta_specs.insert(spec_id, content);
    }
    Ok(delta_specs)
}

fn existing_import_blockers(inspection: &ImportInspection) -> Vec<String> {
    inspection
        .existing_object_id
        .as_ref()
        .map(|object_id| vec![format!("change already has COB object {object_id}")])
        .unwrap_or_default()
}

fn import_receipt(
    options: &CobImportOptions,
    inspection: &ImportInspection,
    policy_name: &str,
    init: &ChangeCobInit,
    content_root: &str,
    blockers: &[String],
    outcome: Option<&CobMutationOutcome>,
) -> Result<Value, Box<dyn Error>> {
    let artifacts = init
        .artifacts
        .iter()
        .map(|(artifact, content)| {
            Ok(json!({
                "id": artifact.as_str(),
                "path": artifact.relative_path()?,
                "blake3": cairn_core::blake3_hex(content.as_bytes()),
                "bytes": content.len(),
            }))
        })
        .collect::<Result<Vec<_>, ChangeCobError>>()?;
    let input_hash = canonical_json_hash(&serde_json::to_value(init)?);
    let plan_hash = canonical_json_hash(&json!({
        "backend": COB_BACKEND_V1,
        "type_name": RADICLE_CHANGE_COB_TYPE_NAME,
        "change": options.change,
        "workspace_id": init.repository_id,
        "actor": inspection.actor,
        "delegates": init.delegates,
        "base_git_oid": inspection.repository_head_oid,
        "policy": policy_name,
        "policy_hash": init.policy_hash,
        "input_hash": input_hash,
        "content_root": content_root,
    }));
    let blocked = !blockers.is_empty();
    Ok(json!({
        "command": "cob import",
        "collaboration_command": "collaboration import",
        "backend": COB_BACKEND_V1,
        "format_libraries": format_library_versions(),
        "root": path_string(&options.root),
        "change": options.change,
        "type_name": RADICLE_CHANGE_COB_TYPE_NAME,
        "workspace_id": init.repository_id,
        "actor": inspection.actor,
        "delegates": init.delegates,
        "authority_source": "signed_root_embedded_authority",
        "repository_head_oid": inspection.repository_head_oid,
        "policy": policy_name,
        "policy_hash": init.policy_hash,
        "artifacts": artifacts,
        "artifact_count": init.artifacts.len(),
        "input_hash": input_hash,
        "plan_hash": plan_hash,
        "content_root": content_root,
        "dry_run": !options.execute,
        "blocked": blocked,
        "blockers": blockers,
        "mutated": outcome.is_some(),
        "object_id": outcome.map(|created| created.object_id.as_str()),
        "operation_id": outcome.map(|created| created.operation_id.as_str()),
        "state_hash": outcome.map(|created| created.state_hash.as_str()),
        "valid": !blocked,
        "verdict": if blocked { "FAIL" } else { "PASS" },
        "non_claims": [
            IMPORT_NON_CLAIM_LOCAL,
            IMPORT_NON_CLAIM_LIFECYCLE,
            IMPORT_NON_CLAIM_AUTHORITY,
            IMPORT_NON_CLAIM_TRANSPORT,
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHANGE_ID: &str = "cob-import-cli";
    const PUBLIC_KEY: &str = "z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi";

    // r[verify cairn.radicle_cob_only.cli.explicit]
    // r[verify cairn.radicle_cob_only.cli.reject_runtime]
    // r[verify cairn.radicle_cob_only.no_runtime.artifacts]
    #[test]
    fn import_arguments_require_only_explicit_cob_context() -> Result<(), Box<dyn Error>> {
        let parsed = parse_import_options(&[
            CHANGE_ID.to_string(),
            "--git-repository".to_string(),
            "/tmp/repository".to_string(),
            "--keystore".to_string(),
            "/tmp/keys".to_string(),
            "--workspace".to_string(),
            "workspace:example".to_string(),
            "--delegate".to_string(),
            PUBLIC_KEY.to_string(),
        ])?;
        assert_eq!(parsed.change, CHANGE_ID);
        assert!(!parsed.execute);
        assert!(parse_import_options(&[CHANGE_ID.to_string()]).is_err());
        assert!(parse_import_options(&[
            CHANGE_ID.to_string(),
            "--rad-home".to_string(),
            "/tmp/radicle".to_string(),
        ])
        .is_err());
        assert!(run_cob(vec!["sync".to_string(), CHANGE_ID.to_string()]).is_err());
        Ok(())
    }
}
