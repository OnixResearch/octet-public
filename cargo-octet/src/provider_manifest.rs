use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::ProviderManifestArgs;
use crate::cli::ProviderManifestMode;

const SCHEMA: &str = "octet-compiler-policy-provider/v1";
const PROVIDER: &str = "octet";
const ADAPTER_KIND: &str = "dylint-rustc-driver";
const DEFAULT_PROFILE: &str = "artifact-required";
const DEFAULT_RUSTC: &str = "rustc";
const ENV_DYLINT_NO_DEPS: &str = "DYLINT_NO_DEPS";
const DYLINT_NO_DEPS_ENABLED: &str = "1";
const DIGEST_PREFIX: &str = "b3:";
const BLAKE3_HEX_CHARS: usize = 64;
const ENV_PATH: &str = "PATH";
const RUSTC_RELEASE_KEY: &str = "release";
const RUSTC_COMMIT_HASH_KEY: &str = "commit-hash";
const UNKNOWN_IDENTITY: &str = "unknown";
const LINT_LEVEL_ALLOW: &str = "allow";
const LINT_NAMESPACE_SEPARATOR: &str = "::";
#[cfg(unix)]
const EXECUTABLE_MODE_BITS: u32 = 0o111;
const REQUIRED_WAIVER_FIELDS: &[&str] = &[
    "policy",
    "source",
    "owner",
    "rationale",
    "review",
    "safer_default",
    "scope",
];
const DEFAULT_REQUIRED_LINTS: &[&str] = &[
    "ambient_clock",
    "ignored_result",
    "no_panic",
    "no_todo",
    "no_unwrap",
    "public_unsafe_api",
    "unchecked_division",
    "unchecked_narrowing",
    "unbounded_channel",
    "verified_purity",
];
const CLAIM_CONFIGURED_PROFILE_COMPLIANCE: &str = "configured-profile-compliance";
const NON_CLAIM_PROGRAM_CORRECTNESS: &str = "program-correctness";
const NON_CLAIM_COMPLETE_FCIS_PROOF: &str = "complete-fcis-proof";
const NON_CLAIM_GENERAL_SAFETY: &str = "general-safety";
const NON_CLAIM_ABSENCE_OF_ALL_DEFECTS: &str = "absence-of-all-defects";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderManifest {
    pub schema: String,
    pub provider: String,
    pub profile: String,
    pub adapter_kind: String,
    pub compatible_rustc: RustcIdentity,
    pub driver: ArtifactIdentity,
    pub lint_library: ArtifactIdentity,
    pub config: ArtifactIdentity,
    pub standards_policy: Option<ArtifactIdentity>,
    pub required_environment: BTreeMap<String, String>,
    pub required_lints: Vec<String>,
    pub disabled_lints: Vec<String>,
    pub waiver_contract: WaiverContract,
    pub claims: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RustcIdentity {
    pub executable: String,
    pub executable_blake3: Option<String>,
    pub version_verbose: String,
    pub release: String,
    pub commit_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub path: String,
    pub blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WaiverContract {
    pub syntax: Vec<String>,
    pub required_fields: Vec<String>,
    pub summary_shape: Vec<String>,
}

pub fn run(args: ProviderManifestArgs) -> Result<i32, String> {
    match args.mode {
        ProviderManifestMode::Export {
            driver,
            lint_library,
            config,
            standards_policy,
            rustc,
            profile,
            required_lints,
        } => {
            let manifest = build_manifest(ManifestInput {
                driver: &driver,
                lint_library: &lint_library,
                config: &config,
                standards_policy: standards_policy.as_deref(),
                rustc: rustc.as_deref().unwrap_or_else(|| Path::new(DEFAULT_RUSTC)),
                profile: profile.as_deref().unwrap_or(DEFAULT_PROFILE),
                required_lints: selected_required_lints(&required_lints),
            })?;
            validate_manifest(&manifest)?;
            print_json(&manifest)
        }
        ProviderManifestMode::Validate { manifest } => {
            let manifest = read_manifest(&manifest)?;
            validate_manifest(&manifest)?;
            println!("octet_provider_manifest.valid: PASS");
            Ok(0)
        }
    }
}

pub struct ManifestInput<'a> {
    pub driver: &'a Path,
    pub lint_library: &'a Path,
    pub config: &'a Path,
    pub standards_policy: Option<&'a Path>,
    pub rustc: &'a Path,
    pub profile: &'a str,
    pub required_lints: Vec<String>,
}

pub fn build_manifest(input: ManifestInput<'_>) -> Result<ProviderManifest, String> {
    let disabled_lints = disabled_lints_from_config(input.config)?;
    let mut required_environment = BTreeMap::new();
    required_environment.insert(ENV_DYLINT_NO_DEPS.to_owned(), DYLINT_NO_DEPS_ENABLED.to_owned());

    Ok(ProviderManifest {
        schema: SCHEMA.to_owned(),
        provider: PROVIDER.to_owned(),
        profile: input.profile.to_owned(),
        adapter_kind: ADAPTER_KIND.to_owned(),
        compatible_rustc: rustc_identity(input.rustc, std::env::var_os(ENV_PATH).as_deref())?,
        driver: artifact_identity(input.driver)?,
        lint_library: artifact_identity(input.lint_library)?,
        config: artifact_identity(input.config)?,
        standards_policy: input.standards_policy.map(artifact_identity).transpose()?,
        required_environment,
        required_lints: sorted_unique(input.required_lints.into_iter().map(canonical_required_lint).collect()),
        disabled_lints,
        waiver_contract: default_waiver_contract(),
        claims: vec![CLAIM_CONFIGURED_PROFILE_COMPLIANCE.to_owned()],
        non_claims: vec![
            NON_CLAIM_PROGRAM_CORRECTNESS.to_owned(),
            NON_CLAIM_COMPLETE_FCIS_PROOF.to_owned(),
            NON_CLAIM_GENERAL_SAFETY.to_owned(),
            NON_CLAIM_ABSENCE_OF_ALL_DEFECTS.to_owned(),
        ],
    })
}

pub fn validate_manifest(manifest: &ProviderManifest) -> Result<(), String> {
    let mut issues = Vec::new();
    require_equal(&mut issues, "schema", &manifest.schema, SCHEMA);
    require_equal(&mut issues, "provider", &manifest.provider, PROVIDER);
    require_equal(&mut issues, "adapter_kind", &manifest.adapter_kind, ADAPTER_KIND);
    require_non_empty(&mut issues, "profile", &manifest.profile);
    validate_rustc_identity(&manifest.compatible_rustc, &mut issues);
    validate_artifact("driver", &manifest.driver, &mut issues);
    validate_artifact("lint_library", &manifest.lint_library, &mut issues);
    validate_artifact("config", &manifest.config, &mut issues);
    if let Some(standards_policy) = &manifest.standards_policy {
        validate_artifact("standards_policy", standards_policy, &mut issues);
    }
    validate_environment(manifest, &mut issues);
    validate_required_lints(manifest, &mut issues);
    validate_waiver_contract(&manifest.waiver_contract, &mut issues);
    validate_claims(manifest, &mut issues);
    if issues.is_empty() {
        return Ok(());
    }
    Err(format!("invalid Octet provider manifest:\n- {}", issues.join("\n- ")))
}

fn artifact_identity(path: &Path) -> Result<ArtifactIdentity, String> {
    let bytes = fs::read(path).map_err(|error| format!("read artifact {}: {error}", path.display()))?;
    Ok(ArtifactIdentity {
        path: normalize_path(path),
        blake3: digest_bytes(&bytes),
    })
}

fn rustc_identity(path: &Path, search_path: Option<&std::ffi::OsStr>) -> Result<RustcIdentity, String> {
    let executable = resolve_executable(path, search_path)?;
    let output = Command::new(&executable)
        .arg("-vV")
        .output()
        .map_err(|error| format!("run {} -vV: {error}", executable.display()))?;
    if !output.status.success() {
        return Err(format!("{} -vV exited with {}", executable.display(), output.status));
    }
    let version_verbose =
        String::from_utf8(output.stdout).map_err(|error| format!("decode rustc -vV output: {error}"))?;
    let release = required_metadata_line(&version_verbose, RUSTC_RELEASE_KEY, &executable)?;
    let commit_hash = required_metadata_line(&version_verbose, RUSTC_COMMIT_HASH_KEY, &executable)?;
    let bytes =
        fs::read(&executable).map_err(|error| format!("read rustc executable {}: {error}", executable.display()))?;
    Ok(RustcIdentity {
        executable: normalize_path(&executable),
        executable_blake3: Some(digest_bytes(&bytes)),
        version_verbose: version_verbose.trim().to_owned(),
        release,
        commit_hash,
    })
}

/// Resolves the executable once the way `execvp` would (paths with a directory component are used
/// as given, bare names are searched in `search_path`, an empty entry meaning the current
/// directory) so the file that is run is the file that is hashed.
fn resolve_executable(path: &Path, search_path: Option<&std::ffi::OsStr>) -> Result<std::path::PathBuf, String> {
    if path.components().nth(1).is_some() {
        return Ok(path.to_path_buf());
    }
    search_path
        .into_iter()
        .flat_map(std::env::split_paths)
        .map(|directory| Path::new(".").join(directory).join(path))
        .find(|candidate| is_executable_file(candidate))
        .ok_or_else(|| format!("resolve {}: no executable file found in {ENV_PATH}", path.display()))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & EXECUTABLE_MODE_BITS != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// Collects every lint the selected config section turns off: entries in `disabled_lints` plus
/// lints whose `lint_levels` entry is `allow`. The section is chosen with the lint library's
/// precedence (`[octet]`, then the `[slotcar]` and `[tigerstyle]` compatibility sections).
fn disabled_lints_from_config(path: &Path) -> Result<Vec<String>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read config {}: {error}", path.display()))?;
    let document = text.parse::<toml::Table>().map_err(|error| format!("parse config {}: {error}", path.display()))?;
    let section = document
        .get("octet")
        .and_then(toml::Value::as_table)
        .or_else(|| document.get("slotcar").and_then(toml::Value::as_table))
        .or_else(|| document.get("tigerstyle").and_then(toml::Value::as_table));
    let mut disabled = section
        .and_then(|table| table.get("disabled_lints"))
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("disabled_lints in {} must contain only strings", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(levels) = section.and_then(|table| table.get("lint_levels")) {
        let levels = levels.as_table().ok_or_else(|| format!("lint_levels in {} must be a table", path.display()))?;
        for (lint, level) in levels {
            let level =
                level.as_str().ok_or_else(|| format!("lint_levels.{lint} in {} must be a string", path.display()))?;
            if level == LINT_LEVEL_ALLOW {
                disabled.push(lint.clone());
            }
        }
    }
    Ok(sorted_unique(disabled))
}

fn default_waiver_contract() -> WaiverContract {
    WaiverContract {
        syntax: vec![
            "octet-allow:".to_owned(),
            "slotcar-allow:".to_owned(),
            "tigerstyle-allow:".to_owned(),
        ],
        required_fields: REQUIRED_WAIVER_FIELDS.iter().map(|field| (*field).to_owned()).collect(),
        summary_shape: vec![
            "policy_id".to_owned(),
            "source_section".to_owned(),
            "owner".to_owned(),
            "rationale".to_owned(),
            "review_trigger".to_owned(),
            "safer_default".to_owned(),
            "scope".to_owned(),
            "path".to_owned(),
            "line".to_owned(),
        ],
    }
}

fn selected_required_lints(overrides: &[String]) -> Vec<String> {
    if overrides.is_empty() {
        return DEFAULT_REQUIRED_LINTS.iter().map(|lint| (*lint).to_owned()).collect();
    }
    overrides.to_vec()
}

fn read_manifest(path: &Path) -> Result<ProviderManifest, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read manifest {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse manifest {}: {error}", path.display()))
}

fn print_json(manifest: &ProviderManifest) -> Result<i32, String> {
    let rendered = serde_json::to_string_pretty(manifest).map_err(|error| format!("render manifest: {error}"))?;
    println!("{rendered}");
    Ok(0)
}

fn validate_rustc_identity(identity: &RustcIdentity, issues: &mut Vec<String>) {
    require_non_empty(issues, "compatible_rustc.executable", &identity.executable);
    require_non_empty(issues, "compatible_rustc.version_verbose", &identity.version_verbose);
    require_known(issues, "compatible_rustc.release", &identity.release);
    require_known(issues, "compatible_rustc.commit_hash", &identity.commit_hash);
    match &identity.executable_blake3 {
        Some(digest) => validate_digest("compatible_rustc.executable_blake3", digest, issues),
        None => issues.push("compatible_rustc.executable_blake3 must be present".to_owned()),
    }
}

fn validate_artifact(name: &str, artifact: &ArtifactIdentity, issues: &mut Vec<String>) {
    require_non_empty(issues, &format!("{name}.path"), &artifact.path);
    validate_digest(&format!("{name}.blake3"), &artifact.blake3, issues);
}

fn validate_environment(manifest: &ProviderManifest, issues: &mut Vec<String>) {
    match manifest.required_environment.get(ENV_DYLINT_NO_DEPS).map(String::as_str) {
        Some(DYLINT_NO_DEPS_ENABLED) => {}
        _ => issues.push(format!("required_environment.{ENV_DYLINT_NO_DEPS} must be `{DYLINT_NO_DEPS_ENABLED}`")),
    }
}

fn validate_required_lints(manifest: &ProviderManifest, issues: &mut Vec<String>) {
    if manifest.required_lints.is_empty() {
        issues.push("required_lints must not be empty for artifact profiles".to_owned());
    }
    for required in &manifest.required_lints {
        let Some(lint) = inventory_lint_name(required) else {
            issues.push(format!("required lint `{required}` is not an Octet lint"));
            continue;
        };
        if manifest.disabled_lints.iter().any(|disabled| disabled == lint) {
            issues.push(format!("required lint `{lint}` is disabled by config"));
        }
    }
}

/// Resolves a lint name to its Octet inventory name, accepting the bare name that config uses
/// and the namespaced tool-lint name (`tigerstyle::<lint>`) it is a compatibility alias for.
fn inventory_lint_name(name: &str) -> Option<&'static str> {
    let bare = name
        .strip_prefix(octet_lint_metadata::CANONICAL_NAMESPACE)
        .and_then(|rest| rest.strip_prefix(LINT_NAMESPACE_SEPARATOR))
        .unwrap_or(name);
    octet_lint_metadata::metadata_for_lint(bare).map(|metadata| metadata.lint_name)
}

/// Records known lints under the bare name config disables, so namespaced input cannot dodge the
/// disabled-lint check. Unknown names stay verbatim for validation to report.
fn canonical_required_lint(name: String) -> String {
    inventory_lint_name(&name).map_or(name, ToOwned::to_owned)
}

fn validate_waiver_contract(contract: &WaiverContract, issues: &mut Vec<String>) {
    for syntax in ["octet-allow:", "slotcar-allow:", "tigerstyle-allow:"] {
        if !contract.syntax.iter().any(|item| item == syntax) {
            issues.push(format!("waiver_contract.syntax missing `{syntax}`"));
        }
    }
    for field in REQUIRED_WAIVER_FIELDS {
        if !contract.required_fields.iter().any(|item| item == field) {
            issues.push(format!("waiver_contract.required_fields missing `{field}`"));
        }
    }
    for field in [
        "policy_id",
        "source_section",
        "owner",
        "review_trigger",
        "scope",
        "path",
        "line",
    ] {
        if !contract.summary_shape.iter().any(|item| item == field) {
            issues.push(format!("waiver_contract.summary_shape missing `{field}`"));
        }
    }
}

fn validate_claims(manifest: &ProviderManifest, issues: &mut Vec<String>) {
    if !manifest.claims.iter().any(|claim| claim == CLAIM_CONFIGURED_PROFILE_COMPLIANCE) {
        issues.push(format!("claims missing `{CLAIM_CONFIGURED_PROFILE_COMPLIANCE}`"));
    }
    for non_claim in [
        NON_CLAIM_PROGRAM_CORRECTNESS,
        NON_CLAIM_COMPLETE_FCIS_PROOF,
        NON_CLAIM_GENERAL_SAFETY,
        NON_CLAIM_ABSENCE_OF_ALL_DEFECTS,
    ] {
        if !manifest.non_claims.iter().any(|claim| claim == non_claim) {
            issues.push(format!("non_claims missing `{non_claim}`"));
        }
    }
}

fn validate_digest(field: &str, digest: &str, issues: &mut Vec<String>) {
    let Some(hex) = digest.strip_prefix(DIGEST_PREFIX) else {
        issues.push(format!("{field} must start with `{DIGEST_PREFIX}`"));
        return;
    };
    if hex.len() != BLAKE3_HEX_CHARS {
        issues.push(format!("{field} must contain {BLAKE3_HEX_CHARS} hex characters"));
        return;
    }
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        issues.push(format!("{field} must be lowercase or uppercase hexadecimal"));
    }
}

fn require_equal(issues: &mut Vec<String>, field: &str, actual: &str, expected: &str) {
    if actual != expected {
        issues.push(format!("{field} must be `{expected}`"));
    }
}

fn require_non_empty(issues: &mut Vec<String>, field: &str, actual: &str) {
    if actual.trim().is_empty() {
        issues.push(format!("{field} must not be empty"));
    }
}

/// Like [`require_non_empty`], but also rejects the `unknown` placeholder, which does not bind an
/// identity.
fn require_known(issues: &mut Vec<String>, field: &str, actual: &str) {
    require_non_empty(issues, field, actual);
    if actual.trim() == UNKNOWN_IDENTITY {
        issues.push(format!("{field} must not be `{UNKNOWN_IDENTITY}`"));
    }
}

fn metadata_line<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| line.strip_prefix(key)?.trim_start_matches(':').trim().into())
}

fn required_metadata_line(text: &str, key: &str, executable: &Path) -> Result<String, String> {
    metadata_line(text, key)
        .filter(|value| !value.is_empty() && *value != UNKNOWN_IDENTITY)
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("{} -vV did not report a known `{key}:` line", executable.display()))
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}

fn normalize_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{DIGEST_PREFIX}{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    const FAKE_DIGEST: &str = "b3:0000000000000000000000000000000000000000000000000000000000000000";

    fn artifact(path: &str) -> ArtifactIdentity {
        ArtifactIdentity {
            path: path.to_owned(),
            blake3: FAKE_DIGEST.to_owned(),
        }
    }

    fn valid_manifest() -> ProviderManifest {
        let mut required_environment = BTreeMap::new();
        required_environment.insert(ENV_DYLINT_NO_DEPS.to_owned(), DYLINT_NO_DEPS_ENABLED.to_owned());
        ProviderManifest {
            schema: SCHEMA.to_owned(),
            provider: PROVIDER.to_owned(),
            profile: DEFAULT_PROFILE.to_owned(),
            adapter_kind: ADAPTER_KIND.to_owned(),
            compatible_rustc: RustcIdentity {
                executable: "rustc".to_owned(),
                executable_blake3: Some(FAKE_DIGEST.to_owned()),
                version_verbose: "rustc 1.0.0\nrelease: 1.0.0\ncommit-hash: abc".to_owned(),
                release: "1.0.0".to_owned(),
                commit_hash: "abc".to_owned(),
            },
            driver: artifact("driver"),
            lint_library: artifact("liboctet.so"),
            config: artifact("dylint.toml"),
            standards_policy: Some(artifact("octet-policies.json")),
            required_environment,
            required_lints: vec!["no_unwrap".to_owned()],
            disabled_lints: Vec::new(),
            waiver_contract: default_waiver_contract(),
            claims: vec![CLAIM_CONFIGURED_PROFILE_COMPLIANCE.to_owned()],
            non_claims: vec![
                NON_CLAIM_PROGRAM_CORRECTNESS.to_owned(),
                NON_CLAIM_COMPLETE_FCIS_PROOF.to_owned(),
                NON_CLAIM_GENERAL_SAFETY.to_owned(),
                NON_CLAIM_ABSENCE_OF_ALL_DEFECTS.to_owned(),
            ],
        }
    }

    #[test]
    fn complete_manifest_validates() {
        let manifest = valid_manifest();
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn missing_rustc_identity_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.compatible_rustc.release.clear();
        let error = validate_manifest(&manifest).expect_err("missing rustc identity should fail");
        assert!(error.contains("compatible_rustc.release"));
        assert!(error.contains("must not be empty"));
    }

    #[test]
    fn missing_driver_identity_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.driver.path.clear();
        let error = validate_manifest(&manifest).expect_err("missing driver identity should fail");
        assert!(error.contains("driver.path"));
        assert!(error.contains("must not be empty"));
    }

    #[test]
    fn missing_lint_library_digest_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.lint_library.blake3 = "missing".to_owned();
        let error = validate_manifest(&manifest).expect_err("invalid digest should fail");
        assert!(error.contains("lint_library.blake3"));
        assert!(error.contains("must start"));
    }

    #[test]
    fn malformed_standards_policy_digest_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.standards_policy = Some(ArtifactIdentity {
            path: "octet-policies.json".to_owned(),
            blake3: "sha256:not-accepted".to_owned(),
        });
        let error = validate_manifest(&manifest).expect_err("invalid standards digest should fail");
        assert!(error.contains("standards_policy.blake3"));
        assert!(error.contains("must start"));
    }

    #[test]
    fn disabled_required_lint_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.disabled_lints = vec!["no_unwrap".to_owned()];
        let error = validate_manifest(&manifest).expect_err("disabled required lint should fail");
        assert!(error.contains("required lint `no_unwrap` is disabled"));
    }

    #[test]
    fn missing_waiver_metadata_field_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.waiver_contract.required_fields.retain(|field| field != "owner");
        let error = validate_manifest(&manifest).expect_err("missing waiver field should fail");
        assert!(error.contains("waiver_contract.required_fields missing `owner`"));
    }

    #[test]
    fn missing_waiver_summary_field_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.waiver_contract.summary_shape.retain(|field| field != "review_trigger");
        let error = validate_manifest(&manifest).expect_err("missing waiver summary field should fail");
        assert!(error.contains("waiver_contract.summary_shape missing `review_trigger`"));
    }

    #[test]
    fn config_disabled_lints_are_sorted_and_validated() {
        let temp = TempDir::new().expect("create temp dir");
        let config = temp.path().join("dylint.toml");
        fs::write(&config, "[octet]\ndisabled_lints = [\"z\", \"a\", \"a\"]\n").expect("write config");
        let disabled = disabled_lints_from_config(&config).expect("parse disabled lints");
        assert_eq!(disabled, vec!["a".to_owned(), "z".to_owned()]);
    }

    #[test]
    fn build_manifest_hashes_declared_artifacts() {
        let temp = TempDir::new().expect("create temp dir");
        let fixture = ManifestFixture::write(temp.path());
        let manifest = fixture.build_manifest();
        assert_ne!(manifest.driver.blake3, manifest.lint_library.blake3);
        assert!(manifest.standards_policy.is_some());
    }

    #[test]
    fn manifest_digest_changes_when_receipted_artifacts_change() {
        let temp = TempDir::new().expect("create temp dir");
        let fixture = ManifestFixture::write(temp.path());
        let original_digest = manifest_digest(&fixture.build_manifest());

        fs::write(&fixture.driver, "driver changed").expect("change driver");
        assert_ne!(original_digest, manifest_digest(&fixture.build_manifest()));
        fs::write(&fixture.driver, "driver").expect("restore driver");

        fs::write(&fixture.lint_library, "lint changed").expect("change lint library");
        assert_ne!(original_digest, manifest_digest(&fixture.build_manifest()));
        fs::write(&fixture.lint_library, "lint").expect("restore lint library");

        fs::write(&fixture.config, "[octet]\n# changed\n").expect("change config");
        assert_ne!(original_digest, manifest_digest(&fixture.build_manifest()));
        fs::write(&fixture.config, "[octet]\n").expect("restore config");

        fs::write(&fixture.standards, "{\"changed\":true}\n").expect("change standards");
        assert_ne!(original_digest, manifest_digest(&fixture.build_manifest()));
    }

    #[test]
    fn default_rustc_identity_hashes_the_executable_it_runs() {
        let search_path = std::env::var_os(ENV_PATH);
        let identity = rustc_identity(Path::new(DEFAULT_RUSTC), search_path.as_deref()).expect("rustc identity");
        let bytes = fs::read(&identity.executable).expect("read reported rustc executable");
        assert_eq!(identity.executable_blake3, Some(digest_bytes(&bytes)));
    }

    #[cfg(unix)]
    #[test]
    fn rustc_identity_skips_non_executable_path_entries_like_execvp() {
        let temp = TempDir::new().expect("create temp dir");
        let decoy = temp.path().join(DEFAULT_RUSTC);
        fs::write(&decoy, "not the rustc that runs").expect("write decoy rustc");
        let real_path = std::env::var_os(ENV_PATH).expect("PATH is set");
        let search_path =
            std::env::join_paths(std::iter::once(temp.path().to_path_buf()).chain(std::env::split_paths(&real_path)))
                .expect("join search path");

        let identity = rustc_identity(Path::new(DEFAULT_RUSTC), Some(&search_path)).expect("rustc identity");

        assert_ne!(identity.executable, normalize_path(&decoy));
        assert_ne!(identity.executable_blake3, Some(digest_bytes(b"not the rustc that runs")));
        let bytes = fs::read(&identity.executable).expect("read reported rustc executable");
        assert_eq!(identity.executable_blake3, Some(digest_bytes(&bytes)));
    }

    #[test]
    fn unresolvable_rustc_fails_closed() {
        let temp = TempDir::new().expect("create temp dir");
        let error = rustc_identity(Path::new(DEFAULT_RUSTC), Some(temp.path().as_os_str()))
            .expect_err("rustc missing from the search path should fail");
        assert!(error.contains("no executable file found in PATH"));
    }

    #[test]
    fn missing_rustc_executable_digest_fails_closed() {
        let mut manifest = valid_manifest();
        manifest.compatible_rustc.executable_blake3 = None;
        let error = validate_manifest(&manifest).expect_err("missing rustc digest should fail");
        assert!(error.contains("compatible_rustc.executable_blake3"));
    }

    #[test]
    fn placeholder_rustc_release_and_commit_fail_closed() {
        let mut manifest = valid_manifest();
        manifest.compatible_rustc.release = "unknown".to_owned();
        manifest.compatible_rustc.commit_hash = "unknown".to_owned();
        let error = validate_manifest(&manifest).expect_err("placeholder rustc identity should fail");
        assert!(error.contains("compatible_rustc.release"));
        assert!(error.contains("compatible_rustc.commit_hash"));
    }

    #[cfg(unix)]
    #[test]
    fn rustc_without_release_or_commit_metadata_fails_closed() {
        let search_path = std::env::var_os(ENV_PATH);
        let error = rustc_identity(Path::new("true"), search_path.as_deref())
            .expect_err("rustc -vV output without release metadata should fail");
        assert!(error.contains("release"));
    }

    #[test]
    fn config_lint_levels_allow_counts_as_disabled_in_the_selected_section() {
        let cases: &[(&str, &[&str])] = &[
            ("[octet.lint_levels]\nno_unwrap = \"allow\"\n", &["no_unwrap"]),
            ("[slotcar.lint_levels]\nno_unwrap = \"allow\"\n", &["no_unwrap"]),
            ("[tigerstyle.lint_levels]\nno_panic = \"allow\"\n", &["no_panic"]),
            ("[octet]\ndisabled_lints = [\"no_todo\"]\n\n[octet.lint_levels]\nno_unwrap = \"allow\"\n", &[
                "no_todo",
                "no_unwrap",
            ]),
            ("[octet]\n\n[slotcar.lint_levels]\nno_unwrap = \"allow\"\n", &[]),
            ("[octet.lint_levels]\nno_unwrap = \"warn\"\nno_panic = \"deny\"\n", &[]),
        ];
        let temp = TempDir::new().expect("create temp dir");
        let config = temp.path().join("dylint.toml");
        for (text, expected) in cases {
            fs::write(&config, text).expect("write config");
            let disabled = disabled_lints_from_config(&config).expect("parse disabled lints");
            assert_eq!(disabled, expected.iter().map(|lint| (*lint).to_owned()).collect::<Vec<_>>(), "{text}");
        }
    }

    #[test]
    fn config_lint_level_allow_of_required_lint_fails_closed() {
        let temp = TempDir::new().expect("create temp dir");
        let fixture = ManifestFixture::write(temp.path());
        fs::write(&fixture.config, "[octet.lint_levels]\nno_unwrap = \"allow\"\n").expect("write config");
        let error = validate_manifest(&fixture.build_manifest()).expect_err("allowed required lint should fail");
        assert!(error.contains("required lint `no_unwrap` is disabled"));

        fs::write(&fixture.config, "[octet.lint_levels]\nno_unwrap = \"deny\"\n").expect("write config");
        assert!(validate_manifest(&fixture.build_manifest()).is_ok());
    }

    #[test]
    fn required_lints_must_name_inventory_lints() {
        let mut manifest = valid_manifest();
        manifest.required_lints = vec!["no_unwarp".to_owned()];
        let error = validate_manifest(&manifest).expect_err("unknown required lint should fail");
        assert!(error.contains("no_unwarp"));

        manifest.required_lints = DEFAULT_REQUIRED_LINTS.iter().map(|lint| (*lint).to_owned()).collect();
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn namespaced_required_lint_resolves_to_the_disableable_lint() {
        let temp = TempDir::new().expect("create temp dir");
        let fixture = ManifestFixture::write(temp.path());
        let manifest = fixture.build_manifest_requiring(&["tigerstyle::no_unwrap"]);
        assert_eq!(manifest.required_lints, vec!["no_unwrap".to_owned()]);
        assert!(validate_manifest(&manifest).is_ok());

        fs::write(&fixture.config, "[octet]\ndisabled_lints = [\"no_unwrap\"]\n").expect("write config");
        let error = validate_manifest(&fixture.build_manifest_requiring(&["tigerstyle::no_unwrap"]))
            .expect_err("disabled namespaced required lint should fail");
        assert!(error.contains("required lint `no_unwrap` is disabled"));
    }

    struct ManifestFixture {
        driver: std::path::PathBuf,
        lint_library: std::path::PathBuf,
        config: std::path::PathBuf,
        standards: std::path::PathBuf,
    }

    impl ManifestFixture {
        fn write(root: &Path) -> Self {
            let fixture = Self {
                driver: root.join("driver"),
                lint_library: root.join("liboctet.so"),
                config: root.join("dylint.toml"),
                standards: root.join("octet-policies.json"),
            };
            fs::write(&fixture.driver, "driver").expect("write driver");
            fs::write(&fixture.lint_library, "lint").expect("write lint library");
            fs::write(&fixture.config, "[octet]\n").expect("write config");
            fs::write(&fixture.standards, "{}\n").expect("write standards");
            fixture
        }

        fn build_manifest(&self) -> ProviderManifest {
            self.build_manifest_requiring(&["no_unwrap"])
        }

        fn build_manifest_requiring(&self, required_lints: &[&str]) -> ProviderManifest {
            build_manifest(ManifestInput {
                driver: &self.driver,
                lint_library: &self.lint_library,
                config: &self.config,
                standards_policy: Some(&self.standards),
                rustc: Path::new(DEFAULT_RUSTC),
                profile: DEFAULT_PROFILE,
                required_lints: required_lints.iter().map(|lint| (*lint).to_owned()).collect(),
            })
            .expect("build manifest")
        }
    }

    fn manifest_digest(manifest: &ProviderManifest) -> String {
        let json = serde_json::to_vec(manifest).expect("serialize manifest");
        digest_bytes(&json)
    }
}
