use super::*;

const CARGO_HOME_DIRECTORY: &str = ".octet/mutants-cargo-home";
const TEMP_DIRECTORY: &str = ".octet/mutants-tmp";
const CARGO_CONFIG_NAME: &str = "config.toml";
const MAX_CARGO_CONFIG_BYTES: u64 = 65_536;
const STDIN_MAX_BYTES: usize = 1;
const LOG_STREAM_COUNT: usize = 2;
const ACCEPTED_EXIT_CODES: [i32; 2] = [0, 2];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTools {
    pub cargo: std::path::PathBuf,
    pub path: std::ffi::OsString,
    pub cargo_config: std::path::PathBuf,
    pub library_path: std::ffi::OsString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunObservation {
    pub parts: request::InputParts,
    pub exit_code: i32,
}

pub fn run(
    workspace_root: &std::path::Path,
    profile: &Profile,
    source_manifest_text: String,
) -> Result<RunObservation, String> {
    run_with(workspace_root, profile, source_manifest_text, production_tools()?)
}

pub fn production_tools() -> Result<RuntimeTools, String> {
    Ok(RuntimeTools {
        cargo: required_absolute_path("OCTET_MUTATION_CARGO")?,
        path: required_environment("OCTET_MUTATION_PATH")?,
        cargo_config: required_absolute_path("OCTET_MUTATION_CARGO_CONFIG")?,
        library_path: required_environment("OCTET_MUTATION_LD_LIBRARY_PATH")?,
    })
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &Profile,
    source_manifest_text: String,
    tools: RuntimeTools,
) -> Result<RunObservation, String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize mutation workspace {}: {error}", workspace_root.display()))?;
    prepare_workspace(&workspace_root, profile, &tools.cargo_config)?;
    let stream_limit = profile
        .max_log_bytes
        .checked_div(LOG_STREAM_COUNT)
        .filter(|limit| *limit > 0)
        .ok_or_else(|| "mutation log limit cannot be divided between output streams".to_owned())?;
    let args = profile
        .command_argv
        .get(1..)
        .ok_or_else(|| "mutation command has no subcommand arguments".to_owned())?
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
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
            stdout_max_bytes: stream_limit,
            stderr_max_bytes: stream_limit,
            poll_interval_ms: profile.poll_interval_ms,
            teardown_timeout_ms: profile.teardown_timeout_ms,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(ACCEPTED_EXIT_CODES.to_vec(), true, true)
            .map_err(|error| format!("construct mutation outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded cargo-mutants command: {error}"))?;
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return Err(format!("cargo-mutants command did not complete: {:?}", output.disposition));
    }
    let exit_code = output.exit_code.ok_or_else(|| "cargo-mutants command has no portable exit code".to_owned())?;
    let bounded_log_identity =
        combined_log_identity(&output.stdout.bytes, &output.stderr.bytes, profile.max_log_bytes)?;
    let mut parts = collect_native(&workspace_root, profile).map_err(|error| {
        format!(
            "{error}; cargo-mutants stdout=`{}` stderr=`{}`",
            String::from_utf8_lossy(&output.stdout.bytes),
            String::from_utf8_lossy(&output.stderr.bytes)
        )
    })?;
    parts.source_manifest_text = source_manifest_text;
    parts.bounded_log_identity = bounded_log_identity;
    Ok(RunObservation { parts, exit_code })
}

pub fn collect_native(workspace_root: &std::path::Path, profile: &Profile) -> Result<request::InputParts, String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let outcomes_path = crate::cap_fs::RootRelativePath::new(&profile.native_outcomes_path)?;
    let inventory_path = crate::cap_fs::RootRelativePath::new(&profile.native_inventory_path)?;
    let outcomes = root.read_bounded(&outcomes_path, profile.max_report_bytes)?;
    let inventory = root.read_bounded(&inventory_path, profile.max_report_bytes)?;
    let outcomes_text =
        String::from_utf8(outcomes).map_err(|error| format!("mutation outcomes are not UTF-8: {error}"))?;
    let inventory_text =
        String::from_utf8(inventory).map_err(|error| format!("mutation inventory is not UTF-8: {error}"))?;
    let artifact_paths = referenced_artifacts(&outcomes_text, profile.max_artifact_count)?;
    let parent = std::path::Path::new(&profile.native_outcomes_path)
        .parent()
        .ok_or_else(|| "mutation outcomes path has no parent".to_owned())?;
    let artifacts = artifact_paths
        .into_iter()
        .map(|path| {
            let workspace_path = parent.join(&path);
            let bytes =
                root.read_bounded(&crate::cap_fs::RootRelativePath::new(workspace_path)?, profile.max_report_bytes)?;
            let text = String::from_utf8(bytes)
                .map_err(|error| format!("mutation artifact `{path}` is not UTF-8: {error}"))?;
            Ok((path, text))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, String>>()?;
    Ok(request::InputParts {
        outcomes_text,
        inventory_text,
        source_manifest_text: String::new(),
        artifacts,
        bounded_log_identity: prefixed_identity(b""),
    })
}

fn referenced_artifacts(text: &str, max_count: usize) -> Result<Vec<String>, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|error| format!("parse mutation artifact references: {error}"))?;
    let outcomes = value
        .get("outcomes")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "mutation outcomes are missing artifact references".to_owned())?;
    let mut paths = std::collections::BTreeSet::new();
    for outcome in outcomes {
        for field in ["log_path", "diff_path"] {
            if let Some(path) = outcome.get(field).and_then(serde_json::Value::as_str) {
                paths.insert(path.to_owned());
            }
        }
    }
    if paths.len() > max_count {
        return Err("mutation artifact reference count exceeds profile limit".to_owned());
    }
    Ok(paths.into_iter().collect())
}

fn prepare_workspace(
    workspace_root: &std::path::Path,
    profile: &Profile,
    cargo_config: &std::path::Path,
) -> Result<(), String> {
    let root = crate::cap_fs::ArtifactRoot::open_or_create(workspace_root)?;
    let output = crate::cap_fs::RootRelativePath::new(&profile.output_root)?;
    let cargo_home = crate::cap_fs::RootRelativePath::new(CARGO_HOME_DIRECTORY)?;
    let temporary = crate::cap_fs::RootRelativePath::new(TEMP_DIRECTORY)?;
    root.remove_file_or_dir_all(&output)?;
    root.create_dir_all(&output)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(format!("{}/mutants.out/log", profile.output_root))?)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(format!("{}/mutants.out/diff", profile.output_root))?)?;
    root.create_dir_all(&cargo_home)?;
    root.create_dir_all(&temporary)?;
    let config_path = crate::cap_fs::RootRelativePath::new(format!("{CARGO_HOME_DIRECTORY}/{CARGO_CONFIG_NAME}"))?;
    root.write_atomic(&config_path, &read_config_bounded(cargo_config)?)
}

fn read_config_bounded(path: &std::path::Path) -> Result<Vec<u8>, String> {
    use std::io::Read as _;
    let file = std::fs::File::open(path)
        .map_err(|error| format!("open pinned mutation Cargo config {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(MAX_CARGO_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read pinned mutation Cargo config {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).map_err(|_| "mutation Cargo config length does not fit u64".to_owned())?
        > MAX_CARGO_CONFIG_BYTES
    {
        return Err("pinned mutation Cargo config exceeds its byte limit".to_owned());
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
        ("HOME".into(), workspace_root.join(".octet").into_os_string()),
        ("CARGO_HOME".into(), workspace_root.join(CARGO_HOME_DIRECTORY).into_os_string()),
        ("TMPDIR".into(), workspace_root.join(TEMP_DIRECTORY).into_os_string()),
        ("LD_LIBRARY_PATH".into(), library_path),
    ]
}

fn required_absolute_path(name: &str) -> Result<std::path::PathBuf, String> {
    let path =
        std::path::PathBuf::from(std::env::var_os(name).ok_or_else(|| format!("mutation runtime is missing {name}"))?);
    if !path.is_absolute() {
        return Err(format!("mutation runtime {name} is not absolute"));
    }
    Ok(path)
}

fn required_environment(name: &str) -> Result<std::ffi::OsString, String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("mutation runtime is missing {name}"))
}

fn combined_log_identity(stdout: &[u8], stderr: &[u8], max_bytes: usize) -> Result<String, String> {
    let length = stdout.len().checked_add(stderr.len()).ok_or_else(|| "mutation log length overflow".to_owned())?;
    if length > max_bytes {
        return Err("mutation process log exceeds profile limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(stdout);
    bytes.extend_from_slice(stderr);
    Ok(prefixed_identity(&bytes))
}
