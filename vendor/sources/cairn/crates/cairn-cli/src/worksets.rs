use std::{
    env,
    error::Error,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use cairn_core::{
    apply_workset_transition, canonical_json_hash, plan_workset_create, plan_workset_open,
    plan_workset_remove, recheck_workset_open_plan, workset_identity, workset_portable_identity,
    workset_portable_state_identity, workset_state_identity, Workset, WorksetCreateRequest,
    WorksetError, WorksetExecutionMode, WorksetId, WorksetMember, WorksetOpenPlan,
    WorksetOpenRequest, WorksetOpenerPolicy, WorksetOpenerProfile, WorksetOpenerProfileId,
    WorksetRemoveRequest, WorksetState, WorksetTransitionPlan,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::support::{
    default_policy_path, load_policy_file, print_json, CapabilityRoot, CapabilityRootKind,
};

const CAIRN_DATA_DIR_ENV: &str = "CAIRN_DATA_DIR";
const XDG_DATA_HOME_ENV: &str = "XDG_DATA_HOME";
const HOME_ENV: &str = "HOME";
const CAIRN_DATA_DIRECTORY_NAME: &str = "cairn";
const HOME_DATA_DIRECTORY: &str = ".local/share/cairn";
const WORKSET_DIRECTORY: &str = "worksets";
const WORKSET_STATE_FILE: &str = "state.json";
const WORKSET_STATE_LOCK_FILE: &str = ".state.json.lock";
const WORKSET_STATE_TEMP_PREFIX: &str = ".state.json";
const WORKSET_STATE_TEMP_SUFFIX: &str = "tmp";
const POSITIONAL_MEMBER_MODE: &str = "positional_paths";
const PRIMARY_MEMBER_LABEL: &str = "primary";
const MEMBER_LABEL_PREFIX: &str = "member";
const WORKSET_COMMAND_SCHEMA: &str = "cairn.workset-command.v1";
const WORKSET_LIST_SCHEMA: &str = "cairn.workset-list.v1";
const WORKSET_SHOW_SCHEMA: &str = "cairn.workset-show.v1";
const PROCESS_OBSERVATION_SCHEMA: &str = "cairn.workset-process-observation.v1";
const MAX_WORKSET_STATE_BYTES: u64 = 32 * 1024 * 1024;
const OPTION_VALUE_WIDTH: usize = 2;

// r[impl cairn.lifecycle_stores.worksets]
// r[impl cairn.lifecycle_stores.functional_boundary.shell]
// r[impl cairn.lifecycle_stores.transport_boundary]
pub(crate) fn run_workset(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().cloned() else {
        return Err("workset requires a subcommand: create, list, show, remove, or open".into());
    };
    let remaining = args.into_iter().skip(1).collect::<Vec<_>>();
    match subcommand.as_str() {
        "create" => run_create(remaining),
        "list" => run_list(remaining),
        "show" => run_show(remaining),
        "remove" => run_remove(remaining),
        "open" => run_open(remaining),
        unknown => Err(format!("unknown workset subcommand: {unknown}").into()),
    }
}

fn run_create(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (workset_id, tail) = required_id(args, "workset create")?;
    let options = parse_options(
        &tail,
        OptionRules {
            members: true,
            opener: true,
            execute: true,
        },
    )?;
    if options.members.is_empty() {
        return Err("workset create requires at least one --member PATH".into());
    }
    let opener = options
        .opener
        .as_deref()
        .ok_or("workset create requires --opener ID")?;
    let policy = load_shell_policy(&options.policy_path)?;
    let paths = StatePaths::discover()?;
    let state = load_state(&paths, &policy)?;
    let members = normalize_members(&options.members)?;
    let workset = core_result(Workset::new(
        workset_id,
        members,
        PRIMARY_MEMBER_LABEL,
        opener,
    ))?;
    let execution = execution_mode(options.execute);
    let plan = core_result(plan_workset_create(
        &state,
        &policy.core,
        WorksetCreateRequest {
            workset,
            expected_prior_state_identity: core_result(workset_state_identity(&state))?,
            expected_prior_portable_state_identity: core_result(workset_portable_state_identity(
                &state,
            ))?,
            execution,
        },
    ))?;
    if options.execute {
        execute_transition(&paths, &options.policy_path, &plan)?;
    }
    print_json(command_receipt(
        "workset create",
        &paths,
        options.execute,
        options.execute,
        serde_json::to_value(plan)?,
        None,
    ))
}

fn run_list(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_options(&args, OptionRules::read_only())?;
    let policy = load_shell_policy(&options.policy_path)?;
    let paths = StatePaths::discover()?;
    let state = load_state(&paths, &policy)?;
    let worksets = state
        .worksets
        .iter()
        .map(|workset| {
            json!({
                "id": workset.id,
                "member_count": workset.members.len(),
                "primary_member": workset.primary_member,
                "opener_profile": workset.opener_profile,
            })
        })
        .collect::<Vec<_>>();
    print_json(json!({
        "schema": WORKSET_LIST_SCHEMA,
        "command": "workset list",
        "read_only": true,
        "state_path": paths.state_path(),
        "state_identity": core_result(workset_state_identity(&state))?,
        "portable_state_identity": core_result(workset_portable_state_identity(&state))?,
        "worksets": worksets,
    }))
}

fn run_show(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (workset_id, tail) = required_id(args, "workset show")?;
    let options = parse_options(&tail, OptionRules::read_only())?;
    let policy = load_shell_policy(&options.policy_path)?;
    let paths = StatePaths::discover()?;
    let state = load_state(&paths, &policy)?;
    let id = core_result(WorksetId::parse(workset_id))?;
    let workset = state
        .worksets
        .iter()
        .find(|workset| workset.id == id)
        .ok_or_else(|| {
            format!(
                "lifecycle_store.workset.invalid: unknown workset: {}",
                id.as_str()
            )
        })?;
    print_json(json!({
        "schema": WORKSET_SHOW_SCHEMA,
        "command": "workset show",
        "read_only": true,
        "state_path": paths.state_path(),
        "state_identity": core_result(workset_state_identity(&state))?,
        "portable_state_identity": core_result(workset_portable_state_identity(&state))?,
        "workset_identity": core_result(workset_identity(workset))?,
        "portable_workset_identity": core_result(workset_portable_identity(workset))?,
        "workset": workset,
    }))
}

fn run_remove(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (workset_id, tail) = required_id(args, "workset remove")?;
    let options = parse_options(
        &tail,
        OptionRules {
            members: false,
            opener: false,
            execute: true,
        },
    )?;
    let policy = load_shell_policy(&options.policy_path)?;
    let paths = StatePaths::discover()?;
    let state = load_state(&paths, &policy)?;
    let plan = core_result(plan_workset_remove(
        &state,
        &policy.core,
        WorksetRemoveRequest {
            workset_id: core_result(WorksetId::parse(workset_id))?,
            expected_prior_state_identity: core_result(workset_state_identity(&state))?,
            expected_prior_portable_state_identity: core_result(workset_portable_state_identity(
                &state,
            ))?,
            execution: execution_mode(options.execute),
        },
    ))?;
    if options.execute {
        execute_transition(&paths, &options.policy_path, &plan)?;
    }
    print_json(command_receipt(
        "workset remove",
        &paths,
        options.execute,
        options.execute,
        serde_json::to_value(plan)?,
        None,
    ))
}

fn run_open(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (workset_id, tail) = required_id(args, "workset open")?;
    let options = parse_options(
        &tail,
        OptionRules {
            members: false,
            opener: true,
            execute: true,
        },
    )?;
    let policy = load_shell_policy(&options.policy_path)?;
    let paths = StatePaths::discover()?;
    let state = load_state(&paths, &policy)?;
    let id = core_result(WorksetId::parse(workset_id))?;
    let workset = require_workset(&state, &id)?;
    recheck_member_paths(workset)?;
    let opener_override = options
        .opener
        .map(WorksetOpenerProfileId::parse)
        .transpose()
        .map_err(boxed_core_error)?;
    let plan = core_result(plan_workset_open(
        &state,
        &policy.core,
        WorksetOpenRequest {
            workset_id: id,
            opener_override,
            expected_state_identity: core_result(workset_state_identity(&state))?,
            expected_portable_state_identity: core_result(workset_portable_state_identity(&state))?,
            execution: execution_mode(options.execute),
        },
    ))?;
    let observation = if options.execute {
        Some(execute_open(&paths, &options.policy_path, &plan)?)
    } else {
        None
    };
    print_json(command_receipt(
        "workset open",
        &paths,
        options.execute,
        false,
        serde_json::to_value(plan)?,
        observation.map(serde_json::to_value).transpose()?,
    ))
}

fn required_id(args: Vec<String>, command: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let Some(id) = args.first() else {
        return Err(format!("{command} requires an ID").into());
    };
    if id.starts_with("--") {
        return Err(format!("{command} requires an ID before options").into());
    }
    Ok((id.clone(), args.into_iter().skip(1).collect()))
}

#[derive(Clone, Copy)]
struct OptionRules {
    members: bool,
    opener: bool,
    execute: bool,
}

impl OptionRules {
    fn read_only() -> Self {
        Self {
            members: false,
            opener: false,
            execute: false,
        }
    }
}

struct CommandOptions {
    members: Vec<PathBuf>,
    opener: Option<String>,
    policy_path: PathBuf,
    execute: bool,
}

fn parse_options(args: &[String], rules: OptionRules) -> Result<CommandOptions, Box<dyn Error>> {
    let mut members = Vec::new();
    let mut opener = None;
    let mut policy_path = None;
    let mut execute = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--member" if rules.members => {
                members.push(PathBuf::from(option_value(args, index, "--member")?));
                index += OPTION_VALUE_WIDTH;
            }
            "--opener" if rules.opener => {
                if opener.is_some() {
                    return Err("--opener can occur only once".into());
                }
                opener = Some(option_value(args, index, "--opener")?.to_string());
                index += OPTION_VALUE_WIDTH;
            }
            "--policy" => {
                if policy_path.is_some() {
                    return Err("--policy can occur only once".into());
                }
                policy_path = Some(PathBuf::from(option_value(args, index, "--policy")?));
                index += OPTION_VALUE_WIDTH;
            }
            "--execute" if rules.execute => {
                if execute {
                    return Err("--execute can occur only once".into());
                }
                execute = true;
                index += 1;
            }
            unknown => {
                return Err(format!("unknown or unsupported workset argument: {unknown}").into())
            }
        }
    }
    Ok(CommandOptions {
        members,
        opener,
        policy_path: policy_path.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        execute,
    })
}

fn option_value<'a>(
    args: &'a [String],
    index: usize,
    option: &str,
) -> Result<&'a str, Box<dyn Error>> {
    args.get(index + 1)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} requires a value").into())
}

fn execution_mode(execute: bool) -> WorksetExecutionMode {
    if execute {
        WorksetExecutionMode::Execute
    } else {
        WorksetExecutionMode::Preview
    }
}

fn normalize_members(paths: &[PathBuf]) -> Result<Vec<WorksetMember>, Box<dyn Error>> {
    paths
        .iter()
        .enumerate()
        .map(|(index, path)| {
            let display_path = canonical_member_path(path)?;
            let label = if index == 0 {
                PRIMARY_MEMBER_LABEL.to_string()
            } else {
                format!("{MEMBER_LABEL_PREFIX}-{}", index + 1)
            };
            core_result(WorksetMember::new(label, display_path))
        })
        .collect()
}

fn canonical_member_path(path: &Path) -> Result<String, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!(
            "lifecycle_store.workset.changed_member: member path {} is not accessible: {error}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "lifecycle_store.workset.changed_member: member path must be a non-symlink directory: {}",
            path.display()
        )
        .into());
    }
    let canonical = fs::canonicalize(path)?;
    canonical
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| "lifecycle_store.workset.invalid: member path must be UTF-8".into())
}

fn recheck_member_paths(workset: &Workset) -> Result<(), Box<dyn Error>> {
    for member in &workset.members {
        let observed = canonical_member_path(Path::new(&member.display_path))?;
        if observed != member.display_path {
            return Err(format!(
                "lifecycle_store.workset.changed_member: member path changed for {}",
                member.label.as_str()
            )
            .into());
        }
    }
    Ok(())
}

fn require_workset<'a>(
    state: &'a WorksetState,
    id: &WorksetId,
) -> Result<&'a Workset, Box<dyn Error>> {
    state
        .worksets
        .iter()
        .find(|workset| workset.id == *id)
        .ok_or_else(|| {
            format!(
                "lifecycle_store.workset.invalid: unknown workset: {}",
                id.as_str()
            )
            .into()
        })
}

#[derive(Deserialize)]
struct PolicyProfile {
    id: String,
    executable: String,
    fixed_args: Vec<String>,
    member_argument_mode: String,
    primary_first: bool,
}

struct ShellPolicy {
    core: WorksetOpenerPolicy,
    state_schema: String,
}

fn load_shell_policy(path: &Path) -> Result<ShellPolicy, Box<dyn Error>> {
    let policy = load_policy_file(path.to_path_buf())?;
    let lifecycle = policy
        .raw
        .get("lifecycle_store_policy")
        .ok_or("lifecycle_store.workset.invalid: policy has no lifecycle_store_policy")?;
    let profiles = lifecycle
        .get("opener_profiles")
        .cloned()
        .ok_or("lifecycle_store.workset.invalid: policy has no opener profiles")?;
    let profiles: Vec<PolicyProfile> = serde_json::from_value(profiles)?;
    if profiles.is_empty() {
        return Err("lifecycle_store.workset.invalid: policy opener profiles are empty".into());
    }
    let mut core_profiles = Vec::with_capacity(profiles.len());
    for profile in profiles {
        if profile.member_argument_mode != POSITIONAL_MEMBER_MODE || !profile.primary_first {
            return Err(format!(
                "lifecycle_store.workset.invalid: unsupported opener profile shape: {}",
                profile.id
            )
            .into());
        }
        core_profiles.push(core_result(WorksetOpenerProfile::new(
            profile.id,
            profile.executable,
            profile.fixed_args,
        ))?);
    }
    let state_schema = lifecycle
        .pointer("/worksets/state/schema")
        .and_then(Value::as_str)
        .ok_or("lifecycle_store.workset.invalid: policy has no workset state schema")?
        .to_string();
    Ok(ShellPolicy {
        core: WorksetOpenerPolicy {
            policy_hash: canonical_json_hash(&policy.raw),
            profiles: core_profiles,
        },
        state_schema,
    })
}

#[derive(Clone, Debug)]
struct StatePaths {
    data_dir: PathBuf,
}

impl StatePaths {
    fn discover() -> Result<Self, Box<dyn Error>> {
        if let Some(path) = env::var_os(CAIRN_DATA_DIR_ENV) {
            if path.is_empty() {
                return Err(format!("{CAIRN_DATA_DIR_ENV} must not be empty").into());
            }
            return Ok(Self {
                data_dir: PathBuf::from(path).join(CAIRN_DATA_DIRECTORY_NAME),
            });
        }
        if let Some(path) = env::var_os(XDG_DATA_HOME_ENV) {
            let path = PathBuf::from(path);
            if !path.is_absolute() {
                return Err(format!("{XDG_DATA_HOME_ENV} must be absolute").into());
            }
            return Ok(Self {
                data_dir: path.join(CAIRN_DATA_DIRECTORY_NAME),
            });
        }
        let home =
            env::var_os(HOME_ENV).ok_or("HOME is not set and no Cairn data override exists")?;
        Ok(Self {
            data_dir: PathBuf::from(home).join(HOME_DATA_DIRECTORY),
        })
    }

    #[cfg(test)]
    fn explicit(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    fn state_relative(&self) -> PathBuf {
        PathBuf::from(WORKSET_DIRECTORY).join(WORKSET_STATE_FILE)
    }

    fn state_path(&self) -> PathBuf {
        self.data_dir.join(self.state_relative())
    }

    fn lock_relative(&self) -> PathBuf {
        PathBuf::from(WORKSET_DIRECTORY).join(WORKSET_STATE_LOCK_FILE)
    }

    fn temporary_relative(&self) -> PathBuf {
        PathBuf::from(WORKSET_DIRECTORY).join(format!(
            "{WORKSET_STATE_TEMP_PREFIX}.{}.{WORKSET_STATE_TEMP_SUFFIX}",
            std::process::id()
        ))
    }
}

#[derive(Deserialize, Serialize)]
struct StoredWorksetState {
    schema: String,
    worksets: Vec<Workset>,
    state_hash: String,
}

fn load_state(paths: &StatePaths, policy: &ShellPolicy) -> Result<WorksetState, Box<dyn Error>> {
    if !paths.data_dir.exists() {
        return Ok(WorksetState::empty());
    }
    let root = CapabilityRoot::open(CapabilityRootKind::LocalState, &paths.data_dir)?;
    load_state_from_root(&root, paths, policy)
}

fn load_state_from_root(
    root: &CapabilityRoot,
    paths: &StatePaths,
    policy: &ShellPolicy,
) -> Result<WorksetState, Box<dyn Error>> {
    let relative = paths.state_relative();
    let metadata = match root.symlink_metadata(&relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(WorksetState::empty()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(
            "lifecycle_store.workset.invalid: workset state must be a regular non-symlink file"
                .into(),
        );
    }
    let bytes = root.read_bounded(&relative, MAX_WORKSET_STATE_BYTES)?;
    let stored: StoredWorksetState = serde_json::from_slice(&bytes)?;
    if stored.schema != policy.state_schema {
        return Err(format!(
            "lifecycle_store.workset.invalid: unsupported stored workset schema: {}",
            stored.schema
        )
        .into());
    }
    let state = WorksetState {
        schema_version: cairn_core::WORKSET_STATE_SCHEMA_V1.to_string(),
        worksets: stored.worksets,
    };
    let observed_hash = core_result(workset_state_identity(&state))?;
    if observed_hash != stored.state_hash {
        return Err("lifecycle_store.workset.changed_member: workset state hash mismatch".into());
    }
    validate_state_policy(&state, policy)?;
    Ok(state)
}

fn validate_state_policy(state: &WorksetState, policy: &ShellPolicy) -> Result<(), Box<dyn Error>> {
    let state_identity = core_result(workset_state_identity(state))?;
    let portable_identity = core_result(workset_portable_state_identity(state))?;
    if let Some(workset) = state.worksets.first() {
        core_result(plan_workset_open(
            state,
            &policy.core,
            WorksetOpenRequest {
                workset_id: workset.id.clone(),
                opener_override: None,
                expected_state_identity: state_identity,
                expected_portable_state_identity: portable_identity,
                execution: WorksetExecutionMode::Preview,
            },
        ))?;
    }
    Ok(())
}

fn execute_transition(
    paths: &StatePaths,
    policy_path: &Path,
    plan: &WorksetTransitionPlan,
) -> Result<(), Box<dyn Error>> {
    with_exclusive_lock(paths, |root| {
        let policy = load_shell_policy(policy_path)?;
        let current = load_state_from_root(root, paths, &policy)?;
        let next = core_result(apply_workset_transition(&current, &policy.core, plan))?;
        atomic_replace_state(root, paths, &policy, &next)?;
        Ok(())
    })
}

fn execute_open(
    paths: &StatePaths,
    policy_path: &Path,
    plan: &WorksetOpenPlan,
) -> Result<ProcessObservation, Box<dyn Error>> {
    with_exclusive_lock(paths, |root| {
        let policy = load_shell_policy(policy_path)?;
        let current = load_state_from_root(root, paths, &policy)?;
        let workset = require_workset(&current, &plan.workset_id)?;
        recheck_member_paths(workset)?;
        core_result(recheck_workset_open_plan(&current, &policy.core, plan))?;
        launch_exact_argv(&plan.argv)
    })
}

fn with_exclusive_lock<T>(
    paths: &StatePaths,
    operation: impl FnOnce(&CapabilityRoot) -> Result<T, Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    let root = prepare_state_root(paths)?;
    let lock = paths.lock_relative();
    root.write_new(&lock, b"cairn workset exclusive lock\n")
        .map_err(|error| {
            format!("lifecycle_store.workset.stale_plan: workset lock unavailable: {error}")
        })?;
    let outcome = operation(&root);
    let cleanup = root.remove_file(&lock);
    match (outcome, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(format!("failed to remove workset lock: {error}").into()),
    }
}

fn prepare_state_root(paths: &StatePaths) -> Result<CapabilityRoot, Box<dyn Error>> {
    let root = CapabilityRoot::open_or_create(CapabilityRootKind::LocalState, &paths.data_dir)?;
    let directory = Path::new(WORKSET_DIRECTORY);
    match root.symlink_metadata(directory) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_dir() {
                return Err("lifecycle_store.workset.invalid: workset data path must be a non-symlink directory".into());
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => root.create_dir(directory)?,
        Err(error) => return Err(error.into()),
    }
    Ok(root)
}

fn atomic_replace_state(
    root: &CapabilityRoot,
    paths: &StatePaths,
    policy: &ShellPolicy,
    state: &WorksetState,
) -> Result<(), Box<dyn Error>> {
    validate_state_policy(state, policy)?;
    let stored = StoredWorksetState {
        schema: policy.state_schema.clone(),
        worksets: state.worksets.clone(),
        state_hash: core_result(workset_state_identity(state))?,
    };
    let mut bytes = serde_json::to_vec_pretty(&stored)?;
    bytes.push(b'\n');
    let temporary = paths.temporary_relative();
    let destination = paths.state_relative();
    root.write_new(&temporary, &bytes)?;
    if let Ok(metadata) = root.symlink_metadata(&destination) {
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            let _ = root.remove_file(&temporary);
            return Err(
                "lifecycle_store.workset.invalid: workset state destination is unsafe".into(),
            );
        }
    }
    if let Err(error) = root.rename(&temporary, root, &destination) {
        let _ = root.remove_file(&temporary);
        return Err(error.into());
    }
    let reloaded = load_state_from_root(root, paths, policy)?;
    if reloaded != *state {
        return Err(
            "lifecycle_store.workset.stale_plan: workset state changed during atomic replacement"
                .into(),
        );
    }
    Ok(())
}

#[derive(Serialize)]
struct ProcessObservation {
    schema: &'static str,
    launched: bool,
    argv: Vec<String>,
    success: bool,
    exit_code: Option<i32>,
    terminated_without_exit_code: bool,
}

fn launch_exact_argv(argv: &[String]) -> Result<ProcessObservation, Box<dyn Error>> {
    let (executable, arguments) = argv
        .split_first()
        .ok_or("lifecycle_store.workset.invalid: opener argv is empty")?;
    let status = Command::new(executable).args(arguments).status()?;
    Ok(process_observation(argv, status))
}

fn process_observation(argv: &[String], status: ExitStatus) -> ProcessObservation {
    ProcessObservation {
        schema: PROCESS_OBSERVATION_SCHEMA,
        launched: true,
        argv: argv.to_vec(),
        success: status.success(),
        exit_code: status.code(),
        terminated_without_exit_code: status.code().is_none(),
    }
}

fn command_receipt(
    command: &str,
    paths: &StatePaths,
    executed: bool,
    mutated: bool,
    plan: Value,
    process: Option<Value>,
) -> Value {
    json!({
        "schema": WORKSET_COMMAND_SCHEMA,
        "command": command,
        "valid": true,
        "verdict": "pass",
        "executed": executed,
        "mutated": mutated,
        "state_path": paths.state_path(),
        "plan": plan,
        "process": process,
    })
}

fn core_result<T>(result: Result<T, WorksetError>) -> Result<T, Box<dyn Error>> {
    result.map_err(boxed_core_error)
}

fn boxed_core_error(error: WorksetError) -> Box<dyn Error> {
    format!("{}: {error}", core_issue_code(&error)).into()
}

fn core_issue_code(error: &WorksetError) -> &'static str {
    match error {
        WorksetError::UnknownOpenerProfile { .. } => "lifecycle_store.workset.unknown_opener",
        WorksetError::StalePriorStateIdentity
        | WorksetError::StalePriorPortableStateIdentity
        | WorksetError::PlanIdentityMismatch
        | WorksetError::ReviewedPlanMismatch => "lifecycle_store.workset.stale_plan",
        _ => "lifecycle_store.workset.invalid",
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::*;

    fn test_directory(name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "cairn-workset-shell-{name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn test_policy_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../cairn-policy/generated/cairn-policy.json")
    }

    fn workset(id: &str, member: &Path) -> Result<Workset, Box<dyn Error>> {
        Ok(Workset::new(
            id,
            vec![WorksetMember::new(
                PRIMARY_MEMBER_LABEL,
                canonical_member_path(member)?,
            )?],
            PRIMARY_MEMBER_LABEL,
            "code-new-window",
        )?)
    }

    fn create_plan(
        state: &WorksetState,
        policy: &ShellPolicy,
        workset: Workset,
    ) -> Result<WorksetTransitionPlan, Box<dyn Error>> {
        Ok(plan_workset_create(
            state,
            &policy.core,
            WorksetCreateRequest {
                workset,
                expected_prior_state_identity: workset_state_identity(state)?,
                expected_prior_portable_state_identity: workset_portable_state_identity(state)?,
                execution: WorksetExecutionMode::Execute,
            },
        )?)
    }

    // r[verify cairn.lifecycle_stores.functional_boundary.shell]
    #[test]
    fn stale_shell_plan_preserves_the_committed_state() -> Result<(), Box<dyn Error>> {
        let fixture = test_directory("stale-plan")?;
        let data = fixture.join("data");
        let first_member = fixture.join("first");
        let second_member = fixture.join("second");
        fs::create_dir_all(&first_member)?;
        fs::create_dir_all(&second_member)?;
        let paths = StatePaths::explicit(data);
        let policy_path = test_policy_path();
        let policy = load_shell_policy(&policy_path)?;
        let empty = WorksetState::empty();
        let stale = create_plan(&empty, &policy, workset("stale", &first_member)?)?;
        let current = create_plan(&empty, &policy, workset("current", &second_member)?)?;
        execute_transition(&paths, &policy_path, &current)?;

        let error = execute_transition(&paths, &policy_path, &stale).unwrap_err();
        assert!(error
            .to_string()
            .contains("lifecycle_store.workset.stale_plan"));
        let observed = load_state(&paths, &policy)?;
        assert_eq!(observed.worksets.len(), 1);
        assert_eq!(observed.worksets[0].id.as_str(), "current");

        fs::remove_dir_all(fixture)?;
        Ok(())
    }

    #[test]
    fn state_path_matches_the_policy_logical_location() {
        assert_eq!(
            Path::new(CAIRN_DATA_DIRECTORY_NAME)
                .join(WORKSET_DIRECTORY)
                .join(WORKSET_STATE_FILE),
            PathBuf::from(cairn_core::WORKSET_STATE_LOGICAL_PATH)
        );
    }
}
