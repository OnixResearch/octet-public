use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Component, Path, PathBuf},
    process::{Command, Output},
};

use nickel_export_core::{
    build_receipt, ArtifactMaterial, Diagnostic, DiagnosticSeverity, EvaluationObservation,
    EvaluatorDescriptor, ExportFormat, ExportReceipt, ExportRequest, ImportPathPolicy,
    REQUEST_SCHEMA,
};

pub(crate) const STANDALONE_SOURCE_REVISION: &str = "257fafc1c746f1faf156207043a4c826bfb16d49";
pub(crate) const NICKEL_EVALUATOR_IDENTITY: &str = "nixpkgs:nickel";
pub(crate) const NICKEL_EVALUATOR_VERSION: &str = "nickel-lang-cli-1.16.0";
const EXPORT_FORMAT_OPTION: &str = "format=json";
const SOURCE_REVISION_OPTION_PREFIX: &str = "source-revision=";
const GIT_SHA_HEX_LENGTH: usize = 40;
const NON_CLAIM: &str = "Nickel export parity proves exact declared input and output identity only; Cairn retains lifecycle semantics, gate authority, generated destinations, replay, and release readiness";

#[derive(Debug)]
pub(crate) struct DualRunExport {
    pub(crate) bytes: Vec<u8>,
    #[allow(dead_code)]
    pub(crate) receipt: ExportReceipt,
    #[allow(dead_code)]
    pub(crate) non_claim: &'static str,
}

#[derive(Debug)]
struct OwnedDependency {
    path: String,
    bytes: Vec<u8>,
}

pub(crate) fn export_json_dual_run(
    family_id: &str,
    source: &Path,
    destination: &Path,
    allow_reviewed_secret_markers: bool,
) -> Result<DualRunExport, Box<dyn Error>> {
    export_json_dual_run_inner(
        family_id,
        source,
        destination,
        None,
        allow_reviewed_secret_markers,
    )
}

pub(crate) fn export_json_dual_run_with_contract(
    family_id: &str,
    source: &Path,
    destination: &Path,
    contract: &Path,
    allow_reviewed_secret_markers: bool,
) -> Result<DualRunExport, Box<dyn Error>> {
    export_json_dual_run_inner(
        family_id,
        source,
        destination,
        Some(contract),
        allow_reviewed_secret_markers,
    )
}

fn export_json_dual_run_inner(
    family_id: &str,
    source: &Path,
    destination: &Path,
    contract: Option<&Path>,
    allow_reviewed_secret_markers: bool,
) -> Result<DualRunExport, Box<dyn Error>> {
    validate_pinned_cohort(STANDALONE_SOURCE_REVISION, NICKEL_EVALUATOR_VERSION)?;
    let source_path = portable_relative_path(source)?;
    let destination_path = portable_relative_path(destination)?;
    reject_symlink_components(source)?;
    reject_symlink_components(destination)?;
    let source_bytes = fs::read(source)?;
    let mut dependencies = load_declared_dependencies(source, &source_bytes)?;
    let contract_path = if let Some(contract) = contract {
        reject_symlink_components(contract)?;
        let bytes = fs::read(contract)?;
        dependencies.extend(load_declared_dependencies(contract, &bytes)?);
        dependencies.push(OwnedDependency {
            path: portable_relative_path(contract)?,
            bytes,
        });
        dependencies.sort_by(|left, right| left.path.cmp(&right.path));
        dependencies.dedup_by(|left, right| left.path == right.path);
        portable_relative_path(contract)?
    } else {
        String::new()
    };

    let request = ExportRequest {
        schema: REQUEST_SCHEMA.to_string(),
        family_id: family_id.to_string(),
        source: source_path.clone(),
        dependencies: dependencies
            .iter()
            .map(|dependency| dependency.path.clone())
            .collect(),
        import_paths: Vec::new(),
        selector: String::new(),
        contract: contract_path,
        format: ExportFormat::Json,
        destination: destination_path.clone(),
        allow_secret_material: allow_reviewed_secret_markers,
    };
    let legacy = run_nickel(source, contract)?;
    let standalone = run_standalone_shell(&request)?;
    require_matching_observations(&legacy, &standalone)?;

    let dependency_material = dependencies
        .iter()
        .map(|dependency| ArtifactMaterial {
            path: dependency.path.as_str(),
            bytes: dependency.bytes.as_slice(),
        })
        .collect();
    let evaluator = EvaluatorDescriptor {
        identity: NICKEL_EVALUATOR_IDENTITY.to_string(),
        version: NICKEL_EVALUATOR_VERSION.to_string(),
        options: vec![
            EXPORT_FORMAT_OPTION.to_string(),
            format!("{SOURCE_REVISION_OPTION_PREFIX}{STANDALONE_SOURCE_REVISION}"),
        ],
        import_path_policy: ImportPathPolicy::DeclaredOnly,
    };
    let observation = EvaluationObservation {
        request: &request,
        source: ArtifactMaterial {
            path: source_path.as_str(),
            bytes: source_bytes.as_slice(),
        },
        dependencies: dependency_material,
        output: ArtifactMaterial {
            path: destination_path.as_str(),
            bytes: standalone.stdout.as_slice(),
        },
        evaluator: &evaluator,
        observed_dependencies: Vec::new(),
        diagnostics: diagnostics_from_output(&standalone),
    };
    let receipt = build_receipt(&observation)
        .map_err(|error| format!("standalone Nickel export admission failed: {error:?}"))?;

    Ok(DualRunExport {
        bytes: legacy.stdout,
        receipt,
        non_claim: NON_CLAIM,
    })
}

fn validate_pinned_cohort(revision: &str, evaluator_version: &str) -> Result<(), Box<dyn Error>> {
    let revision_is_hex = revision.len() == GIT_SHA_HEX_LENGTH
        && revision.bytes().all(|byte| byte.is_ascii_hexdigit());
    if !revision_is_hex {
        return Err("standalone Nickel export source must use an exact Git revision".into());
    }
    if evaluator_version != NICKEL_EVALUATOR_VERSION {
        return Err("Nickel evaluator cohort drift".into());
    }
    Ok(())
}

fn run_standalone_shell(request: &ExportRequest) -> Result<Output, Box<dyn Error>> {
    let scratch_relative = PathBuf::from("target")
        .join("cairn-nickel-export-observation")
        .join(std::process::id().to_string());
    reject_symlink_components(&scratch_relative)?;
    fs::create_dir_all(&scratch_relative)?;
    let request_path = scratch_relative.join("request.json");
    let output_path = scratch_relative.join("output.json");
    let manifest_path = scratch_relative.join("manifest.json");
    let mut isolated_request = request.clone();
    isolated_request.destination = portable_relative_path(&output_path)?;
    fs::write(&request_path, serde_json::to_vec_pretty(&isolated_request)?)?;

    let write = Command::new("nickel-export")
        .arg("export")
        .arg("--spec")
        .arg(&request_path)
        .arg("--root")
        .arg(".")
        .arg("--evaluator")
        .arg("nickel")
        .arg("--evaluator-identity")
        .arg(NICKEL_EVALUATOR_IDENTITY)
        .arg("--evaluator-version")
        .arg(NICKEL_EVALUATOR_VERSION)
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--write")
        .output()?;
    if !write.status.success() {
        let _ = fs::remove_dir_all(&scratch_relative);
        return Ok(write);
    }
    let bytes = fs::read(&output_path)?;
    let check = Command::new("nickel-export")
        .arg("export")
        .arg("--spec")
        .arg(&request_path)
        .arg("--root")
        .arg(".")
        .arg("--evaluator")
        .arg("nickel")
        .arg("--evaluator-identity")
        .arg(NICKEL_EVALUATOR_IDENTITY)
        .arg("--evaluator-version")
        .arg(NICKEL_EVALUATOR_VERSION)
        .arg("--manifest")
        .arg(&manifest_path)
        .arg("--check")
        .output()?;
    let mut stderr = write.stderr;
    stderr.extend(check.stderr);
    fs::remove_dir_all(&scratch_relative)?;
    Ok(Output {
        status: check.status,
        stdout: bytes,
        stderr,
    })
}

fn run_nickel(source: &Path, contract: Option<&Path>) -> Result<Output, Box<dyn Error>> {
    let mut command = Command::new("nickel");
    command
        .arg("export")
        .arg("--format")
        .arg("json")
        .arg(source);
    if let Some(contract) = contract {
        command.arg("--apply-contract").arg(contract);
    }
    let output = command.output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "nickel export failed for {}: {}{}",
            source.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into())
    }
}

fn require_matching_observations(
    legacy: &Output,
    standalone: &Output,
) -> Result<(), Box<dyn Error>> {
    if legacy.status.success() != standalone.status.success() {
        return Err("legacy and standalone Nickel export status drift".into());
    }
    if legacy.stdout != standalone.stdout {
        return Err("legacy and standalone Nickel export byte drift".into());
    }
    if legacy.stderr != standalone.stderr {
        return Err("legacy and standalone Nickel export diagnostic drift".into());
    }
    Ok(())
}

fn diagnostics_from_output(output: &Output) -> Vec<Diagnostic> {
    if output.stderr.is_empty() {
        Vec::new()
    } else {
        vec![Diagnostic::new(
            "evaluator-stderr",
            "nickel",
            String::from_utf8_lossy(&output.stderr).trim(),
            DiagnosticSeverity::Warning,
        )]
    }
}

fn load_declared_dependencies(
    source: &Path,
    source_bytes: &[u8],
) -> Result<Vec<OwnedDependency>, Box<dyn Error>> {
    let mut pending = declared_imports(source, source_bytes)?;
    let mut visited = BTreeSet::new();
    let mut dependencies = Vec::new();
    while let Some(path) = pending.pop() {
        let portable = portable_relative_path(&path)?;
        if !visited.insert(portable.clone()) {
            continue;
        }
        reject_symlink_components(&path)?;
        let bytes = fs::read(&path)?;
        pending.extend(declared_imports(&path, &bytes)?);
        dependencies.push(OwnedDependency {
            path: portable,
            bytes,
        });
    }
    dependencies.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(dependencies)
}

fn declared_imports(source: &Path, bytes: &[u8]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let text = std::str::from_utf8(bytes)?;
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let mut paths = Vec::new();
    for line in text.lines() {
        let Some((_, suffix)) = line.split_once("import \"") else {
            continue;
        };
        let Some((import, _)) = suffix.split_once('"') else {
            return Err(format!("unterminated Nickel import in {}", source.display()).into());
        };
        paths.push(normalize_join(parent, Path::new(import))?);
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn normalize_join(parent: &Path, child: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut normalized = PathBuf::new();
    for component in parent.join(child).components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => normalized.push(value),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("Nickel import escapes the repository root".into());
                }
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err("Nickel import must be repository-root-relative".into());
            }
        }
    }
    Ok(normalized)
}

fn reject_symlink_components(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut observed = PathBuf::new();
    for component in path.components() {
        observed.push(component.as_os_str());
        match fs::symlink_metadata(&observed) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "Nickel export path contains a symlink: {}",
                    observed.display()
                )
                .into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn portable_relative_path(path: &Path) -> Result<String, Box<dyn Error>> {
    let normalized = normalize_join(Path::new("."), path)?;
    let rendered = normalized.to_string_lossy().replace('\\', "/");
    if rendered.is_empty() {
        return Err("Nickel export path must name an artifact".into());
    }
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;

    #[cfg(unix)]
    fn status(code: i32) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code)
    }

    #[test]
    // r[verify cairn.nickel_export_cutover.authority]
    // r[verify cairn.nickel_export_cutover.rollback]
    fn matching_observations_preserve_legacy_authority() {
        let legacy = Output {
            status: status(0),
            stdout: b"{\"ok\":true}\n".to_vec(),
            stderr: Vec::new(),
        };
        let standalone = Output {
            status: status(0),
            stdout: legacy.stdout.clone(),
            stderr: Vec::new(),
        };
        assert!(require_matching_observations(&legacy, &standalone).is_ok());
    }

    #[test]
    // r[verify cairn.nickel_export_cutover.dual_run]
    // r[verify cairn.nickel_export_cutover.validation]
    fn byte_or_diagnostic_drift_fails_closed() {
        let legacy = Output {
            status: status(0),
            stdout: b"legacy".to_vec(),
            stderr: Vec::new(),
        };
        let byte_drift = Output {
            status: status(0),
            stdout: b"standalone".to_vec(),
            stderr: Vec::new(),
        };
        let diagnostic_drift = Output {
            status: status(0),
            stdout: legacy.stdout.clone(),
            stderr: b"warning".to_vec(),
        };
        assert!(require_matching_observations(&legacy, &byte_drift).is_err());
        assert!(require_matching_observations(&legacy, &diagnostic_drift).is_err());
    }

    #[test]
    // r[verify cairn.nickel_export_cutover.boundary]
    fn path_escape_and_absolute_paths_are_rejected() {
        assert!(portable_relative_path(Path::new("../outside.ncl")).is_err());
        assert!(portable_relative_path(Path::new("/outside.ncl")).is_err());
        assert!(portable_relative_path(Path::new("")).is_err());
        assert_eq!(
            portable_relative_path(Path::new("cairn-policy/default.ncl"))
                .unwrap_or_else(|error| panic!("unexpected path error: {error}")),
            "cairn-policy/default.ncl"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlink_inputs_are_rejected() {
        use std::os::unix::fs::symlink;

        let root = std::env::temp_dir().join(format!(
            "cairn-nickel-export-symlink-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to create fixture root: {error}"));
        let target = root.join("target.ncl");
        let link = root.join("source.ncl");
        fs::write(&target, "{}\n")
            .unwrap_or_else(|error| panic!("failed to write fixture: {error}"));
        symlink(&target, &link)
            .unwrap_or_else(|error| panic!("failed to create fixture symlink: {error}"));
        assert!(reject_symlink_components(&link).is_err());
        fs::remove_dir_all(&root)
            .unwrap_or_else(|error| panic!("failed to clean fixture root: {error}"));
    }

    #[test]
    // r[verify cairn.nickel_export_cutover.source]
    fn source_pin_and_evaluator_cohort_are_exact() {
        assert!(
            validate_pinned_cohort(STANDALONE_SOURCE_REVISION, NICKEL_EVALUATOR_VERSION).is_ok()
        );
        assert!(validate_pinned_cohort("main", NICKEL_EVALUATOR_VERSION).is_err());
        assert!(validate_pinned_cohort(STANDALONE_SOURCE_REVISION, "nickel-floating").is_err());
    }
}
