use std::io::Read as _;

use super::*;

const COVERAGE_DIRECTORY: &str = ".octet/coverage";
const CARGO_HOME_DIRECTORY: &str = ".octet/coverage/cargo-home";
const TEMP_DIRECTORY: &str = ".octet/coverage/tmp";
const CARGO_CONFIG_NAME: &str = "config.toml";
const STDIN_MAX_BYTES: usize = 1;
const ACCEPTED_EXIT_CODE: i32 = 0;
const LOG_STREAM_COUNT: usize = 2;
const MAX_CARGO_CONFIG_BYTES: u64 = 65_536;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTools {
    pub cargo: std::path::PathBuf,
    pub path: std::ffi::OsString,
    pub cargo_config: std::path::PathBuf,
    pub library_path: std::ffi::OsString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunObservation {
    pub current_text: String,
    pub bounded_log_identity: String,
    pub exit_code: i32,
}

pub fn run(workspace_root: &std::path::Path, profile: &Profile) -> Result<RunObservation, String> {
    run_with(workspace_root, profile, production_tools()?)
}

pub fn production_tools() -> Result<RuntimeTools, String> {
    let cargo = required_absolute_path("OCTET_COVERAGE_CARGO")?;
    let cargo_config = required_absolute_path("OCTET_COVERAGE_CARGO_CONFIG")?;
    let path = required_environment("OCTET_COVERAGE_PATH")?;
    let library_path = required_environment("OCTET_COVERAGE_LD_LIBRARY_PATH")?;
    Ok(RuntimeTools {
        cargo,
        path,
        cargo_config,
        library_path,
    })
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &Profile,
    tools: RuntimeTools,
) -> Result<RunObservation, String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize coverage workspace {}: {error}", workspace_root.display()))?;
    prepare_directories(&workspace_root, &tools.cargo_config)?;
    let args = profile
        .command_argv
        .get(1..)
        .ok_or_else(|| "coverage command has no subcommand arguments".to_owned())?
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let stream_log_limit = profile
        .max_log_bytes
        .checked_div(LOG_STREAM_COUNT)
        .filter(|limit| *limit > 0)
        .ok_or_else(|| "coverage log limit cannot be divided between output streams".to_owned())?;
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: tools.cargo,
            args,
            current_dir: workspace_root.clone(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: explicit_environment(&workspace_root, tools.path, tools.library_path),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: profile.timeout_ms,
            stdin_max_bytes: STDIN_MAX_BYTES,
            stdout_max_bytes: stream_log_limit,
            stderr_max_bytes: stream_log_limit,
            poll_interval_ms: profile.poll_interval_ms,
            teardown_timeout_ms: profile.teardown_timeout_ms,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct coverage outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded coverage command: {error}"))?;
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return Err(format!("coverage command did not succeed: {:?}", output.disposition));
    }
    let exit_code = output.exit_code.ok_or_else(|| "coverage command has no portable exit code".to_owned())?;
    let bounded_log_identity = log_identity(&output.stdout.bytes, &output.stderr.bytes, profile.max_log_bytes)?;
    let report_path = crate::cap_fs::RootRelativePath::new(std::path::Path::new(&profile.current_report_path))?;
    let source_root = crate::cap_fs::SourceRoot::open_existing(&workspace_root)?;
    let report = source_root.read_bounded(&report_path, profile.max_report_bytes)?;
    let current_text =
        String::from_utf8(report).map_err(|error| format!("coverage native report is not UTF-8: {error}"))?;
    Ok(RunObservation {
        current_text,
        bounded_log_identity,
        exit_code,
    })
}

fn prepare_directories(workspace_root: &std::path::Path, cargo_config: &std::path::Path) -> Result<(), String> {
    let coverage_path = workspace_root.join(COVERAGE_DIRECTORY);
    let root = crate::cap_fs::ArtifactRoot::open_or_create(&coverage_path)?;
    let cargo_home = crate::cap_fs::RootRelativePath::new("cargo-home")?;
    let temporary = crate::cap_fs::RootRelativePath::new("tmp")?;
    let destination = crate::cap_fs::RootRelativePath::new(format!("cargo-home/{CARGO_CONFIG_NAME}"))?;
    root.create_dir_all(&cargo_home)?;
    root.create_dir_all(&temporary)?;
    root.write_atomic(&destination, &read_config_bounded(cargo_config)?)
}

fn read_config_bounded(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let length = std::fs::metadata(path)
        .map_err(|error| format!("inspect pinned coverage Cargo config {}: {error}", path.display()))?
        .len();
    if length > MAX_CARGO_CONFIG_BYTES {
        return Err("pinned coverage Cargo config exceeds its byte limit".to_owned());
    }
    let capacity =
        usize::try_from(length).map_err(|_| "pinned coverage Cargo config length does not fit usize".to_owned())?;
    let mut bytes = Vec::with_capacity(capacity);
    std::fs::File::open(path)
        .map_err(|error| format!("open pinned coverage Cargo config {}: {error}", path.display()))?
        .take(MAX_CARGO_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read pinned coverage Cargo config {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).map_err(|_| "coverage Cargo config length does not fit u64".to_owned())? != length {
        return Err("pinned coverage Cargo config changed during bounded read".to_owned());
    }
    Ok(bytes)
}

fn explicit_environment(
    workspace_root: &std::path::Path,
    path: std::ffi::OsString,
    library_path: std::ffi::OsString,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    vec![
        ("PATH".into(), path),
        ("HOME".into(), workspace_root.join(COVERAGE_DIRECTORY).into_os_string()),
        ("CARGO_HOME".into(), workspace_root.join(CARGO_HOME_DIRECTORY).into_os_string()),
        ("TMPDIR".into(), workspace_root.join(TEMP_DIRECTORY).into_os_string()),
        ("LD_LIBRARY_PATH".into(), library_path),
    ]
}

fn required_absolute_path(name: &str) -> Result<std::path::PathBuf, String> {
    let value = std::env::var_os(name).ok_or_else(|| format!("coverage runtime is missing {name}"))?;
    let path = std::path::PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!("coverage runtime {name} is not absolute"));
    }
    Ok(path)
}

fn required_environment(name: &str) -> Result<std::ffi::OsString, String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("coverage runtime is missing {name}"))
}

fn log_identity(stdout: &[u8], stderr: &[u8], max_bytes: usize) -> Result<String, String> {
    let length = stdout.len().checked_add(stderr.len()).ok_or_else(|| "coverage log length overflow".to_owned())?;
    if length > max_bytes {
        return Err("coverage log exceeds profile byte limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(stdout);
    bytes.extend_from_slice(stderr);
    Ok(prefixed_identity(&bytes))
}
