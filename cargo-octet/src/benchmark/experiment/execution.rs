use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use super::evaluate::evaluate;
use super::evaluate::scope_identity;
use super::model::AdoptionObservation;
use super::model::AdoptionPhase;
use super::model::ApplicabilityStatus;
use super::model::Availability;
use super::model::CacheState;
use super::model::CompileStatus;
use super::model::ExperimentArtifact;
use super::model::ExperimentObservations;
use super::model::ExperimentProfile;
use super::model::MeasurementOrigin;
use super::model::OBSERVATIONS_SCHEMA;
use super::model::ObservationEnvironment;
use super::model::ObservationScope;
use super::model::ObservationStatus;
use super::model::ToolAvailability;
use super::model::ToolObservation;
use super::model::ToolPin;
use super::model::WarmState;
use super::profile::selected_profile;
use super::report::render_report;
use crate::benchmark::prefixed_identity;
use crate::benchmark::pretty_bytes;

const PROBE_TIMEOUT_MS: u64 = 10_000;
const ADOPTION_TIMEOUT_MS: u64 = 120_000;
const STREAM_LIMIT_BYTES: usize = 262_144;
const POLL_INTERVAL_MS: u64 = 25;
const TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const STDIN_LIMIT_BYTES: usize = 1;
static SCRATCH_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExperimentMode {
    Recorded,
    Execute,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutcome {
    pub program: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub succeeded: bool,
    pub timed_out: bool,
    pub duration_ns: u64,
    pub truncated: bool,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentRun {
    pub artifact: ExperimentArtifact,
    pub observations: ExperimentObservations,
    pub report: String,
    pub artifact_bytes: Vec<u8>,
    pub report_bytes: Vec<u8>,
}

pub fn run(
    mode: ExperimentMode,
    profile_id: &str,
    input: Option<&str>,
    corpus_root: &Path,
) -> Result<ExperimentRun, Vec<String>> {
    let profile = selected_profile(profile_id).map_err(|error| vec![error])?;
    let observations = match mode {
        ExperimentMode::Recorded => {
            let Some(input) = input else {
                return Err(vec!["experiment recorded mode requires a retained observation input".to_owned()]);
            };
            serde_json::from_str::<ExperimentObservations>(input)
                .map_err(|error| vec![format!("parse experiment observations: {error}")])?
        }
        ExperimentMode::Execute => execute_observations(&profile, corpus_root)?,
    };
    let evaluation = evaluate(profile_id, &observations);
    let Some(artifact) = evaluation.artifact else {
        return Err(evaluation.diagnostics);
    };
    let report = render_report(&artifact);
    let artifact_bytes = pretty_bytes(&artifact, "experiment artifact").map_err(|error| vec![error])?;
    let report_bytes = report.clone().into_bytes();
    Ok(ExperimentRun {
        artifact,
        observations,
        report,
        artifact_bytes,
        report_bytes,
    })
}

pub fn probe_tool(tool: &ToolPin) -> ToolAvailability {
    let Some(program) = tool.probe_argv.first() else {
        return unavailable(tool, "probe argv is empty");
    };
    let Some(executable) = resolve_executable(program) else {
        return unavailable(tool, &format!("executable `{program}` is not available on PATH"));
    };
    let args = tool.probe_argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
    match execute(&executable, args, Path::new("/"), &base_environment(), PROBE_TIMEOUT_MS) {
        Ok(outcome) if outcome.succeeded => {
            let version_output = format!("{}{}", outcome.stdout, outcome.stderr);
            ToolAvailability {
                tool_id: tool.tool_id.clone(),
                status: Availability::Local,
                version_identity: prefixed_identity(version_output.as_bytes()),
                version_output,
                reason: None,
            }
        }
        Ok(outcome) => unavailable(tool, &format!("probe failed: {:?}", outcome.reason)),
        Err(error) => unavailable(tool, &format!("probe could not run: {error}")),
    }
}

#[cfg(test)]
pub fn run_command(root: &Path, argv: &[String]) -> Result<CommandOutcome, String> {
    let Some(program) = argv.first() else {
        return Err("experiment command is empty".to_owned());
    };
    let executable = resolve_executable(program)
        .ok_or_else(|| format!("experiment executable `{program}` is not available on PATH"))?;
    let args = argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
    let root = absolute_root(root)?;
    let scratch = scratch_directory("command");
    let target_dir = scratch.join("target");
    let home = scratch.join("home");
    let temp = scratch.join("tmp");
    for directory in [&target_dir, &home, &temp] {
        let _ = std::fs::create_dir_all(directory);
    }
    let environment = adoption_environment(&home, &temp, &target_dir);
    let outcome = execute(&executable, args, &root, &environment, ADOPTION_TIMEOUT_MS);
    let _ = std::fs::remove_dir_all(&scratch);
    outcome
}

fn execute_observations(
    profile: &ExperimentProfile,
    corpus_root: &Path,
) -> Result<ExperimentObservations, Vec<String>> {
    let corpus_root = absolute_root(corpus_root).map_err(|error| vec![error])?;
    let mut tool_availability = Vec::new();
    let mut observations = Vec::new();
    let mut adoption = Vec::new();
    for tool in &profile.tools {
        let probe = probe_tool(tool);
        let executable_available = probe.status != Availability::Unavailable;
        let mut local_measurement = false;
        for plan in profile.adoption.tools.iter().filter(|plan| plan.tool_id == tool.tool_id) {
            let adoption_defined = !tool.run_argv.is_empty();
            let (clean_initial, clean_incremental) = measure_target(
                profile,
                tool,
                &corpus_root,
                &profile.adoption.clean_consumer.selection_id,
                &plan.initial_command,
                &plan.incremental_command,
                executable_available,
                adoption_defined,
            );
            local_measurement |= clean_initial.measured || clean_incremental.measured;
            adoption.push(clean_initial);
            adoption.push(clean_incremental);
            let real_root = Path::new(&profile.adoption.real_consumer.workspace_path);
            let (real_initial, real_incremental) = if real_root.is_dir() {
                measure_target(
                    profile,
                    tool,
                    real_root,
                    &profile.adoption.real_consumer.selection_id,
                    &plan.initial_command,
                    &plan.incremental_command,
                    executable_available,
                    adoption_defined,
                )
            } else {
                (
                    unavailable_adoption(
                        profile,
                        tool,
                        &profile.adoption.real_consumer.selection_id,
                        AdoptionPhase::Initial,
                        "immutable real-consumer selection is not checked out",
                    ),
                    unavailable_adoption(
                        profile,
                        tool,
                        &profile.adoption.real_consumer.selection_id,
                        AdoptionPhase::Incremental,
                        "immutable real-consumer selection is not checked out",
                    ),
                )
            };
            adoption.push(real_initial);
            adoption.push(real_incremental);
        }
        let status = if !executable_available {
            Availability::Unavailable
        } else if local_measurement {
            Availability::Local
        } else {
            Availability::Recorded
        };
        tool_availability.push(ToolAvailability {
            tool_id: tool.tool_id.clone(),
            status,
            version_output: probe.version_output.clone(),
            version_identity: probe.version_identity.clone(),
            reason: probe.reason.clone(),
        });
        for entry in profile.applicability.iter().filter(|entry| entry.tool_id == tool.tool_id) {
            if entry.status == ApplicabilityStatus::Unsupported {
                continue;
            }
            for case_id in &entry.case_ids {
                let Some(case) = profile.corpus.iter().find(|case| case.case_id == *case_id) else {
                    continue;
                };
                let scope = ObservationScope {
                    selected_sources: case.scope.selected_sources.clone(),
                    analyzed_sources: Vec::new(),
                    scope_identity: String::new(),
                };
                let scope = ObservationScope {
                    scope_identity: scope_identity(&scope),
                    ..scope
                };
                let stdout = String::new();
                let stderr = String::new();
                let status = if executable_available {
                    ObservationStatus::Unparsed
                } else {
                    ObservationStatus::Unavailable
                };
                let reason = if executable_available {
                    "execute mode probes availability and adoption only; finding extraction requires a parser and is not claimed"
                } else {
                    "executable is unavailable; unsupported scope is not clean evidence"
                };
                observations.push(ToolObservation {
                    observation_id: format!("{}:{}:{}", tool.tool_id, entry.task_id, case.case_id),
                    tool_id: tool.tool_id.clone(),
                    task_id: entry.task_id.clone(),
                    case_id: Some(case.case_id.clone()),
                    source_identity: case.source_identity.clone(),
                    tool_version: tool.version_pin.clone(),
                    origin: MeasurementOrigin::Local,
                    environment: environment(profile, CacheState::Cold),
                    scope,
                    status,
                    compile_status: CompileStatus::Unknown,
                    findings: Vec::new(),
                    suppressions: Vec::new(),
                    stdout_identity: prefixed_identity(stdout.as_bytes()),
                    stderr_identity: prefixed_identity(stderr.as_bytes()),
                    stdout,
                    stderr,
                    exit_code: None,
                    wall_duration_ns: None,
                    warm_state: WarmState::Cold,
                    sample_index: 0,
                    failure_reason: None,
                    unavailable_reason: Some(reason.to_owned()),
                });
            }
        }
    }
    Ok(ExperimentObservations {
        schema_version: OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        tool_availability,
        observations,
        adoption,
    })
}

pub(super) fn measure_target(
    profile: &ExperimentProfile,
    tool: &ToolPin,
    root: &Path,
    selection_id: &str,
    initial_command: &[String],
    incremental_command: &[String],
    executable_available: bool,
    adoption_defined: bool,
) -> (AdoptionObservation, AdoptionObservation) {
    if !executable_available {
        return (
            unavailable_adoption(profile, tool, selection_id, AdoptionPhase::Initial, "executable is unavailable"),
            unavailable_adoption(profile, tool, selection_id, AdoptionPhase::Incremental, "executable is unavailable"),
        );
    }
    if !adoption_defined {
        return (
            unavailable_adoption(
                profile,
                tool,
                selection_id,
                AdoptionPhase::Initial,
                "no reviewed adoption command; probe-only tool",
            ),
            unavailable_adoption(
                profile,
                tool,
                selection_id,
                AdoptionPhase::Incremental,
                "no reviewed adoption command; probe-only tool",
            ),
        );
    }
    let scratch = scratch_directory(&sanitize(&tool.tool_id));
    let _ = std::fs::remove_dir_all(&scratch);
    let target_dir = scratch.join("target");
    let home = scratch.join("home");
    let temp = scratch.join("tmp");
    for directory in [&target_dir, &home, &temp] {
        let _ = std::fs::create_dir_all(directory);
    }
    let environment = adoption_environment(&home, &temp, &target_dir);
    let initial = run_adoption_phase(
        profile,
        tool,
        root,
        selection_id,
        AdoptionPhase::Initial,
        WarmState::Cold,
        initial_command,
        &environment,
    );
    let incremental = run_adoption_phase(
        profile,
        tool,
        root,
        selection_id,
        AdoptionPhase::Incremental,
        WarmState::Warm,
        incremental_command,
        &environment,
    );
    let _ = std::fs::remove_dir_all(&scratch);
    (initial, incremental)
}

fn run_adoption_phase(
    profile: &ExperimentProfile,
    tool: &ToolPin,
    root: &Path,
    selection_id: &str,
    phase: AdoptionPhase,
    warm_state: WarmState,
    argv: &[String],
    environment: &[(OsString, OsString)],
) -> AdoptionObservation {
    let phase_name = match phase {
        AdoptionPhase::Initial => "initial",
        AdoptionPhase::Incremental => "incremental",
    };
    let Some(program) = argv.first() else {
        return unavailable_adoption(profile, tool, selection_id, phase, "adoption command is empty");
    };
    let Some(executable) = resolve_executable(program) else {
        return unavailable_adoption(
            profile,
            tool,
            selection_id,
            phase,
            &format!("executable `{program}` is not available on PATH"),
        );
    };
    let args = argv.iter().skip(1).map(|argument| OsString::from(argument.as_str())).collect();
    let outcome = execute(&executable, args, root, environment, ADOPTION_TIMEOUT_MS);
    let cache_state = match phase {
        AdoptionPhase::Initial => CacheState::Cold,
        AdoptionPhase::Incremental => CacheState::Warm,
    };
    let (stdout, stderr, exit_code, measured, duration, reason) = match outcome {
        Ok(outcome) => (
            outcome.stdout,
            outcome.stderr,
            outcome.exit_code,
            outcome.succeeded,
            Some(outcome.duration_ns),
            outcome.reason,
        ),
        Err(error) => (String::new(), String::new(), None, false, None, Some(error)),
    };
    AdoptionObservation {
        observation_id: format!("{}:{selection_id}:{phase_name}", tool.tool_id),
        tool_id: tool.tool_id.clone(),
        target_selection: selection_id.to_owned(),
        phase,
        origin: MeasurementOrigin::Local,
        measured,
        wall_duration_ns: duration,
        warm_state,
        environment: environment_observation(profile, cache_state),
        stdout_identity: prefixed_identity(stdout.as_bytes()),
        stderr_identity: prefixed_identity(stderr.as_bytes()),
        stdout,
        stderr,
        exit_code,
        unavailable_reason: reason,
        dependencies: profile.sample_plan.dependency_differences.clone(),
    }
}

fn unavailable_adoption(
    profile: &ExperimentProfile,
    tool: &ToolPin,
    selection_id: &str,
    phase: AdoptionPhase,
    reason: &str,
) -> AdoptionObservation {
    let phase_name = match phase {
        AdoptionPhase::Initial => "initial",
        AdoptionPhase::Incremental => "incremental",
    };
    let cache_state = match phase {
        AdoptionPhase::Initial => CacheState::Cold,
        AdoptionPhase::Incremental => CacheState::Warm,
    };
    AdoptionObservation {
        observation_id: format!("{}:{selection_id}:{phase_name}", tool.tool_id),
        tool_id: tool.tool_id.clone(),
        target_selection: selection_id.to_owned(),
        phase,
        origin: MeasurementOrigin::RecordedFixture,
        measured: false,
        wall_duration_ns: None,
        warm_state: match phase {
            AdoptionPhase::Initial => WarmState::Cold,
            AdoptionPhase::Incremental => WarmState::Warm,
        },
        environment: environment_observation(profile, cache_state),
        stdout: String::new(),
        stderr: String::new(),
        stdout_identity: prefixed_identity(b""),
        stderr_identity: prefixed_identity(b""),
        exit_code: None,
        unavailable_reason: Some(reason.to_owned()),
        dependencies: profile.sample_plan.dependency_differences.clone(),
    }
}

fn environment(profile: &ExperimentProfile, cache_state: CacheState) -> ObservationEnvironment {
    environment_observation(profile, cache_state)
}

fn environment_observation(profile: &ExperimentProfile, cache_state: CacheState) -> ObservationEnvironment {
    ObservationEnvironment {
        target: profile.sample_plan.target.clone(),
        host_os: std::env::consts::OS.to_owned(),
        host_arch: std::env::consts::ARCH.to_owned(),
        rust_toolchain: profile.rust_toolchain.clone(),
        cache_state,
    }
}

fn unavailable(tool: &ToolPin, reason: &str) -> ToolAvailability {
    ToolAvailability {
        tool_id: tool.tool_id.clone(),
        status: Availability::Unavailable,
        version_output: String::new(),
        version_identity: prefixed_identity(b""),
        reason: Some(reason.to_owned()),
    }
}

fn base_environment() -> Vec<(OsString, OsString)> {
    const PRESERVED: [&str; 6] = [
        "PATH",
        "RUSTUP_HOME",
        "CARGO_HOME",
        "RUSTUP_TOOLCHAIN",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
    ];
    let mut environment = Vec::new();
    for name in PRESERVED {
        if let Some(value) = std::env::var_os(name) {
            environment.push((OsString::from(name), value));
        }
    }
    environment
}

fn adoption_environment(home: &Path, temp: &Path, target: &Path) -> Vec<(OsString, OsString)> {
    let mut environment = base_environment();
    environment.push((OsString::from("HOME"), home.as_os_str().to_owned()));
    environment.push((OsString::from("TMPDIR"), temp.as_os_str().to_owned()));
    environment.push((OsString::from("CARGO_TARGET_DIR"), target.as_os_str().to_owned()));
    environment
}

fn execute(
    executable: &Path,
    args: Vec<OsString>,
    root: &Path,
    environment: &[(OsString, OsString)],
    timeout_ms: u64,
) -> Result<CommandOutcome, String> {
    let started = std::time::Instant::now();
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: executable.to_path_buf(),
            args,
            current_dir: root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: environment.to_vec(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms,
            stdin_max_bytes: STDIN_LIMIT_BYTES,
            stdout_max_bytes: STREAM_LIMIT_BYTES,
            stderr_max_bytes: STREAM_LIMIT_BYTES,
            poll_interval_ms: POLL_INTERVAL_MS,
            teardown_timeout_ms: TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![0], true, true)
            .map_err(|error| format!("construct experiment outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded experiment command: {error}"))?;
    let duration_ns = u64::try_from(started.elapsed().as_nanos())
        .map_err(|_| "experiment duration does not fit u64 nanoseconds".to_owned())?;
    let stdout = String::from_utf8_lossy(&output.stdout.bytes).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr.bytes).into_owned();
    let succeeded = output.disposition == bounded_exec::Disposition::Succeeded;
    let timed_out = output.completion == bounded_exec::Completion::TimedOut;
    let truncated = output.stdout.truncated || output.stderr.truncated;
    let reason = if succeeded {
        None
    } else if timed_out {
        Some("experiment command timed out".to_owned())
    } else {
        Some(format!(
            "experiment command did not succeed: {:?} (exit {:?})",
            output.disposition, output.exit_code
        ))
    };
    Ok(CommandOutcome {
        program: executable.display().to_string(),
        stdout,
        stderr,
        exit_code: output.exit_code,
        succeeded,
        timed_out,
        duration_ns,
        truncated,
        reason,
    })
}

fn resolve_executable(program: &str) -> Option<PathBuf> {
    if program.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(program);
        return is_executable(&path).then_some(path);
    }
    let path = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join(program);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn absolute_root(root: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(root).map_err(|error| format!("canonicalize experiment root {}: {error}", root.display()))
}

fn scratch_directory(tag: &str) -> PathBuf {
    let counter = SCRATCH_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "octet-benchmark-experiment-{}-{}-{tag}-{counter}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default()
    ))
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}
