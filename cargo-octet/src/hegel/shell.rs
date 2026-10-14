use super::*;

const CARGO_HOME_ROOT: &str = ".octet/hegel-cargo-home";
const TEMP_ROOT: &str = ".octet/hegel-tmp";
const TARGET_ROOT: &str = ".octet/hegel-target";
const CARGO_CONFIG_NAME: &str = "config.toml";
const CARGO_LOCK_PATH: &str = "Cargo.lock";
const PACKAGE_MANIFEST_PATH: &str = "cargo-octet/Cargo.toml";
const MAX_CARGO_CONFIG_BYTES: u64 = 65_536;
const STDIN_MAX_BYTES: usize = 1;
const LOG_STREAM_COUNT: usize = 2;
const ACCEPTED_EXIT_CODE: i32 = 0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeTools {
    pub cargo: std::path::PathBuf,
    pub path: std::ffi::OsString,
    pub cargo_config: std::path::PathBuf,
    pub library_path: std::ffi::OsString,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RunObservation {
    pub input: request::Input,
    pub exit_code: i32,
}

pub fn run(workspace_root: &std::path::Path, profile: &Profile) -> Result<RunObservation, String> {
    run_with(workspace_root, profile, production_tools()?)
}

pub fn production_tools() -> Result<RuntimeTools, String> {
    Ok(RuntimeTools {
        cargo: required_absolute_path("OCTET_HEGEL_CARGO")?,
        path: required_environment("OCTET_HEGEL_PATH")?,
        cargo_config: required_absolute_path("OCTET_HEGEL_CARGO_CONFIG")?,
        library_path: required_environment("OCTET_HEGEL_LD_LIBRARY_PATH")?,
    })
}

pub fn run_with(
    workspace_root: &std::path::Path,
    profile: &Profile,
    tools: RuntimeTools,
) -> Result<RunObservation, String> {
    reject_profile_drift(profile)?;
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize Hegel workspace {}: {error}", workspace_root.display()))?;
    let measured = measure_inputs(&workspace_root, profile)?;
    prepare_directories(&workspace_root, profile, &tools.cargo_config)?;
    let stream_limit = profile
        .limits
        .max_log_bytes
        .checked_div(LOG_STREAM_COUNT)
        .filter(|limit| *limit > 0)
        .ok_or_else(|| "Hegel log limit cannot be divided between output streams".to_owned())?;
    let args = profile
        .command_argv
        .get(1..)
        .ok_or_else(|| "Hegel fixed command has no arguments".to_owned())?
        .iter()
        .map(std::ffi::OsString::from)
        .collect();
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: tools.cargo.clone(),
            args,
            current_dir: workspace_root.clone(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: explicit_environment(&workspace_root, &tools, &measured, profile),
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
            .map_err(|error| format!("construct Hegel outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded Hegel producer: {error}"))?;
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return Err(format!("Hegel producer did not complete: {:?}", output.disposition));
    }
    let exit_code = output.exit_code.ok_or_else(|| "Hegel producer has no portable exit code".to_owned())?;
    let bounded_log = combined_log(&output.stdout.bytes, &output.stderr.bytes, profile.limits.max_log_bytes)?;
    let root = crate::cap_fs::SourceRoot::open_existing(&workspace_root)?;
    let summary_path = crate::cap_fs::RootRelativePath::new(&profile.summary_path)?;
    let summary_bytes =
        safe_read::read_no_symlinks(&workspace_root, &root, &summary_path, profile.limits.max_summary_bytes)?;
    let summary_text =
        String::from_utf8(summary_bytes).map_err(|error| format!("Hegel producer summary is not UTF-8: {error}"))?;
    let summary: ProducerSummary = serde_json::from_str(&summary_text)
        .map_err(|error| format!("parse dedicated Hegel producer summary after run: {error}"))?;
    let counterexample = if summary.counterexample_identity.is_some() {
        let path = crate::cap_fs::RootRelativePath::new(&profile.counterexample_path)?;
        Some(safe_read::read_no_symlinks(&workspace_root, &root, &path, profile.limits.max_counterexample_bytes)?)
    } else {
        None
    };
    Ok(RunObservation {
        input: request::Input {
            schema_version: REQUEST_SCHEMA.to_owned(),
            summary_text,
            bounded_log,
            source_manifest: measured.source_manifest,
            generator_manifest: measured.generator_manifest,
            cargo_lock_identity: measured.cargo_lock_identity,
            package_manifest_identity: measured.package_manifest_identity,
            counterexample,
        },
        exit_code,
    })
}

struct MeasuredInputs {
    source_manifest: Vec<FileIdentity>,
    generator_manifest: Vec<FileIdentity>,
    source_manifest_identity: String,
    generator_manifest_identity: String,
    cargo_lock_identity: String,
    package_manifest_identity: String,
}

fn measure_inputs(workspace_root: &std::path::Path, profile: &Profile) -> Result<MeasuredInputs, String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let source_manifest =
        measure_manifest(workspace_root, &root, &profile.source_scope, profile.limits.max_summary_bytes)?;
    let generator_manifest =
        measure_manifest(workspace_root, &root, &profile.generator_inputs, profile.limits.max_summary_bytes)?;
    let cargo_lock_path = crate::cap_fs::RootRelativePath::new(CARGO_LOCK_PATH)?;
    let cargo_lock =
        safe_read::read_no_symlinks(workspace_root, &root, &cargo_lock_path, profile.limits.max_log_bytes as u64)?;
    let package_path = crate::cap_fs::RootRelativePath::new(PACKAGE_MANIFEST_PATH)?;
    let package_manifest =
        safe_read::read_no_symlinks(workspace_root, &root, &package_path, profile.limits.max_summary_bytes)?;
    Ok(MeasuredInputs {
        source_manifest_identity: request::manifest_identity(&source_manifest, "Hegel source manifest")?,
        generator_manifest_identity: request::manifest_identity(&generator_manifest, "Hegel generator manifest")?,
        cargo_lock_identity: prefixed_identity(&cargo_lock),
        package_manifest_identity: prefixed_identity(&package_manifest),
        source_manifest,
        generator_manifest,
    })
}

fn measure_manifest(
    workspace_root: &std::path::Path,
    root: &crate::cap_fs::SourceRoot,
    paths: &[String],
    max_bytes: u64,
) -> Result<Vec<FileIdentity>, String> {
    let mut files = paths
        .iter()
        .map(|path| {
            let relative = crate::cap_fs::RootRelativePath::new(path)?;
            let bytes = safe_read::read_no_symlinks(workspace_root, root, &relative, max_bytes)?;
            Ok(FileIdentity {
                locator: path.clone(),
                identity: prefixed_identity(&bytes),
                bytes: u64::try_from(bytes.len()).map_err(|_| "Hegel source byte count does not fit u64".to_owned())?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    files.sort_by(|left, right| left.locator.cmp(&right.locator));
    Ok(files)
}

fn prepare_directories(
    workspace_root: &std::path::Path,
    profile: &Profile,
    cargo_config: &std::path::Path,
) -> Result<(), String> {
    let root = crate::cap_fs::ArtifactRoot::open_or_create(workspace_root)?;
    let output = crate::cap_fs::RootRelativePath::new(&profile.output_root)?;
    root.remove_file_or_dir_all(&output)?;
    root.create_dir_all(&output)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(CARGO_HOME_ROOT)?)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(TEMP_ROOT)?)?;
    root.create_dir_all(&crate::cap_fs::RootRelativePath::new(TARGET_ROOT)?)?;
    let config = read_bounded_file(cargo_config, MAX_CARGO_CONFIG_BYTES, "Hegel Cargo config")?;
    root.write_atomic(&crate::cap_fs::RootRelativePath::new(format!("{CARGO_HOME_ROOT}/{CARGO_CONFIG_NAME}"))?, &config)
}

fn explicit_environment(
    workspace_root: &std::path::Path,
    tools: &RuntimeTools,
    measured: &MeasuredInputs,
    profile: &Profile,
) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    vec![
        ("PATH".into(), tools.path.clone()),
        ("HOME".into(), workspace_root.join(".octet").into_os_string()),
        ("CARGO_HOME".into(), workspace_root.join(CARGO_HOME_ROOT).into_os_string()),
        ("TMPDIR".into(), workspace_root.join(TEMP_ROOT).into_os_string()),
        ("CARGO_TARGET_DIR".into(), workspace_root.join(TARGET_ROOT).into_os_string()),
        ("LD_LIBRARY_PATH".into(), tools.library_path.clone()),
        ("CI".into(), "1".into()),
        ("OCTET_HEGEL_OUTPUT".into(), workspace_root.join(&profile.summary_path).into_os_string()),
        ("OCTET_HEGEL_PROFILE_IDENTITY".into(), PROFILE_IDENTITY.into()),
        ("OCTET_HEGEL_SOURCE_IDENTITY".into(), measured.source_manifest_identity.clone().into()),
        ("OCTET_HEGEL_GENERATOR_IDENTITY".into(), measured.generator_manifest_identity.clone().into()),
        ("OCTET_HEGEL_LOCK_IDENTITY".into(), measured.cargo_lock_identity.clone().into()),
        ("OCTET_HEGEL_PACKAGE_IDENTITY".into(), measured.package_manifest_identity.clone().into()),
    ]
}

fn combined_log(stdout: &[u8], stderr: &[u8], max_bytes: usize) -> Result<Vec<u8>, String> {
    const STDERR_MARKER: &[u8] = b"\n--- stderr ---\n";
    let length = stdout
        .len()
        .checked_add(STDERR_MARKER.len())
        .and_then(|value| value.checked_add(stderr.len()))
        .ok_or_else(|| "Hegel combined log byte count overflowed".to_owned())?;
    if length > max_bytes {
        return Err("Hegel combined log exceeds the profile limit".to_owned());
    }
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(stdout);
    bytes.extend_from_slice(STDERR_MARKER);
    bytes.extend_from_slice(stderr);
    Ok(bytes)
}

fn reject_profile_drift(profile: &Profile) -> Result<(), String> {
    let diagnostics = profile_validation::validate(profile);
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid Hegel producer profile:\n- {}", diagnostics.join("\n- ")))
    }
}

fn read_bounded_file(path: &std::path::Path, limit: u64, label: &str) -> Result<Vec<u8>, String> {
    use std::io::Read as _;
    let file = std::fs::File::open(path).map_err(|error| format!("open {label} {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read {label} {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).map_err(|_| format!("{label} byte count does not fit u64"))? > limit {
        return Err(format!("{label} exceeds its byte limit"));
    }
    Ok(bytes)
}

fn required_absolute_path(name: &str) -> Result<std::path::PathBuf, String> {
    let path =
        std::path::PathBuf::from(std::env::var_os(name).ok_or_else(|| format!("Hegel runtime is missing {name}"))?);
    if !path.is_absolute() {
        return Err(format!("Hegel runtime {name} is not absolute"));
    }
    Ok(path)
}

fn required_environment(name: &str) -> Result<std::ffi::OsString, String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("Hegel runtime is missing {name}"))
}
