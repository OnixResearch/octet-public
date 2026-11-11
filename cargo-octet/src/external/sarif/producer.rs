// r[impl octet.external_lint.sarif_producer_execution]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

#[cfg(test)]
use super::Evaluation;
use super::PROFILE_IDENTITY;
use super::evaluate as evaluate_import;
use super::prefixed_identity;
use super::selected_profile as selected_intake_profile;

/// Schema of the checked Nickel producer profile registry.
pub(crate) const PROFILE_SCHEMA: &str = "octet-sarif-producer-profiles/v1";
/// BLAKE3 identity of the checked `standards/sarif/producers.json` bytes.
pub(crate) const PROFILE_IDENTITY_VALUE: &str = "b3:bc12c39f5fdb66607f0acd91f00cd72785f75f526a4f5504d4e2e847492b94cd";
/// Versioned schema of one Octet-owned producer execution receipt.
pub(crate) const EXECUTION_SCHEMA: &str = "octet-sarif-producer-execution/v1";
/// Artifact that carries the execution receipt next to the raw report and projection.
pub(crate) const EXECUTION_ARTIFACT: &str = "sarif-producer-execution.json";
/// Artifact directory prefix, relative to the workspace root, that holds producer staging state.
pub(crate) const STAGE_ROOT: &str = ".octet/sarif-producer";
const REPORT_FILE_NAME: &str = "report.sarif";
const DATABASE_DIR_NAME: &str = "codeql-database";
const DATABASE_BINDING_FILE_NAME: &str = "octet-sarif-database-binding.json";
const EVIDENCE_ORIGIN_EXECUTED: &str = "octet_executed";
pub(crate) const OUTCOME_COMPLETED: &str = "completed";
pub(crate) const OUTCOME_FAILED: &str = "failed";
pub(crate) const OUTCOME_UNSUPPORTED: &str = "unsupported";
pub(crate) const OUTCOME_TOOL_ERROR: &str = "tool_error";
pub(crate) const OUTCOME_TIMEOUT: &str = "timeout";
pub(crate) const OUTCOME_INCOMPLETE: &str = "incomplete_analysis";
const STATUS_SUCCEEDED: &str = "succeeded";
const STATUS_FAILED: &str = "failed";
const STATUS_REUSED: &str = "reused";
const PRODUCER_SEMGREP: &str = "semgrep";
const PRODUCER_CODEQL: &str = "codeql";
const LOCAL_RULE_FILE: &str = "local-rule-file";
const QUERY_PACK_SET: &str = "query-pack-set";
const IDENTITY_SOURCE_CHECKED_BYTES: &str = "checked-file-bytes";
const IDENTITY_SOURCE_DECLARED_PIN: &str = "declared-pin";
const DATABASE_POLICY_REUSE_OR_CREATE: &str = "reuse-or-create";
const BUILD_MODE_SOURCE_ONLY: &str = "source-only";
const BUILD_MODE_NONE: &str = "none";
const SNAPSHOT_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-source-snapshot-v1\0";
const QUERY_PACK_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-codeql-query-pack-v1\0";
const QUERY_PACK_SET_IDENTITY_DOMAIN: &[u8] = b"octet-sarif-codeql-pack-set-v1\0";
pub(crate) const MAX_EXECUTABLE_BYTES: u64 = 512 * 1_024 * 1_024;
const EXIT_SUCCESS: i32 = 0;

const PROFILE_JSON: &str = include_str!("../../../../standards/sarif/producers.json");

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerRegistry {
    schema_version: String,
    profiles: Vec<ProducerProfile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerExecutable {
    pub name: String,
    pub declared_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerRuleSource {
    pub kind: String,
    pub path: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerQueryPack {
    pub name: String,
    pub version: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerDatabase {
    pub language: String,
    pub build_mode: String,
    pub policy: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerScope {
    pub paths: Vec<String>,
    pub extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
    pub max_files: usize,
    pub max_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerRustScope {
    pub build_mode: String,
    pub target: String,
    pub features: Vec<String>,
    pub packages: Vec<String>,
    pub unsupported_features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerLimits {
    pub wall_clock_ms: u64,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub max_report_bytes: u64,
    pub max_memory_mb: u64,
    pub max_threads: u64,
}

#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerProfile {
    pub profile_id: String,
    pub producer: String,
    pub intake_profile_id: String,
    pub driver_name: String,
    pub executable: ProducerExecutable,
    pub supported_languages: Vec<String>,
    pub rules: ProducerRuleSource,
    pub query_packs: Vec<ProducerQueryPack>,
    pub database: Option<ProducerDatabase>,
    pub scope: ProducerScope,
    pub rust: ProducerRustScope,
    pub limits: ProducerLimits,
    pub non_claims: Vec<String>,
}

/// One selected source file and its exact content identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct SourceFile {
    pub path: String,
    pub identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ScopeReceipt {
    pub declared_paths: Vec<String>,
    pub selected_paths: Vec<String>,
    pub missing_paths: Vec<String>,
    pub extensions: Vec<String>,
    pub exclude_globs: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RustScopeReceipt {
    pub build_mode: String,
    pub target: String,
    pub features: Vec<String>,
    pub packages: Vec<String>,
    pub unsupported_features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct LimitReceipt {
    pub wall_clock_ms: u64,
    pub stdout_max_bytes: usize,
    pub stderr_max_bytes: usize,
    pub max_report_bytes: u64,
    pub max_memory_mb: u64,
    pub max_threads: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ExecutableReceipt {
    pub name: String,
    pub declared_version: String,
    pub observed_version: String,
    pub resolved_path: String,
    pub identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct RuleReceipt {
    pub kind: String,
    pub path: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryPackReceipt {
    pub name: String,
    pub version: String,
    pub identity: String,
    pub identity_source: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct StepReceipt {
    pub step: String,
    pub argv: Vec<String>,
    pub exit_code: Option<i32>,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ReportReceipt {
    pub bytes: u64,
    pub identity: String,
}

/// The database facts a CodeQL analysis is bound to. A database whose recorded facts disagree with
/// the current source snapshot, build configuration, query packs, or executable is stale.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DatabaseBinding {
    pub source_snapshot_identity: String,
    pub language: String,
    pub build_mode: String,
    pub query_packs: Vec<QueryPackReceipt>,
    pub executable_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct DatabaseReceipt {
    pub path: String,
    pub created: bool,
    pub binding: DatabaseBinding,
    pub binding_identity: String,
    pub creation: Option<StepReceipt>,
    pub analysis: Option<StepReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ProducerExecution {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub producer: String,
    pub driver_name: String,
    pub intake_profile_id: String,
    pub intake_profile_identity: String,
    pub evidence_origin: String,
    pub evidence_tier: String,
    pub supported_languages: Vec<String>,
    pub executable: Option<ExecutableReceipt>,
    pub rules: RuleReceipt,
    pub query_packs: Vec<QueryPackReceipt>,
    pub scope: ScopeReceipt,
    pub rust: RustScopeReceipt,
    pub source_snapshot_identity: String,
    pub source_files: Vec<SourceFile>,
    pub limits: LimitReceipt,
    pub steps: Vec<StepReceipt>,
    pub database: Option<DatabaseReceipt>,
    pub report: Option<ReportReceipt>,
    pub outcome: String,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceSnapshot {
    pub files: Vec<SourceFile>,
    pub identity: String,
    pub selected_paths: Vec<String>,
    pub missing_paths: Vec<String>,
}

/// Resolved executable and its exact bytes identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableObservation {
    pub resolved_path: String,
    pub identity: String,
}

/// One completed child process observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StepOutput {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FailureKind {
    /// The executable name does not resolve on the declared search path.
    NotFound,
    /// The process could not be spawned or its executable could not be inspected.
    Spawn,
    /// The process outlived the declared wall-clock deadline.
    TimedOut,
    /// The process wrote more than the declared per-stream capture limit.
    OutputLimitExceeded,
    /// The process was cancelled.
    Cancelled,
    /// A signal terminated the process before it exited.
    Terminated,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StepFailure {
    pub kind: FailureKind,
    pub message: String,
}

impl StepFailure {
    fn new(kind: FailureKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Process mechanics the producer shell needs. The production implementation is Bounded Exec; tests
/// supply a deliberately labeled fixture that emits fixed SARIF because the pinned real tools are
/// absent from this environment.
pub(crate) trait Executor {
    fn resolve(&self, name: &str, current_dir: &Path) -> Result<ExecutableObservation, StepFailure>;
    fn run(&self, argv: &[String], current_dir: &Path, limits: &ProducerLimits) -> Result<StepOutput, StepFailure>;
}

pub(crate) struct ExecuteRequest<'a> {
    pub workspace_root: &'a Path,
    pub stage_dir: &'a Path,
}

pub(crate) fn registry() -> Result<ProducerRegistry, String> {
    let registry: ProducerRegistry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded SARIF producer profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded SARIF producer profile registry schema is unsupported".to_owned());
    }
    Ok(registry)
}

pub(crate) fn selected_profile(profile_id: &str) -> Result<ProducerProfile, String> {
    let registry = registry()?;
    let profile = registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown SARIF producer profile `{profile_id}`"))?;
    let diagnostics = validate_profile(&profile);
    if !diagnostics.is_empty() {
        return Err(format!("SARIF producer profile `{profile_id}` is invalid: {}", diagnostics.join("; ")));
    }
    Ok(profile)
}

pub(crate) fn profile_identity() -> &'static str {
    PROFILE_IDENTITY_VALUE
}

/// Whether the id names a checked producer execution profile. Unknown or malformed ids are not
/// producer profiles; callers then fall back to import-only handling, which rejects them.
pub(crate) fn is_producer_profile(profile_id: &str) -> Result<bool, String> {
    Ok(registry()?.profiles.iter().any(|profile| profile.profile_id == profile_id))
}

/// Map an intake or producer profile id to the SARIF intake profile that validates the report.
pub(crate) fn intake_profile_id(profile_id: &str) -> Result<String, String> {
    let registry = registry()?;
    if let Some(profile) = registry.profiles.iter().find(|profile| profile.profile_id == profile_id) {
        return Ok(profile.intake_profile_id.clone());
    }
    Ok(profile_id.to_owned())
}

/// The surrogate identity of one CodeQL query pack pin: `name` and declared `version` only. It is
/// not byte evidence for the locally resolved pack; the analysis report binding is.
pub(crate) fn query_pack_identity(name: &str, version: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(QUERY_PACK_IDENTITY_DOMAIN);
    hasher.update(&length_frame(name.len()));
    hasher.update(name.as_bytes());
    hasher.update(&length_frame(version.len()));
    hasher.update(version.as_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

pub(crate) fn query_pack_set_identity(packs: &[ProducerQueryPack]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(QUERY_PACK_SET_IDENTITY_DOMAIN);
    for pack in packs {
        hasher.update(&length_frame(pack.name.len()));
        hasher.update(pack.name.as_bytes());
        hasher.update(&length_frame(pack.version.len()));
        hasher.update(pack.version.as_bytes());
    }
    format!("b3:{}", hasher.finalize().to_hex())
}

fn length_frame(length: usize) -> [u8; 8] {
    u64::try_from(length).unwrap_or(u64::MAX).to_le_bytes()
}

pub(crate) fn execution_bytes(execution: &ProducerExecution) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(execution)
        .map_err(|error| format!("serialize SARIF producer execution receipt: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(crate) fn execution_identity(execution: &ProducerExecution) -> Result<String, String> {
    Ok(prefixed_identity(&execution_bytes(execution)?))
}

#[allow(clippy::too_many_lines)]
pub(crate) fn validate_profile(profile: &ProducerProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.profile_id.trim().is_empty() {
        diagnostics.push("SARIF producer profile_id must not be empty".to_owned());
    }
    if !matches!(profile.producer.as_str(), PRODUCER_SEMGREP | PRODUCER_CODEQL) {
        diagnostics.push(format!(
            "SARIF producer `{}` is unsupported; expected `{PRODUCER_SEMGREP}` or `{PRODUCER_CODEQL}`",
            profile.producer
        ));
    }
    if profile.executable.name != profile.producer {
        diagnostics.push("SARIF producer executable.name must equal the declared producer".to_owned());
    }
    if profile.executable.declared_version.trim().is_empty() || !profile.executable.declared_version.contains('.') {
        diagnostics.push("SARIF producer executable.declared_version must be a pinned version".to_owned());
    }
    if profile.supported_languages.len() != 1 || profile.supported_languages[0] != "rust" {
        diagnostics.push("SARIF producer supported_languages must be exactly [\"rust\"]".to_owned());
    }
    match selected_intake_profile(&profile.intake_profile_id) {
        Ok(intake) if intake.driver_name == profile.driver_name => {}
        Ok(_) => diagnostics.push("SARIF producer driver_name disagrees with its intake profile".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    if profile.non_claims.is_empty() {
        diagnostics.push("SARIF producer profile is missing non-claims".to_owned());
    }
    validate_rule_source(profile, &mut diagnostics);
    validate_scope(profile, &mut diagnostics);
    validate_limits(profile, &mut diagnostics);
    validate_producer_shape(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_rule_source(profile: &ProducerProfile, diagnostics: &mut Vec<String>) {
    if profile.rules.identity_source != IDENTITY_SOURCE_CHECKED_BYTES
        && profile.rules.identity_source != IDENTITY_SOURCE_DECLARED_PIN
    {
        diagnostics.push("SARIF producer rule identity_source is unsupported".to_owned());
    }
    match profile.rules.kind.as_str() {
        LOCAL_RULE_FILE => {
            if profile.rules.identity_source != IDENTITY_SOURCE_CHECKED_BYTES {
                diagnostics.push("a local SARIF producer rule file must bind checked file bytes".to_owned());
            }
            if !is_safe_relative_path(&profile.rules.path) {
                diagnostics.push("SARIF producer rule file path must be workspace-relative".to_owned());
            }
        }
        QUERY_PACK_SET => {
            if profile.rules.identity_source != IDENTITY_SOURCE_DECLARED_PIN {
                diagnostics.push("a SARIF producer query pack set must record a declared pin".to_owned());
            }
            if !profile.rules.path.is_empty() {
                diagnostics.push("a SARIF producer query pack set must not declare a local rule path".to_owned());
            }
            if profile.query_packs.is_empty() {
                diagnostics.push("a SARIF producer query pack set requires at least one pack".to_owned());
            }
            if query_pack_set_identity(&profile.query_packs) != profile.rules.identity {
                diagnostics.push("SARIF producer rules identity does not match its query pack pins".to_owned());
            }
        }
        other => diagnostics.push(format!("SARIF producer rule kind `{other}` is unsupported")),
    }
    let mut names = BTreeSet::new();
    for pack in &profile.query_packs {
        if pack.name.trim().is_empty() || pack.version.trim().is_empty() {
            diagnostics.push("SARIF producer query pack needs a name and version".to_owned());
        }
        if pack.identity_source != IDENTITY_SOURCE_DECLARED_PIN {
            diagnostics.push("SARIF producer query pack identity_source must be `declared-pin`".to_owned());
        }
        if query_pack_identity(&pack.name, &pack.version) != pack.identity {
            diagnostics.push(format!("SARIF producer query pack `{}` identity is stale", pack.name));
        }
        if !names.insert(pack.name.clone()) {
            diagnostics.push(format!("duplicate SARIF producer query pack `{}`", pack.name));
        }
    }
}

fn validate_scope(profile: &ProducerProfile, diagnostics: &mut Vec<String>) {
    if profile.scope.paths.is_empty() {
        diagnostics.push("SARIF producer scope must select at least one path".to_owned());
    }
    for path in &profile.scope.paths {
        if !is_safe_relative_path(path) {
            diagnostics.push(format!("SARIF producer scope path `{path}` must be workspace-relative"));
        }
    }
    if profile.scope.extensions.is_empty() {
        diagnostics.push("SARIF producer scope must declare at least one extension".to_owned());
    }
    for extension in &profile.scope.extensions {
        if !extension.starts_with('.') || extension.len() < 2 {
            diagnostics.push(format!("SARIF producer scope extension `{extension}` must start with `.`"));
        }
    }
    for glob in &profile.scope.exclude_globs {
        if !is_valid_glob(glob) {
            diagnostics.push(format!("SARIF producer exclusion glob `{glob}` is unsupported"));
        }
    }
    if profile.scope.max_files == 0 || profile.scope.max_bytes == 0 {
        diagnostics.push("SARIF producer scope limits must be positive".to_owned());
    }
}

fn validate_limits(profile: &ProducerProfile, diagnostics: &mut Vec<String>) {
    if profile.limits.max_report_bytes != super::MAX_REPORT_BYTES {
        diagnostics.push("SARIF producer max_report_bytes must match the intake profile limit".to_owned());
    }
    for (label, value) in [
        ("wall_clock_ms", profile.limits.wall_clock_ms),
        ("stdout_max_bytes", u64::try_from(profile.limits.stdout_max_bytes).unwrap_or_default()),
        ("stderr_max_bytes", u64::try_from(profile.limits.stderr_max_bytes).unwrap_or_default()),
        ("max_memory_mb", profile.limits.max_memory_mb),
        ("max_threads", profile.limits.max_threads),
    ] {
        if value == 0 {
            diagnostics.push(format!("SARIF producer limit `{label}` must be positive"));
        }
    }
}

fn validate_producer_shape(profile: &ProducerProfile, diagnostics: &mut Vec<String>) {
    match profile.producer.as_str() {
        PRODUCER_SEMGREP => {
            if profile.rules.kind != LOCAL_RULE_FILE {
                diagnostics.push("Semgrep producer requires a checked local rule file".to_owned());
            }
            if profile.database.is_some() || !profile.query_packs.is_empty() {
                diagnostics.push("Semgrep producer must not declare a CodeQL database or query packs".to_owned());
            }
            if profile.rust.build_mode != BUILD_MODE_SOURCE_ONLY {
                diagnostics.push("Semgrep producer rust.build_mode must be `source-only`".to_owned());
            }
        }
        PRODUCER_CODEQL => {
            if profile.rules.kind != QUERY_PACK_SET {
                diagnostics.push("CodeQL producer requires a query pack set".to_owned());
            }
            match profile.database.as_ref() {
                Some(database) if database.language == "rust" && database.build_mode == BUILD_MODE_NONE => {
                    if database.policy != DATABASE_POLICY_REUSE_OR_CREATE {
                        diagnostics.push("CodeQL database policy is unsupported".to_owned());
                    }
                }
                Some(_) => diagnostics
                    .push("CodeQL producer database must use language `rust` and build mode `none`".to_owned()),
                None => diagnostics.push("CodeQL producer requires a database declaration".to_owned()),
            }
            if profile.rust.build_mode != BUILD_MODE_NONE {
                diagnostics.push("CodeQL producer rust.build_mode must be `none`".to_owned());
            }
        }
        _ => {}
    }
}

fn is_safe_relative_path(path: &str) -> bool {
    let candidate = Path::new(path);
    !path.is_empty()
        && !path.contains('\0')
        && !candidate.is_absolute()
        && candidate.components().all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn is_valid_glob(glob: &str) -> bool {
    !glob.is_empty()
        && !glob.contains('\0')
        && !glob.starts_with('/')
        && glob.split('/').all(|segment| !matches!(segment, "" | "." | ".."))
}

/// Matches one path against a glob where `**` spans path segments and `*`/`?` stay within one
/// segment. The matcher is intentionally small: producer exclusions use segment globs only.
fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    match_segments(&pattern, &path)
}

fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
    match pattern.split_first() {
        None => path.is_empty(),
        Some((&"**", rest)) => (0..=path.len()).any(|skip| match_segments(rest, &path[skip..])),
        Some((&segment, rest)) => match path.split_first() {
            Some((&path_segment, path_rest)) => match_segment(segment, path_segment) && match_segments(rest, path_rest),
            None => false,
        },
    }
}

fn match_segment(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let (mut pattern_index, mut value_index) = (0, 0);
    let (mut star, mut star_value) = (None, 0);
    while value_index < value.len() {
        match pattern.get(pattern_index) {
            Some(&b'*') => {
                star = Some(pattern_index);
                star_value = value_index;
                pattern_index += 1;
            }
            Some(&b'?') => {
                pattern_index += 1;
                value_index += 1;
            }
            Some(&literal) if literal == value[value_index] => {
                pattern_index += 1;
                value_index += 1;
            }
            _ => match star {
                Some(star_index) => {
                    pattern_index = star_index + 1;
                    star_value += 1;
                    value_index = star_value;
                }
                None => return false,
            },
        }
    }
    pattern[pattern_index..].iter().all(|byte| *byte == b'*')
}

pub(crate) fn snapshot_sources(profile: &ProducerProfile, workspace_root: &Path) -> Result<SourceSnapshot, String> {
    let mut files = BTreeMap::new();
    let mut selected_paths = Vec::new();
    let mut missing_paths = Vec::new();
    let mut total_bytes = 0u64;
    for declared in &profile.scope.paths {
        let absolute = workspace_root.join(declared);
        match std::fs::symlink_metadata(&absolute) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!("SARIF producer scope path `{declared}` is a symlink, which is not allowed"));
            }
            Ok(metadata) if metadata.is_dir() => {
                selected_paths.push(declared.clone());
                walk_source_tree(profile, workspace_root, Path::new(declared), &mut files, &mut total_bytes)?;
            }
            Ok(metadata) if metadata.is_file() => {
                if scope_includes(profile, Path::new(declared), declared) {
                    let bytes = read_bounded_file(&absolute, profile.scope.max_bytes)?;
                    total_bytes = total_bytes.saturating_add(u64::try_from(bytes.len()).unwrap_or_default());
                    files.insert(declared.clone(), prefixed_identity(&bytes));
                    selected_paths.push(declared.clone());
                }
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => missing_paths.push(declared.clone()),
            Err(error) => return Err(format!("inspect SARIF producer scope path `{declared}`: {error}")),
        }
    }
    if total_bytes > profile.scope.max_bytes {
        return Err(format!(
            "SARIF producer source snapshot exceeds the declared {} byte limit",
            profile.scope.max_bytes
        ));
    }
    if files.len() > profile.scope.max_files {
        return Err(format!(
            "SARIF producer source snapshot exceeds the declared {} file limit",
            profile.scope.max_files
        ));
    }
    let files = files.into_iter().map(|(path, identity)| SourceFile { path, identity }).collect::<Vec<_>>();
    Ok(SourceSnapshot {
        identity: source_snapshot_identity(&files)?,
        files,
        selected_paths,
        missing_paths,
    })
}

fn walk_source_tree(
    profile: &ProducerProfile,
    workspace_root: &Path,
    relative: &Path,
    files: &mut BTreeMap<String, String>,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let absolute = workspace_root.join(relative);
    let mut names = std::fs::read_dir(&absolute)
        .map_err(|error| format!("read SARIF producer scope directory {}: {error}", relative.display()))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read SARIF producer scope entry in {}: {error}", relative.display()))?;
    names.sort();
    for name in names {
        let child = relative.join(&name);
        let locator = child.to_string_lossy().replace('\\', "/");
        if profile.scope.exclude_globs.iter().any(|glob| glob_matches(glob, &locator)) {
            continue;
        }
        let metadata = std::fs::symlink_metadata(workspace_root.join(&child))
            .map_err(|error| format!("inspect SARIF producer source `{locator}`: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("SARIF producer source `{locator}` is a symlink, which is not allowed"));
        }
        if metadata.is_dir() {
            walk_source_tree(profile, workspace_root, &child, files, total_bytes)?;
        } else if metadata.is_file() && scope_includes(profile, &child, &locator) {
            let bytes = read_bounded_file(&workspace_root.join(&child), profile.scope.max_bytes)?;
            *total_bytes = total_bytes.saturating_add(u64::try_from(bytes.len()).unwrap_or_default());
            if *total_bytes > profile.scope.max_bytes {
                return Err(format!(
                    "SARIF producer source snapshot exceeds the declared {} byte limit",
                    profile.scope.max_bytes
                ));
            }
            files.insert(locator, prefixed_identity(&bytes));
            if files.len() > profile.scope.max_files {
                return Err(format!(
                    "SARIF producer source snapshot exceeds the declared {} file limit",
                    profile.scope.max_files
                ));
            }
        }
    }
    Ok(())
}

fn scope_includes(profile: &ProducerProfile, path: &Path, locator: &str) -> bool {
    profile.scope.extensions.iter().any(|extension| {
        path.extension()
            .is_some_and(|observed| observed.to_string_lossy() == extension.trim_start_matches('.'))
    }) && !profile.scope.exclude_globs.iter().any(|glob| glob_matches(glob, locator))
}

pub(crate) fn source_snapshot_identity(files: &[SourceFile]) -> Result<String, String> {
    let mut sorted = files.to_vec();
    sorted.sort_by(|left, right| left.path.cmp(&right.path));
    let mut hasher = blake3::Hasher::new();
    hasher.update(SNAPSHOT_IDENTITY_DOMAIN);
    for file in &sorted {
        hasher.update(&length_frame(file.path.len()));
        hasher.update(file.path.as_bytes());
        hasher.update(&length_frame(file.identity.len()));
        hasher.update(file.identity.as_bytes());
    }
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

/// Read at most `max_bytes` from a regular file. A path that is a symlink or not a regular file is
/// rejected, and an over-limit file fails closed instead of being truncated.
fn read_bounded_file(path: &Path, max_bytes: u64) -> Result<Vec<u8>, String> {
    use std::io::Read as _;

    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("inspect SARIF producer input {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("SARIF producer input {} must be a regular file", path.display()));
    }
    if metadata.len() > max_bytes {
        return Err(format!("SARIF producer input {} exceeds the {max_bytes} byte limit", path.display()));
    }
    let mut file =
        std::fs::File::open(path).map_err(|error| format!("open SARIF producer input {}: {error}", path.display()))?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or_default());
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("read SARIF producer input {}: {error}", path.display()))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(format!("SARIF producer input {} exceeds the {max_bytes} byte limit", path.display()));
    }
    Ok(bytes)
}

/// Read the report twice and require identical bytes. A scanner output swapped between the process
/// exit and the ingestion read is not evidence for this execution.
fn read_stable_report_with(path: &Path, max_bytes: u64, between: impl FnOnce()) -> Result<Vec<u8>, StepFailure> {
    let first = read_bounded_file(path, max_bytes).map_err(|error| StepFailure::new(FailureKind::Spawn, error))?;
    between();
    let second = read_bounded_file(path, max_bytes).map_err(|error| StepFailure::new(FailureKind::Spawn, error))?;
    if first != second {
        return Err(StepFailure::new(
            FailureKind::Spawn,
            "SARIF report changed between execution and ingestion; the retained bytes are not evidence",
        ));
    }
    Ok(second)
}

pub(crate) fn read_stable_report(path: &Path, max_bytes: u64) -> Result<Vec<u8>, StepFailure> {
    read_stable_report_with(path, max_bytes, || {})
}

pub(crate) fn version_argv(profile: &ProducerProfile) -> Vec<String> {
    match profile.producer.as_str() {
        PRODUCER_CODEQL => string_argv(&["codeql", "version", "--format=json"]),
        _ => string_argv(&["semgrep", "--version"]),
    }
}

fn semgrep_scan_argv(profile: &ProducerProfile, report_path: &Path, selected_paths: &[String]) -> Vec<String> {
    let mut argv = string_argv(&[
        "semgrep",
        "scan",
        "--config",
        profile.rules.path.as_str(),
        "--sarif",
        "--output",
        report_path.to_string_lossy().as_ref(),
        "--metrics=off",
        "--disable-version-check",
    ]);
    argv.push(format!("--jobs={}", profile.limits.max_threads));
    argv.push(format!("--max-memory={}", profile.limits.max_memory_mb));
    argv.extend(selected_paths.iter().cloned());
    for glob in &profile.scope.exclude_globs {
        argv.push("--exclude".to_owned());
        argv.push(glob.clone());
    }
    argv
}

fn string_argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn executable_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn stage_paths(stage_dir: &Path) -> (PathBuf, PathBuf) {
    (stage_dir.join(REPORT_FILE_NAME), stage_dir.join(DATABASE_DIR_NAME))
}

#[allow(clippy::too_many_lines)]
pub(crate) fn execute(
    profile: &ProducerProfile,
    request: &ExecuteRequest<'_>,
    executor: &dyn Executor,
) -> Result<(ProducerExecution, String), String> {
    let (report_path, database_path) = stage_paths(request.stage_dir);
    let report_path = report_path.as_path();
    std::fs::create_dir_all(request.stage_dir)
        .map_err(|error| format!("create SARIF producer stage directory {}: {error}", request.stage_dir.display()))?;
    remove_if_present(report_path)?;

    let limits = limit_receipt(&profile.limits);
    let mut execution = ProducerExecution {
        schema_version: EXECUTION_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: PROFILE_IDENTITY_VALUE.to_owned(),
        producer: profile.producer.clone(),
        driver_name: profile.driver_name.clone(),
        intake_profile_id: profile.intake_profile_id.clone(),
        intake_profile_identity: PROFILE_IDENTITY.to_owned(),
        evidence_origin: EVIDENCE_ORIGIN_EXECUTED.to_owned(),
        evidence_tier: "static_lint".to_owned(),
        supported_languages: profile.supported_languages.clone(),
        executable: None,
        rules: rule_receipt(&profile.rules),
        query_packs: profile.query_packs.iter().map(query_pack_receipt).collect(),
        scope: ScopeReceipt {
            declared_paths: profile.scope.paths.clone(),
            selected_paths: Vec::new(),
            missing_paths: Vec::new(),
            extensions: profile.scope.extensions.clone(),
            exclude_globs: profile.scope.exclude_globs.clone(),
        },
        rust: RustScopeReceipt {
            build_mode: profile.rust.build_mode.clone(),
            target: profile.rust.target.clone(),
            features: profile.rust.features.clone(),
            packages: profile.rust.packages.clone(),
            unsupported_features: profile.rust.unsupported_features.clone(),
        },
        source_snapshot_identity: String::new(),
        source_files: Vec::new(),
        limits,
        steps: Vec::new(),
        database: None,
        report: None,
        outcome: OUTCOME_TOOL_ERROR.to_owned(),
        diagnostics: Vec::new(),
    };

    let snapshot = match snapshot_sources(profile, request.workspace_root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            execution.outcome = OUTCOME_UNSUPPORTED.to_owned();
            execution.diagnostics = vec![error];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    execution.scope.selected_paths = snapshot.selected_paths.clone();
    execution.scope.missing_paths = snapshot.missing_paths.clone();
    execution.source_snapshot_identity = snapshot.identity.clone();
    execution.source_files = snapshot.files.clone();
    if snapshot.files.is_empty() {
        execution.outcome = OUTCOME_UNSUPPORTED.to_owned();
        execution.diagnostics = vec![
            "no Rust source file matches the declared SARIF producer scope; an empty scan is not clean evidence"
                .to_owned(),
        ];
        return Ok((finish_diagnostics(execution), String::new()));
    }
    if let Err(error) = verify_rule_bytes(profile, request.workspace_root) {
        execution.outcome = OUTCOME_TOOL_ERROR.to_owned();
        execution.diagnostics = vec![error];
        return Ok((finish_diagnostics(execution), String::new()));
    }

    let executable = match executor.resolve(&profile.executable.name, request.workspace_root) {
        Ok(executable) => executable,
        Err(failure) => {
            execution.outcome = OUTCOME_UNSUPPORTED.to_owned();
            execution.diagnostics = vec![failure.message];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    let probe_argv = version_argv(profile);
    let probe = match executor.run(&probe_argv, request.workspace_root, &profile.limits) {
        Ok(output) => output,
        Err(failure) => {
            return Ok((failure_execution(execution, failure, "version", probe_argv), String::new()));
        }
    };
    let observed_version = match probe_version(profile, &probe) {
        Ok(version) => version,
        Err((kind, message)) => {
            execution.executable = Some(ExecutableReceipt {
                name: profile.executable.name.clone(),
                declared_version: profile.executable.declared_version.clone(),
                observed_version: String::new(),
                resolved_path: executable.resolved_path.clone(),
                identity: executable.identity.clone(),
            });
            execution.steps.push(StepReceipt {
                step: "version".to_owned(),
                argv: probe_argv,
                exit_code: Some(probe.exit_code),
                status: STATUS_FAILED.to_owned(),
            });
            execution.outcome = kind.to_owned();
            execution.diagnostics = vec![message];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    execution.executable = Some(ExecutableReceipt {
        name: profile.executable.name.clone(),
        declared_version: profile.executable.declared_version.clone(),
        observed_version: observed_version.clone(),
        resolved_path: executable.resolved_path.clone(),
        identity: executable.identity.clone(),
    });
    execution.steps.push(StepReceipt {
        step: "version".to_owned(),
        argv: probe_argv,
        exit_code: Some(probe.exit_code),
        status: STATUS_SUCCEEDED.to_owned(),
    });

    if profile.producer == PRODUCER_CODEQL {
        let database = profile.database.as_ref().ok_or_else(|| "CodeQL producer has no database".to_owned())?;
        let binding = DatabaseBinding {
            source_snapshot_identity: snapshot.identity.clone(),
            language: database.language.clone(),
            build_mode: database.build_mode.clone(),
            query_packs: profile.query_packs.iter().map(query_pack_receipt).collect(),
            executable_identity: executable.identity.clone(),
        };
        match prepare_database(profile, &database_path, &binding, request.workspace_root, executor) {
            Ok((receipt, Ok(()))) => execution.database = Some(receipt),
            Ok((receipt, Err((kind, message)))) => {
                execution.database = Some(receipt);
                execution.outcome = kind.to_owned();
                execution.diagnostics = vec![message];
                return Ok((finish_diagnostics(execution), String::new()));
            }
            Err(error) => {
                execution.outcome = OUTCOME_TOOL_ERROR.to_owned();
                execution.diagnostics = vec![error];
                return Ok((finish_diagnostics(execution), String::new()));
            }
        }
    }

    let analyze_argv = match profile.producer.as_str() {
        PRODUCER_CODEQL => codeql_analyze_argv(profile, &database_path, report_path),
        _ => semgrep_scan_argv(profile, report_path, &execution.scope.selected_paths),
    };
    remove_if_present(report_path)?;
    let scan = match executor.run(&analyze_argv, request.workspace_root, &profile.limits) {
        Ok(output) => output,
        Err(failure) => {
            return Ok((failure_execution(execution, failure, "analyze", analyze_argv), String::new()));
        }
    };
    let scan_step = StepReceipt {
        step: "analyze".to_owned(),
        argv: analyze_argv.clone(),
        exit_code: Some(scan.exit_code),
        status: if scan.exit_code == EXIT_SUCCESS {
            STATUS_SUCCEEDED.to_owned()
        } else {
            STATUS_FAILED.to_owned()
        },
    };
    execution.steps.push(scan_step);
    if let Some(database) = execution.database.as_mut() {
        database.analysis = execution.steps.last().cloned();
    }
    if scan.exit_code != EXIT_SUCCESS {
        execution.outcome = OUTCOME_TOOL_ERROR.to_owned();
        execution.diagnostics = vec![format!(
            "{} exited {}; no SARIF report from a failed scan is evidence",
            profile.driver_name, scan.exit_code
        )];
        return Ok((finish_diagnostics(execution), String::new()));
    }

    let report = match read_stable_report(report_path, profile.limits.max_report_bytes) {
        Ok(bytes) => bytes,
        Err(failure) => {
            execution.outcome = outcome_for_failure(failure.kind).to_owned();
            execution.diagnostics = vec![failure.message];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    if report.is_empty() {
        execution.outcome = OUTCOME_INCOMPLETE.to_owned();
        execution.diagnostics = vec!["the scanner exited successfully but wrote an empty SARIF report".to_owned()];
        return Ok((finish_diagnostics(execution), String::new()));
    }
    let report_text = match String::from_utf8(report) {
        Ok(text) => text,
        Err(error) => {
            execution.outcome = OUTCOME_FAILED.to_owned();
            execution.diagnostics = vec![format!("SARIF producer report is not UTF-8: {error}")];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    execution.report = Some(ReportReceipt {
        bytes: u64::try_from(report_text.len()).unwrap_or(u64::MAX),
        identity: prefixed_identity(report_text.as_bytes()),
    });
    if let Err(error) = verify_report_producer_binding(&report_text, profile, &observed_version) {
        execution.outcome = OUTCOME_FAILED.to_owned();
        execution.diagnostics = vec![error];
        return Ok((finish_diagnostics(execution), String::new()));
    }
    let evaluation = evaluate_import(&report_text, &profile.intake_profile_id);
    let Some(details) = evaluation.details.as_ref() else {
        execution.outcome = OUTCOME_FAILED.to_owned();
        execution.diagnostics = evaluation.diagnostics;
        return Ok((finish_diagnostics(execution), String::new()));
    };
    let selected: BTreeSet<&str> = snapshot.files.iter().map(|file| file.path.as_str()).collect();
    let outside = details
        .findings
        .iter()
        .filter(|finding| !selected.contains(finding.file.as_str()))
        .map(|finding| finding.file.as_str())
        .collect::<BTreeSet<_>>();
    if !outside.is_empty() {
        execution.outcome = OUTCOME_FAILED.to_owned();
        execution.diagnostics = vec![format!(
            "SARIF report references files outside the declared scope: {}",
            outside.into_iter().collect::<Vec<_>>().join(", ")
        )];
        return Ok((finish_diagnostics(execution), String::new()));
    }
    let after = match snapshot_sources(profile, request.workspace_root) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            execution.outcome = OUTCOME_FAILED.to_owned();
            execution.diagnostics = vec![error];
            return Ok((finish_diagnostics(execution), String::new()));
        }
    };
    if after.identity != snapshot.identity {
        execution.outcome = OUTCOME_FAILED.to_owned();
        execution.diagnostics =
            vec!["source changed while the scanner ran; the report is not bound to one snapshot".to_owned()];
        return Ok((finish_diagnostics(execution), String::new()));
    }
    execution.outcome = OUTCOME_COMPLETED.to_owned();
    Ok((execution, report_text))
}

fn failure_execution(
    mut execution: ProducerExecution,
    failure: StepFailure,
    step: &str,
    argv: Vec<String>,
) -> ProducerExecution {
    execution.steps.push(StepReceipt {
        step: step.to_owned(),
        argv,
        exit_code: None,
        status: STATUS_FAILED.to_owned(),
    });
    execution.outcome = outcome_for_failure(failure.kind).to_owned();
    execution.diagnostics = vec![failure.message];
    finish_diagnostics(execution)
}

fn outcome_for_failure(kind: FailureKind) -> &'static str {
    match kind {
        FailureKind::NotFound => OUTCOME_UNSUPPORTED,
        FailureKind::TimedOut => OUTCOME_TIMEOUT,
        FailureKind::OutputLimitExceeded => OUTCOME_INCOMPLETE,
        FailureKind::Cancelled | FailureKind::Terminated | FailureKind::Spawn => OUTCOME_TOOL_ERROR,
    }
}

fn finish_diagnostics(mut execution: ProducerExecution) -> ProducerExecution {
    execution.diagnostics.sort();
    execution.diagnostics.dedup();
    execution
}

fn remove_if_present(path: &Path) -> Result<(), String> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("remove stale SARIF producer report {}: {error}", path.display())),
    }
}

fn limit_receipt(limits: &ProducerLimits) -> LimitReceipt {
    LimitReceipt {
        wall_clock_ms: limits.wall_clock_ms,
        stdout_max_bytes: limits.stdout_max_bytes,
        stderr_max_bytes: limits.stderr_max_bytes,
        max_report_bytes: limits.max_report_bytes,
        max_memory_mb: limits.max_memory_mb,
        max_threads: limits.max_threads,
    }
}

fn rule_receipt(rules: &ProducerRuleSource) -> RuleReceipt {
    RuleReceipt {
        kind: rules.kind.clone(),
        path: rules.path.clone(),
        identity: rules.identity.clone(),
        identity_source: rules.identity_source.clone(),
    }
}

fn query_pack_receipt(pack: &ProducerQueryPack) -> QueryPackReceipt {
    QueryPackReceipt {
        name: pack.name.clone(),
        version: pack.version.clone(),
        identity: pack.identity.clone(),
        identity_source: pack.identity_source.clone(),
    }
}

fn verify_rule_bytes(profile: &ProducerProfile, workspace_root: &Path) -> Result<(), String> {
    if profile.rules.kind != LOCAL_RULE_FILE {
        return Ok(());
    }
    let bytes = read_bounded_file(&workspace_root.join(&profile.rules.path), profile.limits.max_report_bytes)?;
    if prefixed_identity(&bytes) != profile.rules.identity {
        return Err(format!(
            "SARIF producer rule bundle `{}` no longer matches its checked identity",
            profile.rules.path
        ));
    }
    Ok(())
}

fn probe_version(profile: &ProducerProfile, probe: &StepOutput) -> Result<String, (&'static str, String)> {
    if probe.exit_code != EXIT_SUCCESS {
        return Err((OUTCOME_TOOL_ERROR, format!("{} version probe exited {}", profile.driver_name, probe.exit_code)));
    }
    let stdout = String::from_utf8_lossy(&probe.stdout);
    let observed = match profile.producer.as_str() {
        PRODUCER_CODEQL => serde_json::from_str::<serde_json::Value>(stdout.trim())
            .ok()
            .and_then(|value| value.get("version").and_then(serde_json::Value::as_str).map(ToOwned::to_owned))
            .unwrap_or_default(),
        _ => stdout.trim().to_owned(),
    };
    if observed != profile.executable.declared_version {
        return Err((
            OUTCOME_TOOL_ERROR,
            format!(
                "{} is pinned to {} but the resolved executable reports `{observed}`",
                profile.driver_name, profile.executable.declared_version
            ),
        ));
    }
    Ok(observed)
}

fn codeql_create_argv(profile: &ProducerProfile, database_path: &Path, workspace_root: &Path) -> Vec<String> {
    let mut argv = string_argv(&["codeql", "database", "create"]);
    argv.push(executable_path_string(database_path));
    argv.push("--language=rust".to_owned());
    argv.push(format!("--build-mode={}", BUILD_MODE_NONE));
    argv.push(format!("--source-root={}", workspace_root.display()));
    argv.push(format!("--threads={}", profile.limits.max_threads));
    argv.push(format!("--ram={}", profile.limits.max_memory_mb));
    argv
}

fn codeql_analyze_argv(profile: &ProducerProfile, database_path: &Path, report_path: &Path) -> Vec<String> {
    let mut argv = string_argv(&["codeql", "database", "analyze"]);
    argv.push(executable_path_string(database_path));
    argv.push("--format=sarif-latest".to_owned());
    argv.push(format!("--output={}", report_path.display()));
    argv.push("--sarif-group-rules-by-pack".to_owned());
    argv.push(format!("--threads={}", profile.limits.max_threads));
    argv.push(format!("--ram={}", profile.limits.max_memory_mb));
    for pack in &profile.query_packs {
        argv.push(format!("{}@{}", pack.name, pack.version));
    }
    argv
}

type DatabaseStepResult = Result<(DatabaseReceipt, Result<(), (&'static str, String)>), String>;

fn prepare_database(
    profile: &ProducerProfile,
    database_path: &Path,
    binding: &DatabaseBinding,
    workspace_root: &Path,
    executor: &dyn Executor,
) -> DatabaseStepResult {
    let binding_path = database_path.join(DATABASE_BINDING_FILE_NAME);
    if database_path.exists() {
        let existing = read_database_binding(&binding_path)?;
        let Some((existing, existing_identity)) = existing else {
            return Ok((
                DatabaseReceipt {
                    path: executable_path_string(database_path),
                    created: false,
                    binding: binding.clone(),
                    binding_identity: String::new(),
                    creation: None,
                    analysis: None,
                },
                Err((
                    OUTCOME_FAILED,
                    "a CodeQL database exists without an Octet binding; it is stale or foreign".to_owned(),
                )),
            ));
        };
        if existing != *binding {
            return Ok((
                DatabaseReceipt {
                    path: executable_path_string(database_path),
                    created: false,
                    binding: binding.clone(),
                    binding_identity: existing_identity,
                    creation: None,
                    analysis: None,
                },
                Err((
                    OUTCOME_FAILED,
                    "the CodeQL database is stale: its source snapshot, build configuration, query packs, or executable differ"
                        .to_owned(),
                )),
            ));
        }
        return Ok((
            DatabaseReceipt {
                path: executable_path_string(database_path),
                created: false,
                binding: binding.clone(),
                binding_identity: existing_identity,
                creation: Some(StepReceipt {
                    step: "database_create".to_owned(),
                    argv: codeql_create_argv(profile, database_path, workspace_root),
                    exit_code: Some(EXIT_SUCCESS),
                    status: STATUS_REUSED.to_owned(),
                }),
                analysis: None,
            },
            Ok(()),
        ));
    }
    let argv = codeql_create_argv(profile, database_path, workspace_root);
    let output = executor.run(&argv, workspace_root, &profile.limits).map_err(|failure| failure.message)?;
    let creation = StepReceipt {
        step: "database_create".to_owned(),
        argv,
        exit_code: Some(output.exit_code),
        status: if output.exit_code == EXIT_SUCCESS {
            STATUS_SUCCEEDED.to_owned()
        } else {
            STATUS_FAILED.to_owned()
        },
    };
    let mut receipt = DatabaseReceipt {
        path: executable_path_string(database_path),
        created: true,
        binding: binding.clone(),
        binding_identity: String::new(),
        creation: Some(creation),
        analysis: None,
    };
    if output.exit_code != EXIT_SUCCESS {
        return Ok((
            receipt,
            Err((OUTCOME_TOOL_ERROR, format!("CodeQL database creation exited {}", output.exit_code))),
        ));
    }
    std::fs::create_dir_all(database_path)
        .map_err(|error| format!("create CodeQL database directory {}: {error}", database_path.display()))?;
    let bytes =
        serde_json::to_vec_pretty(binding).map_err(|error| format!("serialize CodeQL database binding: {error}"))?;
    std::fs::write(&binding_path, &bytes)
        .map_err(|error| format!("write CodeQL database binding {}: {error}", binding_path.display()))?;
    receipt.binding_identity = prefixed_identity(&bytes);
    Ok((receipt, Ok(())))
}

fn read_database_binding(path: &Path) -> Result<Option<(DatabaseBinding, String)>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("read CodeQL database binding {}: {error}", path.display())),
    };
    let identity = prefixed_identity(&bytes);
    let binding: DatabaseBinding = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse CodeQL database binding {}: {error}", path.display()))?;
    Ok(Some((binding, identity)))
}

fn verify_report_producer_binding(
    report: &str,
    profile: &ProducerProfile,
    observed_version: &str,
) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_str(report).map_err(|error| format!("SARIF producer report is malformed: {error}"))?;
    let run = value
        .get("runs")
        .and_then(serde_json::Value::as_array)
        .and_then(|runs| runs.first())
        .ok_or_else(|| "SARIF producer report has no run to bind".to_owned())?;
    let driver = run
        .pointer("/tool/driver")
        .ok_or_else(|| "SARIF producer report has no tool driver to bind".to_owned())?;
    if !driver
        .get("name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|name| super::driver_name_matches(&profile.driver_name, name))
    {
        return Err("SARIF producer report driver does not match the selected profile".to_owned());
    }
    let driver_version = driver
        .get("semanticVersion")
        .or_else(|| driver.get("version"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "SARIF producer report does not declare its driver version".to_owned())?;
    if !driver_version.starts_with(observed_version) {
        return Err(format!(
            "SARIF producer report driver version `{driver_version}` does not match the executed `{observed_version}`"
        ));
    }
    if profile.producer != PRODUCER_CODEQL {
        return Ok(());
    }
    let extensions = run
        .pointer("/tool/extensions")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);
    for pack in &profile.query_packs {
        let declared = extensions
            .iter()
            .find(|extension| extension.get("name").and_then(serde_json::Value::as_str) == Some(pack.name.as_str()));
        let Some(extension) = declared else {
            return Err(format!("SARIF producer report does not name the selected query pack `{}`", pack.name));
        };
        let version = extension.get("semanticVersion").and_then(serde_json::Value::as_str).unwrap_or_default();
        if !version.starts_with(&pack.version) {
            return Err(format!(
                "SARIF producer report names query pack `{}` at `{version}`, not the pinned `{}`",
                pack.name, pack.version
            ));
        }
    }
    Ok(())
}

/// Validate one execution receipt's internal binding and outcome consistency.
pub(crate) fn validate_execution(execution: &ProducerExecution) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if execution.schema_version != EXECUTION_SCHEMA {
        diagnostics.push("SARIF producer execution schema is unsupported".to_owned());
    }
    if execution.evidence_origin != EVIDENCE_ORIGIN_EXECUTED {
        diagnostics.push("SARIF producer execution must be labeled octet_executed".to_owned());
    }
    if execution.evidence_tier != "static_lint" {
        diagnostics.push("SARIF producer execution must retain the static_lint evidence tier".to_owned());
    }
    if execution.profile_identity != PROFILE_IDENTITY_VALUE {
        diagnostics.push("SARIF producer profile identity is stale".to_owned());
    }
    let profile = match selected_profile(&execution.profile_id) {
        Ok(profile) => Some(profile),
        Err(error) => {
            diagnostics.push(error);
            None
        }
    };
    if let Some(profile) = profile.as_ref()
        && (execution.producer != profile.producer
            || execution.driver_name != profile.driver_name
            || execution.intake_profile_id != profile.intake_profile_id
            || execution.intake_profile_identity != PROFILE_IDENTITY)
    {
        diagnostics.push("SARIF producer execution disagrees with its profile".to_owned());
    }
    if !matches!(
        execution.outcome.as_str(),
        OUTCOME_COMPLETED
            | OUTCOME_FAILED
            | OUTCOME_UNSUPPORTED
            | OUTCOME_TOOL_ERROR
            | OUTCOME_TIMEOUT
            | OUTCOME_INCOMPLETE
    ) {
        diagnostics.push("SARIF producer execution outcome is unsupported".to_owned());
    }
    if let Ok(observed) = source_snapshot_identity(&execution.source_files)
        && observed != execution.source_snapshot_identity
    {
        diagnostics.push("SARIF producer source snapshot identity is stale".to_owned());
    }
    if execution.outcome == OUTCOME_COMPLETED {
        if execution.report.is_none() {
            diagnostics.push("a completed SARIF producer execution must retain its report".to_owned());
        }
        if execution.scope.selected_paths.is_empty() || execution.source_files.is_empty() {
            diagnostics.push("a completed SARIF producer execution must establish its source scope".to_owned());
        }
        if execution.steps.last().is_none_or(|step| step.status != STATUS_SUCCEEDED) {
            diagnostics.push("a completed SARIF producer execution must end in a successful step".to_owned());
        }
        if execution.producer == PRODUCER_CODEQL
            && execution.database.as_ref().is_none_or(|database| database.analysis.is_none())
        {
            diagnostics.push("a completed CodeQL execution must record database analysis".to_owned());
        }
    }
    if let Some(database) = execution.database.as_ref() {
        if database.binding.source_snapshot_identity != execution.source_snapshot_identity {
            diagnostics.push("CodeQL database binding does not match the source snapshot".to_owned());
        }
        if let Some(profile) = profile.as_ref() {
            let expected = profile.query_packs.iter().map(query_pack_receipt).collect::<Vec<_>>();
            if database.binding.query_packs != expected {
                diagnostics.push("CodeQL database binding does not match the selected query packs".to_owned());
            }
        }
        if database.created && database.creation.as_ref().is_none_or(|creation| creation.status != STATUS_SUCCEEDED) {
            diagnostics.push("CodeQL database creation was not recorded as successful".to_owned());
        }
    }
    if let Some(report) = execution.report.as_ref() {
        if report.bytes == 0 {
            diagnostics.push("SARIF producer report must not be empty".to_owned());
        }
        if report.bytes > execution.limits.max_report_bytes {
            diagnostics.push("SARIF producer report exceeds the declared limit".to_owned());
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Re-validate an executed report against the existing intake profile and bind it to the execution
/// receipt through the raw report identity.
#[cfg(test)]
pub(crate) fn evaluate_executed(report: &str, execution: &ProducerExecution) -> Evaluation {
    let mut diagnostics = validate_execution(execution);
    if !diagnostics.is_empty() {
        return Evaluation {
            details: None,
            diagnostics,
        };
    }
    let Some(receipt) = execution.report.as_ref() else {
        diagnostics.push("SARIF producer execution did not retain a report".to_owned());
        return Evaluation {
            details: None,
            diagnostics,
        };
    };
    if receipt.identity != prefixed_identity(report.as_bytes()) {
        diagnostics.push("SARIF producer report bytes do not match the execution receipt".to_owned());
        return Evaluation {
            details: None,
            diagnostics,
        };
    }
    if execution.outcome != OUTCOME_COMPLETED {
        diagnostics.push(format!("SARIF producer execution outcome `{}` is not completed evidence", execution.outcome));
        return Evaluation {
            details: None,
            diagnostics,
        };
    }
    evaluate_import(report, &execution.intake_profile_id)
}

#[cfg(test)]
mod tests {
    include!("producer_tests.rs");
}
