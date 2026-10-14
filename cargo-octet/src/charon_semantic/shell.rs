use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use octet_standards::formal_proof::AeneasExecutionLimits;

use super::core;
use super::model::EXPECTED_PROFILE_IDENTITY;
use crate::aeneas_execution::ReusedCharonStageInput;

const SNAPSHOT_FILE_NAME: &str = "snapshot.llbc";
const WORK_DIRECTORY_PREFIX: &str = "octet-charon-semantic";
const WORK_ID_HEX_LENGTH: usize = 16;
const INTEGRATION_FAILURE_EXIT_CODE: i32 = 2;

pub(crate) struct CharonSemanticShellOutput {
    pub(crate) snapshot_text: String,
    pub(crate) exit_code: i32,
    pub(crate) diagnostics: Vec<String>,
}

pub(crate) fn execute_from_profile(workspace_root: &Path, profile_text: &str) -> CharonSemanticShellOutput {
    let profile = match core::parse_profile(profile_text) {
        Ok(profile) => profile,
        Err(error) => return failed_output(error),
    };
    let profile_diagnostics = core::validate_profile(&profile);
    if !profile_diagnostics.is_empty() {
        return CharonSemanticShellOutput {
            snapshot_text: String::new(),
            exit_code: INTEGRATION_FAILURE_EXIT_CODE,
            diagnostics: profile_diagnostics,
        };
    }
    if super::prefixed_identity(profile_text.as_bytes()) != EXPECTED_PROFILE_IDENTITY {
        return failed_output("Charon semantic profile bytes do not match the shipped profile identity".to_owned());
    }
    let canonical_workspace = match workspace_root.canonicalize() {
        Ok(path) => path,
        Err(error) => return failed_output(format!("canonicalize Charon semantic workspace root: {error}")),
    };
    let source_root = match canonical_workspace.join(&profile.source.root).canonicalize() {
        Ok(path) => path,
        Err(error) => return failed_output(format!("canonicalize Charon semantic source root: {error}")),
    };
    if !source_root.starts_with(&canonical_workspace) {
        return failed_output("Charon semantic source root escapes the workspace root".to_owned());
    }
    let charon_executable = match pinned_path(&profile.provider.executable_env, "Charon executable") {
        Ok(path) => path,
        Err(error) => return failed_output(error),
    };
    let network_sandbox = match pinned_path(&profile.provider.network_sandbox_env, "network sandbox") {
        Ok(path) => path,
        Err(error) => return failed_output(error),
    };
    let work_root = fresh_work_root(&source_root);
    let snapshot_path = PathBuf::from(SNAPSHOT_FILE_NAME);
    let observation = crate::aeneas_execution::execute_reused_charon_stage(ReusedCharonStageInput {
        charon_argv: profile.extraction.command_argv,
        charon_executable,
        network_sandbox,
        source_root,
        work_root: work_root.clone(),
        snapshot_path: snapshot_path.clone(),
        limits: AeneasExecutionLimits {
            timeout_ms: profile.limits.timeout_ms,
            stdout_max_bytes: profile.limits.stdout_max_bytes,
            stderr_max_bytes: profile.limits.stderr_max_bytes,
        },
        environment_allowlist: profile.extraction.environment_allowlist,
    });
    let mut diagnostics = observation.diagnostics;
    if observation.exit_code != Some(0) {
        diagnostics.push(format!("pinned Charon stage exited with {:?}", observation.exit_code));
    }
    if observation.timed_out {
        diagnostics.push("pinned Charon stage exceeded the configured timeout".to_owned());
    }
    if observation.stdout_truncated || observation.stderr_truncated {
        diagnostics.push("pinned Charon stage exceeded a configured log limit".to_owned());
    }
    let snapshot_file = work_root.join(snapshot_path);
    let snapshot_bytes = if diagnostics.is_empty() {
        match fs::read(&snapshot_file) {
            Ok(bytes) => bytes,
            Err(error) => {
                diagnostics.push(format!("read pinned Charon snapshot `{}`: {error}", snapshot_file.display()));
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };
    if snapshot_bytes.len() > profile.limits.snapshot_max_bytes {
        diagnostics.push(format!(
            "pinned Charon snapshot exceeds configured byte limit: {} > {}",
            snapshot_bytes.len(),
            profile.limits.snapshot_max_bytes
        ));
    }
    let snapshot_text = match String::from_utf8(snapshot_bytes) {
        Ok(text) => text,
        Err(error) => {
            diagnostics.push(format!("pinned Charon snapshot is not UTF-8 JSON: {error}"));
            String::new()
        }
    };
    if let Err(error) = fs::remove_dir_all(&work_root) {
        diagnostics.push(format!("remove Charon semantic work root `{}`: {error}", work_root.display()));
    }
    diagnostics.sort();
    diagnostics.dedup();
    CharonSemanticShellOutput {
        snapshot_text,
        exit_code: observation.exit_code.unwrap_or(INTEGRATION_FAILURE_EXIT_CODE),
        diagnostics,
    }
}

fn pinned_path(environment_name: &str, label: &str) -> Result<PathBuf, String> {
    let value =
        env::var_os(environment_name).ok_or_else(|| format!("missing pinned {label} path in `{environment_name}`"))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() || !path.is_file() {
        return Err(format!("pinned {label} path from `{environment_name}` is not an absolute file"));
    }
    Ok(path)
}

fn fresh_work_root(source_root: &Path) -> PathBuf {
    let seed = format!("{}:{}", source_root.display(), std::process::id());
    let identity = blake3::hash(seed.as_bytes()).to_hex().to_string();
    let short_identity = &identity[..WORK_ID_HEX_LENGTH];
    env::temp_dir().join(format!("{WORK_DIRECTORY_PREFIX}-{}-{short_identity}", std::process::id()))
}

fn failed_output(diagnostic: String) -> CharonSemanticShellOutput {
    CharonSemanticShellOutput {
        snapshot_text: String::new(),
        exit_code: INTEGRATION_FAILURE_EXIT_CODE,
        diagnostics: vec![diagnostic],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_provider_environment_fails_before_execution() {
        let result = pinned_path("OCTET_CHARON_SEMANTIC_MISSING_TEST", "test executable");
        assert!(result.is_err());
    }

    #[test]
    #[ignore = "requires the packaged Charon provider and Bubblewrap network sandbox"]
    fn live_charon_semantic_reuses_pinned_provider_and_replays() {
        const EXPECTED_PROJECTION_IDENTITY: &str =
            "b3:8e31a3a08cbe8b20c70906c47b2ddea301ca980bfcbd4feab0f547b63047fe90";
        const EXPECTED_ANALYSIS_IDENTITY: &str = "b3:42d37877986f3480e75e793c5e182f928eaf5ec256467ab074938afd3742dbac";
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-octet manifest must have a workspace parent");
        let first = execute_from_profile(workspace_root, super::super::PROFILE_JSON);
        assert_eq!(first.exit_code, 0, "{}", first.diagnostics.join("; "));
        assert!(first.diagnostics.is_empty());
        let second = execute_from_profile(workspace_root, super::super::PROFILE_JSON);
        assert_eq!(second.exit_code, 0, "{}", second.diagnostics.join("; "));
        assert_eq!(first.snapshot_text, second.snapshot_text);
        let envelope = super::super::build_envelope(first.snapshot_text, Vec::new()).expect("live envelope must build");
        let evaluation = super::super::evaluate_envelope(&envelope);
        assert!(evaluation.diagnostics.is_empty(), "{}", evaluation.diagnostics.join("; "));
        let details = evaluation.details.expect("live execution must produce details");
        assert_eq!(details.projection_identity, EXPECTED_PROJECTION_IDENTITY);
        assert_eq!(details.analysis_identity, EXPECTED_ANALYSIS_IDENTITY);
        assert_eq!(details.verification_role, super::super::model::RECORDED_ONLY_ROLE);
    }
}
