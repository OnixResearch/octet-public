//! Thin std shell for deterministic Nickel exports.

use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};

use serde::Serialize;

use nickel_export_core::{
    ArtifactMaterial, EvaluationObservation, EvaluatorDescriptor, ExportFormat, ExportManifest,
    ExportRequest, ImportPathPolicy, build_manifest, build_receipt, normalize_request,
    verify_manifest_fresh,
};

/// Stable non-zero process exit used for all fail-closed shell errors.
pub const FAILURE_EXIT_CODE: i32 = 2;
/// Structured shell failure schema.
pub const SHELL_ERROR_SCHEMA: &str = "onix-nickel-export-shell-error/v1";
const USAGE: &str = "usage: nickel-export export --spec <request.json> --root <dir> --evaluator <program> --evaluator-identity <identity> --evaluator-version <version> --manifest <relative-path> (--write|--check)";
const COMMAND_EXPORT: &str = "export";
const FIRST_OPTION_INDEX: usize = 2;
const FLAG_SPEC: &str = "--spec";
const FLAG_ROOT: &str = "--root";
const FLAG_EVALUATOR: &str = "--evaluator";
const FLAG_EVALUATOR_IDENTITY: &str = "--evaluator-identity";
const FLAG_EVALUATOR_VERSION: &str = "--evaluator-version";
const FLAG_MANIFEST: &str = "--manifest";
const FLAG_WRITE: &str = "--write";
const FLAG_CHECK: &str = "--check";
const NICKEL_PACKAGE_VERSION_PREFIX: &str = "nickel-lang-cli-";

/// Shell error with a stable stage classification.
#[derive(Debug, Serialize)]
pub struct ShellError {
    schema: &'static str,
    stage: &'static str,
    message: String,
}

impl ShellError {
    fn new(stage: &'static str, message: impl Into<String>) -> Self {
        Self {
            schema: SHELL_ERROR_SCHEMA,
            stage,
            message: message.into(),
        }
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.stage, self.message)
    }
}

impl std::error::Error for ShellError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Write,
    Check,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CliOptions {
    spec: PathBuf,
    root: PathBuf,
    evaluator: PathBuf,
    evaluator_identity: String,
    evaluator_version: String,
    manifest: PathBuf,
    mode: Mode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvaluationPlan {
    program: PathBuf,
    args: Vec<OsString>,
    current_dir: PathBuf,
}

/// Execute the CLI shell around the pure core.
///
/// # Errors
///
/// Returns a staged error for malformed arguments, unsafe paths, unreadable
/// inputs, evaluator failure, receipt rejection, stale checked-in artifacts,
/// serialization failure, or output-write failure.
pub fn run(args: &[String]) -> Result<(), ShellError> {
    let options = parse_args(args)?;
    execute(&options)
}

// r[impl nickel_export.shell.authority]
fn execute(options: &CliOptions) -> Result<(), ShellError> {
    let root = canonical_root(&options.root)?;
    let request_bytes = read_root_path(&root, &options.spec, "read-spec")?;
    let request: ExportRequest = serde_json::from_slice(&request_bytes).map_err(|error| {
        ShellError::new("parse-spec", format!("{}: {error}", options.spec.display()))
    })?;
    let request = normalize_request(&request)
        .map_err(|error| ShellError::new("validate-spec", error.to_string()))?;
    validate_shell_contract(&request)?;
    let source_bytes = read_root_file(&root, &request.source, "read-source")?;
    let dependency_bytes = request
        .dependencies
        .iter()
        .map(|path| {
            read_root_file(&root, path, "read-dependency").map(|bytes| (path.as_str(), bytes))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let plan = evaluation_plan(options, &request, &root);
    verify_evaluator_version(&plan.program, &options.evaluator_version)?;
    let output = run_evaluator(&plan)?;
    let evaluator = EvaluatorDescriptor {
        identity: options.evaluator_identity.clone(),
        version: options.evaluator_version.clone(),
        options: evaluator_options(&request),
        import_path_policy: ImportPathPolicy::DeclaredOnly,
    };
    let dependencies = dependency_bytes
        .iter()
        .map(|(path, bytes)| ArtifactMaterial {
            path,
            bytes: bytes.as_slice(),
        })
        .collect();
    let observation = EvaluationObservation {
        request: &request,
        source: ArtifactMaterial {
            path: &request.source,
            bytes: &source_bytes,
        },
        dependencies,
        output: ArtifactMaterial {
            path: &request.destination,
            bytes: &output.stdout,
        },
        evaluator: &evaluator,
        observed_dependencies: Vec::new(),
        diagnostics: Vec::new(),
    };
    let receipt = build_receipt(&observation)
        .map_err(|error| ShellError::new("admit-receipt", error.to_string()))?;
    let manifest = build_manifest(core::slice::from_ref(&receipt))
        .map_err(|error| ShellError::new("build-manifest", error.to_string()))?;
    match options.mode {
        Mode::Write => write_artifacts(
            &root,
            &request,
            &output.stdout,
            &options.manifest,
            &manifest,
        )?,
        Mode::Check => check_artifacts(
            &root,
            &request,
            &output.stdout,
            &options.manifest,
            &manifest,
        )?,
    }
    let rendered_receipt = serde_json::to_string(&receipt)
        .map_err(|error| ShellError::new("render-receipt", error.to_string()))?;
    println!("{rendered_receipt}");
    Ok(())
}

fn parse_args(args: &[String]) -> Result<CliOptions, ShellError> {
    if args.get(1).map(String::as_str) != Some(COMMAND_EXPORT) {
        return Err(ShellError::new("arguments", USAGE));
    }
    let mut spec = None;
    let mut root = None;
    let mut evaluator = None;
    let mut evaluator_identity = None;
    let mut evaluator_version = None;
    let mut manifest = None;
    let mut mode = None;
    let mut index = FIRST_OPTION_INDEX;
    while index < args.len() {
        let flag = &args[index];
        match flag.as_str() {
            FLAG_WRITE => set_mode(&mut mode, Mode::Write)?,
            FLAG_CHECK => set_mode(&mut mode, Mode::Check)?,
            FLAG_SPEC => spec = Some(PathBuf::from(next_value(args, &mut index, FLAG_SPEC)?)),
            FLAG_ROOT => root = Some(PathBuf::from(next_value(args, &mut index, FLAG_ROOT)?)),
            FLAG_EVALUATOR => {
                evaluator = Some(PathBuf::from(next_value(args, &mut index, FLAG_EVALUATOR)?));
            }
            FLAG_EVALUATOR_IDENTITY => {
                evaluator_identity =
                    Some(next_value(args, &mut index, FLAG_EVALUATOR_IDENTITY)?.to_string());
            }
            FLAG_EVALUATOR_VERSION => {
                evaluator_version =
                    Some(next_value(args, &mut index, FLAG_EVALUATOR_VERSION)?.to_string());
            }
            FLAG_MANIFEST => {
                manifest = Some(PathBuf::from(next_value(args, &mut index, FLAG_MANIFEST)?));
            }
            unknown => {
                return Err(ShellError::new(
                    "arguments",
                    format!("unknown argument `{unknown}`; {USAGE}"),
                ));
            }
        }
        index += 1;
    }
    let options = CliOptions {
        spec: required(spec, FLAG_SPEC)?,
        root: required(root, FLAG_ROOT)?,
        evaluator: required(evaluator, FLAG_EVALUATOR)?,
        evaluator_identity: required(evaluator_identity, FLAG_EVALUATOR_IDENTITY)?,
        evaluator_version: required(evaluator_version, FLAG_EVALUATOR_VERSION)?,
        manifest: required(manifest, FLAG_MANIFEST)?,
        mode: required(mode, "--write or --check")?,
    };
    require_nonempty(&options.evaluator_identity, FLAG_EVALUATOR_IDENTITY)?;
    require_nonempty(&options.evaluator_version, FLAG_EVALUATOR_VERSION)?;
    validate_relative_shell_path(&options.spec, FLAG_SPEC)?;
    validate_relative_shell_path(&options.manifest, FLAG_MANIFEST)?;
    Ok(options)
}

fn next_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag: &str,
) -> Result<&'a str, ShellError> {
    *index += 1;
    args.get(*index)
        .map(String::as_str)
        .ok_or_else(|| ShellError::new("arguments", format!("missing value for `{flag}`")))
}

fn set_mode(mode: &mut Option<Mode>, value: Mode) -> Result<(), ShellError> {
    if mode.replace(value).is_some() {
        return Err(ShellError::new(
            "arguments",
            "choose exactly one of --write or --check",
        ));
    }
    Ok(())
}

fn required<T>(value: Option<T>, flag: &str) -> Result<T, ShellError> {
    value.ok_or_else(|| {
        ShellError::new(
            "arguments",
            format!("missing required argument `{flag}`; {USAGE}"),
        )
    })
}

fn require_nonempty(value: &str, flag: &str) -> Result<(), ShellError> {
    if value.trim().is_empty() {
        Err(ShellError::new(
            "arguments",
            format!("`{flag}` must not be empty"),
        ))
    } else {
        Ok(())
    }
}

fn validate_shell_contract(request: &ExportRequest) -> Result<(), ShellError> {
    if request.contract.is_empty() {
        return Ok(());
    }
    validate_relative_shell_path(Path::new(&request.contract), "contract")?;
    if request.dependencies.contains(&request.contract) {
        Ok(())
    } else {
        Err(ShellError::new(
            "validate-spec",
            format!(
                "CLI contract file `{}` must also be declared as an exact dependency",
                request.contract
            ),
        ))
    }
}

fn evaluation_plan(options: &CliOptions, request: &ExportRequest, root: &Path) -> EvaluationPlan {
    let mut args = vec![
        OsString::from("export"),
        OsString::from("--format"),
        OsString::from(evaluator_format(request.format)),
        OsString::from(&request.source),
    ];
    for import_path in &request.import_paths {
        args.push(OsString::from("--import-path"));
        args.push(root.join(import_path).into_os_string());
    }
    if !request.selector.is_empty() {
        args.push(OsString::from("--field"));
        args.push(OsString::from(&request.selector));
    }
    if !request.contract.is_empty() {
        args.push(OsString::from("--apply-contract"));
        args.push(root.join(&request.contract).into_os_string());
    }
    EvaluationPlan {
        program: options.evaluator.clone(),
        args,
        current_dir: root.to_path_buf(),
    }
}

const fn evaluator_format(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Json => "json",
        ExportFormat::Toml => "toml",
        ExportFormat::Yaml => "yaml",
        ExportFormat::Raw => "text",
    }
}

fn evaluator_options(request: &ExportRequest) -> Vec<String> {
    let mut options = vec![format!("format={}", request.format.as_str())];
    options.extend(
        request
            .import_paths
            .iter()
            .map(|path| format!("import-path={path}")),
    );
    if !request.selector.is_empty() {
        options.push(format!("selector={}", request.selector));
    }
    if !request.contract.is_empty() {
        options.push(format!("contract={}", request.contract));
    }
    options
}

fn verify_evaluator_version(program: &Path, expected: &str) -> Result<(), ShellError> {
    let output = Command::new(program)
        .arg("--version")
        .output()
        .map_err(|error| ShellError::new("evaluator-version", error.to_string()))?;
    if !output.status.success() {
        return Err(ShellError::new(
            "evaluator-version",
            format!("version command failed with status {}", output.status),
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_token = expected
        .strip_prefix(NICKEL_PACKAGE_VERSION_PREFIX)
        .unwrap_or(expected);
    if stdout
        .split_whitespace()
        .any(|token| token == expected_token)
    {
        Ok(())
    } else {
        Err(ShellError::new(
            "evaluator-version",
            format!(
                "evaluator version output `{}` does not contain expected token `{expected_token}`",
                stdout.trim()
            ),
        ))
    }
}

fn run_evaluator(plan: &EvaluationPlan) -> Result<Output, ShellError> {
    let output = Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
        .map_err(|error| ShellError::new("evaluator-spawn", error.to_string()))?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ShellError::new(
            "evaluator-failure",
            format!("status {}: {stderr}", output.status),
        ))
    }
}

fn write_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &ExportManifest,
) -> Result<(), ShellError> {
    let manifest_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| ShellError::new("render-manifest", error.to_string()))?;
    write_root_file(
        root,
        Path::new(&request.destination),
        output,
        "write-output",
    )?;
    write_root_file(root, manifest_path, &manifest_bytes, "write-manifest")
}

fn check_artifacts(
    root: &Path,
    request: &ExportRequest,
    output: &[u8],
    manifest_path: &Path,
    manifest: &ExportManifest,
) -> Result<(), ShellError> {
    let checked_output = read_root_file(root, &request.destination, "check-output")?;
    if checked_output != output {
        return Err(ShellError::new(
            "check-output",
            format!("`{}` is stale", request.destination),
        ));
    }
    let checked_manifest_bytes = read_root_path(root, manifest_path, "check-manifest")?;
    let checked_manifest: ExportManifest = serde_json::from_slice(&checked_manifest_bytes)
        .map_err(|error| ShellError::new("check-manifest", error.to_string()))?;
    verify_manifest_fresh(&checked_manifest, manifest)
        .map_err(|error| ShellError::new("check-manifest", error.to_string()))
}

fn canonical_root(path: &Path) -> Result<PathBuf, ShellError> {
    path.canonicalize()
        .map_err(|error| ShellError::new("root", format!("{}: {error}", path.display())))
}

fn read_file(path: &Path, stage: &'static str) -> Result<Vec<u8>, ShellError> {
    fs::read(path).map_err(|error| ShellError::new(stage, format!("{}: {error}", path.display())))
}

fn read_root_file(root: &Path, path: &str, stage: &'static str) -> Result<Vec<u8>, ShellError> {
    read_root_path(root, Path::new(path), stage)
}

fn read_root_path(root: &Path, path: &Path, stage: &'static str) -> Result<Vec<u8>, ShellError> {
    validate_relative_shell_path(path, stage)?;
    let candidate = root.join(path);
    reject_symlink_components(root, path, stage)?;
    let canonical = candidate
        .canonicalize()
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", candidate.display())))?;
    if !canonical.starts_with(root) {
        return Err(ShellError::new(
            "unsafe-path",
            format!("{} escapes the repository root", candidate.display()),
        ));
    }
    read_file(&canonical, stage)
}

fn write_root_file(
    root: &Path,
    path: &Path,
    bytes: &[u8],
    stage: &'static str,
) -> Result<(), ShellError> {
    validate_relative_shell_path(path, stage)?;
    reject_symlink_components(root, path, stage)?;
    let destination = root.join(path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
        let canonical_parent = parent
            .canonicalize()
            .map_err(|error| ShellError::new(stage, format!("{}: {error}", parent.display())))?;
        if !canonical_parent.starts_with(root) {
            return Err(ShellError::new(
                "unsafe-path",
                format!("{} escapes the repository root", parent.display()),
            ));
        }
    }
    fs::write(&destination, bytes)
        .map_err(|error| ShellError::new(stage, format!("{}: {error}", destination.display())))
}

fn reject_symlink_components(
    root: &Path,
    path: &Path,
    stage: &'static str,
) -> Result<(), ShellError> {
    let mut candidate = root.to_path_buf();
    for component in path.components() {
        candidate.push(component);
        match fs::symlink_metadata(&candidate) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(ShellError::new(
                    "unsafe-path",
                    format!("{stage} rejects symlink component {}", candidate.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(ShellError::new(
                    stage,
                    format!("inspecting {}: {error}", candidate.display()),
                ));
            }
        }
    }
    Ok(())
}

fn validate_relative_shell_path(path: &Path, subject: &str) -> Result<(), ShellError> {
    if path.as_os_str().is_empty() || path.is_absolute() {
        return Err(ShellError::new(
            "unsafe-path",
            format!("`{subject}` must be repository-root-relative"),
        ));
    }
    for component in path.components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(ShellError::new(
                "unsafe-path",
                format!("`{subject}` must not escape the root"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    fn valid_args() -> Vec<String> {
        [
            "nickel-export",
            COMMAND_EXPORT,
            FLAG_SPEC,
            "config/export.json",
            FLAG_ROOT,
            ".",
            FLAG_EVALUATOR,
            "nickel",
            FLAG_EVALUATOR_IDENTITY,
            "nix:nickel",
            FLAG_EVALUATOR_VERSION,
            "nickel-1.13.0",
            FLAG_MANIFEST,
            "generated/manifest.json",
            FLAG_CHECK,
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }

    #[test]
    fn parser_accepts_complete_check_request() {
        let parsed = parse_args(&valid_args());
        assert!(parsed.is_ok());
        assert!(matches!(
            parsed.map(|options| options.mode),
            Ok(Mode::Check)
        ));
    }

    #[test]
    fn parser_rejects_missing_mode_unknown_flags_and_unsafe_paths() {
        let mut missing_mode = valid_args();
        missing_mode.pop();
        assert!(parse_args(&missing_mode).is_err());

        let mut unknown = valid_args();
        unknown.push("--ambient-authority".to_string());
        assert!(parse_args(&unknown).is_err());

        let mut unsafe_spec = valid_args();
        let spec_index = unsafe_spec
            .iter()
            .position(|value| value == FLAG_SPEC)
            .map(|index| index + 1)
            .unwrap_or_default();
        unsafe_spec[spec_index] = "../secret.ncl".to_string();
        assert!(parse_args(&unsafe_spec).is_err());
    }

    #[test]
    fn cli_contract_files_must_be_safe_declared_dependencies() {
        let mut request = ExportRequest {
            schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
            family_id: "tests.config".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: Vec::new(),
            import_paths: Vec::new(),
            selector: String::new(),
            contract: "DynamicEvidenceProfiles".to_string(),
            format: ExportFormat::Json,
            destination: "generated/config.json".to_string(),
            allow_secret_material: false,
        };
        assert!(validate_shell_contract(&request).is_err());
        request.contract = "config/contract.ncl".to_string();
        request.dependencies.push(request.contract.clone());
        assert!(validate_shell_contract(&request).is_ok());
        request.contract = "../contract.ncl".to_string();
        assert!(validate_shell_contract(&request).is_err());
    }

    #[test]
    fn plan_keeps_external_evaluation_in_the_shell() {
        let options = parse_args(&valid_args()).unwrap_or_else(|error| panic_for_test(&error));
        let request = ExportRequest {
            schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
            family_id: "tests.config".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: Vec::new(),
            import_paths: vec!["config/imports".to_string()],
            selector: "value".to_string(),
            contract: "config/contract.ncl".to_string(),
            format: ExportFormat::Raw,
            destination: "generated/value.txt".to_string(),
            allow_secret_material: false,
        };
        let plan = evaluation_plan(&options, &request, Path::new("/tmp/root"));
        assert_eq!(plan.program, PathBuf::from("nickel"));
        assert!(plan.args.contains(&OsString::from("text")));
        assert!(plan.args.contains(&OsString::from("--apply-contract")));
    }

    #[test]
    fn parser_rejects_conflicting_side_effect_modes() {
        let mut args = valid_args();
        args.push(FLAG_WRITE.to_string());
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn failures_serialize_without_a_success_receipt() {
        let error = parse_args(&["nickel-export".to_string()]).err();
        assert!(error.is_some());
        let rendered = error
            .as_ref()
            .and_then(|value| serde_json::to_string(value).ok())
            .unwrap_or_default();
        assert!(rendered.contains(SHELL_ERROR_SCHEMA));
        assert!(rendered.contains("arguments"));
        assert!(!rendered.contains("receipt"));
    }

    #[cfg(unix)]
    #[test]
    fn confined_path_check_accepts_direct_files_and_rejects_symlinks() {
        use std::os::unix::fs::symlink;

        let root =
            std::env::temp_dir().join(format!("nickel-export-path-test-{}", std::process::id()));
        if let Err(error) = fs::remove_dir_all(&root) {
            if error.kind() != std::io::ErrorKind::NotFound {
                panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
            }
        }
        fs::create_dir_all(root.join("safe")).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        fs::write(root.join("safe/input.ncl"), b"{}\n").unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });
        symlink(root.join("safe/input.ncl"), root.join("linked.ncl")).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-setup", error.to_string()));
        });

        assert!(reject_symlink_components(&root, Path::new("safe/input.ncl"), "test").is_ok());
        assert!(reject_symlink_components(&root, Path::new("linked.ncl"), "test").is_err());
        assert!(reject_symlink_components(&root, Path::new("new/output.json"), "test").is_ok());
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic_for_test::<()>(&ShellError::new("test-cleanup", error.to_string()));
        });
    }

    fn panic_for_test<T>(error: &ShellError) -> T {
        panic!("unexpected shell error: {error}")
    }
}
