// r[impl octet.benchmark_evidence.performance_protocol]
// r[impl octet.benchmark_evidence.resource_observation]
pub(super) use super::resource_shell::read_allocation_file;
pub(super) use super::resource_shell::scan_llvm_file;
pub use super::runtime::RuntimeTools;
use super::runtime::environment_observation;
pub use super::runtime::production_tools;
use super::*;

const CACHE_ROOT: &str = ".octet/benchmark-cache";
const HOME_ROOT: &str = ".octet/benchmark-home";
const TEMP_ROOT: &str = ".octet/benchmark-tmp";
const STDIN_MAX_BYTES: usize = 1;
const LOG_STREAM_COUNT: usize = 2;
const ACCEPTED_EXIT_CODE: i32 = 0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunObservation {
    pub input: request::Input,
    pub exit_code: i32,
}

struct ObservedExecution {
    sample: Sample,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: Option<i32>,
}

pub fn run(workspace_root: &std::path::Path, profile: &Profile, baseline: RunSet) -> Result<RunObservation, String> {
    run_with(workspace_root, profile, baseline, production_tools()?)
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &Profile,
    baseline: RunSet,
    tools: RuntimeTools,
) -> Result<RunObservation, String> {
    let workload = profile
        .workloads
        .iter()
        .find(|candidate| candidate.workload_id == baseline.workload_id)
        .ok_or_else(|| "benchmark baseline selects an unknown workload".to_owned())?;
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize benchmark workspace {}: {error}", workspace_root.display()))?;
    prepare_directories(&workspace_root, workload)?;
    let mut retained_logs = Vec::new();
    if workload.mode == "warm" {
        let warmup = execute_once(&workspace_root, profile, workload, &tools, 0)?;
        retained_logs.push(warmup.stdout);
        retained_logs.push(warmup.stderr);
    }
    let mut samples = Vec::with_capacity(workload.sample_count);
    let mut exit_code = ACCEPTED_EXIT_CODE;
    for sample_index in 0..workload.sample_count {
        if workload.mode == "cold" {
            reset_cache(&workspace_root, workload)?;
        }
        let observed = execute_once(&workspace_root, profile, workload, &tools, sample_index)?;
        if observed.sample.outcome != "succeeded" {
            exit_code = match observed.exit_code {
                Some(code) => code,
                None => EXIT_INTEGRATION_FAILURE,
            };
        }
        retained_logs.push(observed.stdout);
        retained_logs.push(observed.stderr);
        samples.push(observed.sample);
    }
    let bounded_log_identity = combined_log_identity(&retained_logs, profile.limits.max_log_bytes)?;
    let allocations = read_allocation_file(
        &workspace_root,
        std::path::Path::new(&profile.resource_observation.allocation_input_path),
        profile,
    )?;
    let copies = scan_llvm_file(
        &workspace_root,
        std::path::Path::new(&profile.resource_observation.llvm_input_path),
        profile,
        &workload.workload_id,
        &profile.resource_observation.compiler_identity,
        &profile.resource_observation.target,
        &profile.resource_observation.optimization_profile,
    )?;
    let candidate = RunSet {
        workload_id: workload.workload_id.clone(),
        workload_identity: workload.workload_identity.clone(),
        command_id: workload.command_id.clone(),
        command_argv: workload.command_argv.clone(),
        mode: workload.mode.clone(),
        source_state_identity: workload.source_state_identity.clone(),
        environment: environment_observation(profile, workload, &tools),
        samples,
    };
    Ok(RunObservation {
        input: request::Input {
            schema_version: INPUT_SCHEMA.to_owned(),
            baseline,
            candidate,
            allocations,
            copies,
            oracle: Vec::new(),
            findings: Vec::new(),
            public_claims: Vec::new(),
            bounded_log_identity,
        },
        exit_code,
    })
}

fn execute_once(
    workspace_root: &std::path::Path,
    profile: &Profile,
    workload: &WorkloadProfile,
    tools: &RuntimeTools,
    sample_index: usize,
) -> Result<ObservedExecution, String> {
    let stream_limit = profile
        .limits
        .max_log_bytes
        .checked_div(LOG_STREAM_COUNT)
        .filter(|limit| *limit > 0)
        .ok_or_else(|| "benchmark log limit cannot be divided between output streams".to_owned())?;
    let args = workload
        .command_argv
        .get(1..)
        .ok_or_else(|| "benchmark command has no subcommand arguments".to_owned())?
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let started = std::time::Instant::now();
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: tools.cargo.clone(),
            args,
            current_dir: workspace_root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: explicit_environment(workspace_root, workload, tools),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: profile.limits.timeout_ms,
            stdin_max_bytes: STDIN_MAX_BYTES,
            stdout_max_bytes: stream_limit,
            stderr_max_bytes: stream_limit,
            poll_interval_ms: profile.limits.poll_interval_ms,
            teardown_timeout_ms: profile.limits.teardown_timeout_ms,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct benchmark outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded benchmark command: {error}"))?;
    let elapsed = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| "benchmark wall duration does not fit u64 nanoseconds".to_owned())?;
    let output_bytes = output
        .stdout
        .bytes
        .len()
        .checked_add(output.stderr.bytes.len())
        .ok_or_else(|| "benchmark output byte count overflowed".to_owned())?;
    let output_bytes =
        u64::try_from(output_bytes).map_err(|_| "benchmark output byte count does not fit u64".to_owned())?;
    let log_identity = sample_log_identity(&output.stdout.bytes, &output.stderr.bytes)?;
    let outcome = if output.disposition == bounded_exec::Disposition::Succeeded {
        "succeeded"
    } else {
        "failed"
    };
    Ok(ObservedExecution {
        sample: Sample {
            sample_index,
            outcome: outcome.to_owned(),
            wall_duration_ns: elapsed,
            cpu_duration_ns: None,
            peak_memory_bytes: None,
            output_bytes,
            log_identity,
        },
        stdout: output.stdout.bytes,
        stderr: output.stderr.bytes,
        exit_code: output.exit_code,
    })
}

fn prepare_directories(workspace_root: &std::path::Path, workload: &WorkloadProfile) -> Result<(), String> {
    let root = crate::cap_fs::ArtifactRoot::open_or_create(workspace_root)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(HOME_ROOT)?)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(TEMP_ROOT)?)?;
    root.create_dir_all(&cache_path(workload)?)
}

fn reset_cache(workspace_root: &std::path::Path, workload: &WorkloadProfile) -> Result<(), String> {
    let root = crate::cap_fs::ArtifactRoot::open_or_create(workspace_root)?;
    let path = cache_path(workload)?;
    root.remove_file_or_dir_all(&path)?;
    root.create_dir_all(&path)
}

fn cache_path(workload: &WorkloadProfile) -> Result<crate::cap_fs::RootRelativePath, String> {
    crate::cap_fs::RootRelativePath::new(format!("{CACHE_ROOT}/{}", workload.workload_id))
}

fn explicit_environment(
    workspace_root: &std::path::Path,
    workload: &WorkloadProfile,
    tools: &RuntimeTools,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    vec![
        ("PATH".into(), tools.path.clone()),
        ("HOME".into(), workspace_root.join(HOME_ROOT).into_os_string()),
        ("TMPDIR".into(), workspace_root.join(TEMP_ROOT).into_os_string()),
        (
            "CARGO_TARGET_DIR".into(),
            workspace_root.join(CACHE_ROOT).join(&workload.workload_id).into_os_string(),
        ),
        ("LD_LIBRARY_PATH".into(), tools.library_path.clone()),
    ]
}

fn combined_log_identity(logs: &[Vec<u8>], max_bytes: usize) -> Result<String, String> {
    let mut total = 0_usize;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"octet-benchmark-bounded-log-v1\0");
    for log in logs {
        total = total.checked_add(log.len()).ok_or_else(|| "benchmark combined log length overflowed".to_owned())?;
        if total > max_bytes {
            return Err("benchmark combined process log exceeds the profile limit".to_owned());
        }
        update_frame(&mut hasher, log)?;
    }
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

fn sample_log_identity(stdout: &[u8], stderr: &[u8]) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"octet-benchmark-sample-log-v1\0");
    update_frame(&mut hasher, stdout)?;
    update_frame(&mut hasher, stderr)?;
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

fn update_frame(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), String> {
    let length = u64::try_from(bytes.len()).map_err(|_| "benchmark log frame length does not fit u64".to_owned())?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

const EXIT_INTEGRATION_FAILURE: i32 = 2;
