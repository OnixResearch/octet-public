use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

use crate::artifact;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;
use crate::cli::AdmissionArgs;
use crate::cli::AdmissionMode;
use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::effect_closure;
use crate::effect_closure::AdmissionStatus;
use crate::effect_closure::ClosureInput;
use crate::effect_closure::EffectClosureReceipt;
use crate::effect_closure::FunctionAddressRecord;
use crate::effect_closure::ProviderIdentity;
use crate::effect_closure::ProviderKind;
use crate::effect_closure::SourceEnvelope;
use crate::effect_closure::WorkspacePolicy;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;
use crate::function_address::FunctionAddressCheck;
use crate::function_address::FunctionAddressMode;

const EXIT_ADMITTED: i32 = 0;
const EXIT_DENIED: i32 = 1;
const ADMISSION_STATUS_SCHEMA: &str = "octet-effect-closure-status/v1";
const ADMISSION_VERIFICATION_SCHEMA: &str = "octet-effect-closure-verification/v1";
const DEPENDENCY_GRAPH_SCHEMA: &str = "octet-effect-closure-dependency-graph/v1";
const STATUS_ADMITTED: &str = "admitted";
const STATUS_DENIED: &str = "denied";
const STATUS_VALID: &str = "valid";
const STATUS_INVALID: &str = "invalid";
const PHASE_DIRECT_CHECK: &str = "direct-check";
const PHASE_CLOSURE: &str = "effect-closure";
const CHECK_DIR: &str = "check";
const COMMAND_FILE: &str = "command.txt";
const STATUS_FILE: &str = "status.json";
const SUMMARY_FILE: &str = "summary.txt";
const PROVENANCE_FILE: &str = "provenance.jsonl";
const WORKSPACE_POLICY_ARTIFACT: &str = "workspace-policy.json";
const DEPENDENCY_GRAPH_ARTIFACT: &str = "dependency-graph.json";
const CHECK_STATUS_ARTIFACT: &str = "check/status.json";
const CHECK_FUNCTION_ADDRESS_ARTIFACT: &str = "check/function-addresses.json";
const FUNCTION_ADDRESS_ROOT_KIND: &str = "function-address-record-root";
const FUNCTION_ADDRESS_MODE_DENY: &str = "deny";
const CARGO_COMMAND: &str = "cargo";
const CARGO_METADATA_COMMAND: &str = "metadata";
const CARGO_METADATA_FORMAT_VERSION: &str = "1";
const TOOLCHAIN_ENV: &str = "RUSTUP_TOOLCHAIN";
const CHECKED_SCOPE_PREFIX: &str = "effect-closure:";
const PROVENANCE_SOURCE_ID: &str = "octet.runner.effect-closure";
const CHECKER_ID: &str = "cargo-octet";
const PROVIDER_SOURCE_TOOLCHAIN: &str = "rustc-toolchain";
const PROVIDER_SOURCE_WORKSPACE: &str = "workspace";
const PROVIDER_SOURCE_PATH_PREFIX: &str = "path-manifest:";
const TOOLCHAIN_PROVIDER_NAMES: &[&str] = &["alloc", "core", "rustc", "std"];
const TOP_LEVEL_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    PROVENANCE_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
];
const REQUIRED_PROVENANCE_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
    CHECK_STATUS_ARTIFACT,
    CHECK_FUNCTION_ADDRESS_ARTIFACT,
];
const NON_CLAIM_INTEGRITY: &str = "Admission verification checks stored BLAKE3 identity and linkage only; it does not rerun Cargo, source inference, dependency review, or compiler analysis, and it provides no signer authenticity, authorization, or defense against coherent replacement of every trust root.";

#[derive(Clone, Debug, Deserialize)]
struct CheckStatus {
    status: String,
    exit_code: i32,
    metadata: CheckMetadata,
    error_findings: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct CheckMetadata {
    rustc_version: String,
    toolchain: String,
    profile_name: String,
    profile_hash: String,
    config_hash: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FunctionAddressReport {
    mode: String,
    source_envelopes: Vec<FunctionAddressSourceEnvelope>,
    records: Vec<RawFunctionAddressRecord>,
    evidence_chain_handoff: FunctionAddressHandoff,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct FunctionAddressSourceEnvelope {
    path: String,
    source_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
struct RawFunctionAddressRecord {
    path: String,
    name: String,
    function_address: String,
    source_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FunctionAddressHandoff {
    canonical_roots: Vec<CanonicalRoot>,
}

#[derive(Clone, Debug, Deserialize)]
struct CanonicalRoot {
    kind: String,
    digest: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
    resolve: Option<CargoResolve>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoNode {
    id: String,
    features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DependencyGraphArtifact {
    schema_version: String,
    target: String,
    toolchain: String,
    rustc_version: String,
    cargo_lock_blake3: String,
    providers: Vec<ProviderIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AdmissionStatusArtifact {
    schema_version: String,
    status: String,
    exit_code: i32,
    phase: String,
    profile_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_identity: Option<String>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AdmissionVerificationReport {
    schema_version: String,
    artifact_dir: String,
    status: String,
    receipt_identity: Option<String>,
    checked_artifacts: Vec<String>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

struct PreparedInputs {
    policy: WorkspacePolicy,
    policy_bytes: Vec<u8>,
    baseline_path: PathBuf,
}

// r[impl octet.effect_closure.cli]
// r[impl octet.effect_closure.boundaries.default]
pub fn run(args: &AdmissionArgs) -> Result<i32, String> {
    match &args.mode {
        AdmissionMode::Run {
            profile,
            closure,
            policy,
            scope_args,
            cargo_check_args,
        } => run_admission(args, profile, *closure, policy, scope_args, cargo_check_args.as_deref()),
        AdmissionMode::Verify => run_verification(args),
    }
}

// r[impl octet.effect_closure.cli.admitted]
// r[impl octet.effect_closure.cli.check_failure]
fn run_admission(
    args: &AdmissionArgs,
    profile: &str,
    closure: bool,
    policy_path: &Path,
    scope_args: &[String],
    cargo_check_args: Option<&[String]>,
) -> Result<i32, String> {
    if profile != effect_closure::DETERMINISTIC_CORE_PROFILE || !closure {
        return Err(format!(
            "effect closure v1 requires `--profile {} --closure`",
            effect_closure::DETERMINISTIC_CORE_PROFILE
        ));
    }

    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let workspace_config = crate::config::load(&cwd)?;
    let workspace_root = workspace_config.workspace_root;
    let selected_profile = crate::strict_profile::prepare_selected_profile(Some(profile), &workspace_root)?
        .ok_or_else(|| "effect closure requires a selected strict profile".to_owned())?;
    let prepared = prepare_inputs(&workspace_root, policy_path)?;
    let artifact_dir = absolute_from_workspace(&workspace_root, &args.artifact_dir);
    let check_dir = artifact_dir.join(CHECK_DIR);
    let root = prepare_artifact_root(&artifact_dir)?;
    let command_bytes = normalized_text_bytes(&args.command_line);
    root.write(&relative(COMMAND_FILE)?, &command_bytes)?;

    let check = CheckArgs {
        scope_args: scope_args.to_vec(),
        cargo_check_args: cargo_check_args.map(<[String]>::to_vec),
        profile: Some(profile.to_owned()),
        output_format: OutputFormat::Human,
        artifact_dir: Some(check_dir.clone()),
        baseline: None,
        write_baseline: None,
        cache: false,
        function_addresses: FunctionAddressCheck {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
            baseline: Some(prepared.baseline_path.clone()),
        },
        command_line: args.command_line.clone(),
    };
    let check_exit = crate::engine::run(&check)?;
    if check_exit != EXIT_ADMITTED {
        write_blocked_status(&root, profile, check_exit, &command_bytes)?;
        return Ok(check_exit);
    }
    verify_check_bundle(&check_dir)?;

    let check_status_bytes = read_required(&check_dir.join(STATUS_FILE))?;
    let function_address_bytes =
        read_required(&check_dir.join(crate::function_address::FUNCTION_ADDRESS_ARTIFACT_NAME))?;
    let check_status = parse_check_status(&check_status_bytes, profile)?;
    let function_report = parse_function_report(&function_address_bytes)?;
    let target = selected_target(cargo_check_args.unwrap_or_default())?;
    let (dependency_graph, dependency_graph_bytes) =
        dependency_graph(&workspace_root, cargo_check_args.unwrap_or_default(), &target, &check_status.metadata)?;
    let sources = read_selected_sources(&workspace_root, &selected_profile.core_source_scopes)?;
    validate_function_source_envelopes(&sources, &function_report)?;
    let record_root = function_address_record_root(&function_report)?;
    let catalog = effect_closure::embedded_catalog()?;

    let receipt = effect_closure::compute_closure(ClosureInput {
        profile_id: profile.to_owned(),
        profile_hash: check_status.metadata.profile_hash.clone(),
        config_hash: check_status.metadata.config_hash.clone(),
        command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
        check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
        function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
        function_address_record_root: record_root,
        workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&prepared.policy_bytes),
        cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&dependency_graph_bytes),
        cargo_lock_blake3: dependency_graph.cargo_lock_blake3.clone(),
        target,
        toolchain: check_status.metadata.toolchain.clone(),
        sources,
        function_addresses: function_report
            .records
            .into_iter()
            .map(|record| FunctionAddressRecord {
                path: record.path,
                name: record.name,
                function_address: record.function_address,
                source_identity: record.source_identity,
            })
            .collect(),
        resolved_providers: dependency_graph.providers.clone(),
        catalog,
        policy: prepared.policy,
    });

    write_closure_bundle(
        &root,
        &artifact_dir,
        &command_bytes,
        &prepared.policy_bytes,
        &dependency_graph_bytes,
        &check_status_bytes,
        &function_address_bytes,
        &receipt,
    )?;
    render_admission_output(args.output_format, &artifact_dir, &receipt)?;
    Ok(match receipt.status {
        AdmissionStatus::Admitted => EXIT_ADMITTED,
        AdmissionStatus::Denied => EXIT_DENIED,
    })
}

fn run_verification(args: &AdmissionArgs) -> Result<i32, String> {
    let report = verify_bundle(&args.artifact_dir)?;
    match args.output_format {
        OutputFormat::Human => {
            eprintln!("Effect closure admission verification: {}", report.status);
            eprintln!("Artifact directory: {}", report.artifact_dir);
            if let Some(identity) = &report.receipt_identity {
                eprintln!("Receipt identity: {identity}");
            }
            for diagnostic in &report.diagnostics {
                eprintln!("- {diagnostic}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize admission verification report: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    Ok(if report.diagnostics.is_empty() {
        EXIT_ADMITTED
    } else {
        EXIT_DENIED
    })
}

fn prepare_inputs(workspace_root: &Path, policy_path: &Path) -> Result<PreparedInputs, String> {
    let policy_path = checked_workspace_path(workspace_root, policy_path, "effect-closure policy")?;
    let policy_bytes = read_required(&policy_path)?;
    let policy = serde_json::from_slice::<WorkspacePolicy>(&policy_bytes)
        .map_err(|error| format!("parse {}: {error}", policy_path.display()))?;
    let diagnostics = effect_closure::validate_workspace_policy(&policy);
    if !diagnostics.is_empty() {
        return Err(format!("invalid effect-closure workspace policy:\n- {}", diagnostics.join("\n- ")));
    }
    let baseline_path = checked_workspace_path(
        workspace_root,
        Path::new(&policy.function_address_baseline),
        "function-address baseline",
    )?;
    if !baseline_path.is_file() {
        return Err(format!("function-address baseline does not exist: {}", baseline_path.display()));
    }
    Ok(PreparedInputs {
        policy,
        policy_bytes,
        baseline_path,
    })
}

fn prepare_artifact_root(path: &Path) -> Result<ArtifactRoot, String> {
    let root = ArtifactRoot::open_or_create(path)?;
    for artifact in TOP_LEVEL_ARTIFACTS {
        root.remove_file_or_dir_all(&relative(artifact)?)?;
    }
    Ok(root)
}

fn write_blocked_status(
    root: &ArtifactRoot,
    profile: &str,
    check_exit: i32,
    command_bytes: &[u8],
) -> Result<(), String> {
    let diagnostic =
        format!("direct strict-profile check failed with exit code {check_exit}; closure was not attempted");
    let status = AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: STATUS_DENIED.to_owned(),
        exit_code: check_exit,
        phase: PHASE_DIRECT_CHECK.to_owned(),
        profile_id: profile.to_owned(),
        receipt_identity: None,
        diagnostics: vec![diagnostic.clone()],
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    };
    let status_bytes = pretty_json_bytes(&status)?;
    let summary = format!(
        "Octet effect closure admission\nStatus: denied\nPhase: direct-check\nProfile: {profile}\nDiagnostic: {diagnostic}\n"
    );
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, summary.as_bytes())
}

fn verify_check_bundle(check_dir: &Path) -> Result<(), String> {
    let root = ArtifactRoot::open_existing(check_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| artifact::BundleFile {
            name: file.locator,
            contents: file.contents,
        })
        .collect::<Vec<_>>();
    let report = artifact::verify_bundle(artifact::ArtifactBundleInput {
        artifact_dir: check_dir.display().to_string(),
        files,
    });
    if report.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "strict-profile parent artifact verification failed:\n- {}",
            report.diagnostics.join("\n- ")
        ))
    }
}

fn parse_check_status(bytes: &[u8], expected_profile: &str) -> Result<CheckStatus, String> {
    let status =
        serde_json::from_slice::<CheckStatus>(bytes).map_err(|error| format!("parse check/status.json: {error}"))?;
    if status.exit_code != EXIT_ADMITTED
        || status.error_findings != 0
        || status.metadata.profile_name != expected_profile
    {
        return Err(format!(
            "direct check status is not eligible for closure: status={}, exit_code={}, errors={}, profile={}",
            status.status, status.exit_code, status.error_findings, status.metadata.profile_name
        ));
    }
    Ok(status)
}

fn parse_function_report(bytes: &[u8]) -> Result<FunctionAddressReport, String> {
    let report = serde_json::from_slice::<FunctionAddressReport>(bytes)
        .map_err(|error| format!("parse check/function-addresses.json: {error}"))?;
    if report.mode != FUNCTION_ADDRESS_MODE_DENY {
        return Err(format!("effect closure requires deny-mode function addresses, got `{}`", report.mode));
    }
    if report.records.is_empty() {
        return Err("effect closure requires at least one function-address record".to_owned());
    }
    Ok(report)
}

fn function_address_record_root(report: &FunctionAddressReport) -> Result<String, String> {
    report
        .evidence_chain_handoff
        .canonical_roots
        .iter()
        .find(|root| root.kind == FUNCTION_ADDRESS_ROOT_KIND)
        .map(|root| root.digest.clone())
        .ok_or_else(|| "function-address receipt is missing its canonical record root".to_owned())
}

fn read_selected_sources(workspace_root: &Path, scopes: &[PathBuf]) -> Result<Vec<SourceEnvelope>, String> {
    let source_root = SourceRoot::open_existing(workspace_root)?;
    let mut selected = BTreeMap::<String, Vec<u8>>::new();
    for scope in scopes {
        let relative_scope = RootRelativePath::new(scope)?;
        if source_root.is_dir(&relative_scope)? {
            for file in source_root.read_files_recursive_from(&relative_scope)? {
                if Path::new(&file.locator).extension().is_some_and(|extension| extension == "rs") {
                    selected.insert(file.locator, file.contents);
                }
            }
        } else {
            if scope.extension().is_none_or(|extension| extension != "rs") {
                return Err(format!("selected core source file is not Rust: {}", scope.display()));
            }
            let source = source_root.read_to_string(&relative_scope)?;
            selected.insert(relative_scope.display(), source.into_bytes());
        }
    }
    if selected.is_empty() {
        return Err("selected core source scopes contain no Rust files".to_owned());
    }
    selected
        .into_iter()
        .map(|(path, bytes)| {
            let source = String::from_utf8(bytes)
                .map_err(|error| format!("selected Rust source `{path}` is not UTF-8: {error}"))?;
            let source_identity = crate::determinism::full_hash(&source);
            Ok(SourceEnvelope {
                path,
                source,
                source_identity,
            })
        })
        .collect()
}

fn validate_function_source_envelopes(
    sources: &[SourceEnvelope],
    report: &FunctionAddressReport,
) -> Result<(), String> {
    let selected = sources
        .iter()
        .map(|source| FunctionAddressSourceEnvelope {
            path: source.path.clone(),
            source_identity: source.source_identity.clone(),
        })
        .collect::<BTreeSet<_>>();
    let reported = report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
    if selected == reported {
        Ok(())
    } else {
        Err("selected source identities do not exactly match deny-mode function-address envelopes".to_owned())
    }
}

fn dependency_graph(
    workspace_root: &Path,
    cargo_check_args: &[String],
    target: &str,
    check_metadata: &CheckMetadata,
) -> Result<(DependencyGraphArtifact, Vec<u8>), String> {
    let cargo_lock = read_required(&workspace_root.join("Cargo.lock"))?;
    let cargo_lock_blake3 = crate::evidence::artifact_blake3_hex(&cargo_lock);
    let mut command = Command::new(CARGO_COMMAND);
    command
        .arg(CARGO_METADATA_COMMAND)
        .arg("--format-version")
        .arg(CARGO_METADATA_FORMAT_VERSION)
        .arg("--locked")
        .arg("--filter-platform")
        .arg(target)
        .args(metadata_feature_args(cargo_check_args))
        .current_dir(workspace_root)
        .env(TOOLCHAIN_ENV, &check_metadata.toolchain);
    let output = command.output().map_err(|error| format!("run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata = serde_json::from_slice::<CargoMetadata>(&output.stdout)
        .map_err(|error| format!("parse cargo metadata JSON: {error}"))?;
    let workspace_members = metadata.workspace_members.into_iter().collect::<BTreeSet<_>>();
    let feature_map = metadata
        .resolve
        .map(|resolve| {
            resolve
                .nodes
                .into_iter()
                .map(|node| {
                    let mut features = node.features;
                    features.sort();
                    features.dedup();
                    (node.id, features)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut providers = Vec::new();
    for package in metadata.packages {
        let Some(features) = feature_map.get(&package.id).cloned() else {
            continue;
        };
        let workspace_package = workspace_members.contains(&package.id);
        let source = package_source(&package, workspace_package);
        providers.push(ProviderIdentity {
            kind: if workspace_package {
                ProviderKind::Workspace
            } else {
                ProviderKind::CargoPackage
            },
            name: package.name,
            version: package.version,
            source,
            target: target.to_owned(),
            features,
        });
    }
    for name in TOOLCHAIN_PROVIDER_NAMES {
        providers.push(ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: (*name).to_owned(),
            version: check_metadata.rustc_version.clone(),
            source: PROVIDER_SOURCE_TOOLCHAIN.to_owned(),
            target: target.to_owned(),
            features: Vec::new(),
        });
    }
    providers.sort();
    providers.dedup();
    let graph = DependencyGraphArtifact {
        schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
        target: target.to_owned(),
        toolchain: check_metadata.toolchain.clone(),
        rustc_version: check_metadata.rustc_version.clone(),
        cargo_lock_blake3,
        providers,
    };
    let bytes = pretty_json_bytes(&graph)?;
    Ok((graph, bytes))
}

fn package_source(package: &CargoPackage, workspace_package: bool) -> String {
    if workspace_package {
        return PROVIDER_SOURCE_WORKSPACE.to_owned();
    }
    package.source.clone().unwrap_or_else(|| {
        let manifest_hash = fs::read(&package.manifest_path)
            .map(|bytes| crate::evidence::artifact_blake3_hex(&bytes))
            .unwrap_or_else(|_| crate::determinism::full_hash(&package.manifest_path));
        format!("{PROVIDER_SOURCE_PATH_PREFIX}{manifest_hash}")
    })
}

fn metadata_feature_args(cargo_check_args: &[String]) -> Vec<String> {
    let mut args = Vec::new();
    let mut index = 0;
    while index < cargo_check_args.len() {
        let arg = &cargo_check_args[index];
        match arg.as_str() {
            "--features" => {
                if let Some(value) = cargo_check_args.get(index + 1) {
                    args.push(arg.clone());
                    args.push(value.clone());
                    index += 2;
                    continue;
                }
            }
            "--all-features" | "--no-default-features" => args.push(arg.clone()),
            _ if arg.starts_with("--features=") => args.push(arg.clone()),
            _ => {}
        }
        index += 1;
    }
    args
}

fn selected_target(cargo_check_args: &[String]) -> Result<String, String> {
    let mut explicit = None;
    let mut index = 0;
    while index < cargo_check_args.len() {
        let arg = &cargo_check_args[index];
        if arg == "--target" {
            let value = cargo_check_args.get(index + 1).ok_or_else(|| "missing target after `--target`".to_owned())?;
            if explicit.replace(value.clone()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
            index += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--target=") {
            if value.is_empty() {
                return Err("missing target after `--target=`".to_owned());
            }
            if explicit.replace(value.to_owned()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
        }
        index += 1;
    }
    explicit.ok_or_else(|| {
        "effect closure v1 requires an explicit Cargo `--target TARGET` argument after `--`; ambient Cargo target configuration is not admissible evidence"
            .to_owned()
    })
}

// r[impl octet.effect_closure.evidence.visible]
fn write_closure_bundle(
    root: &ArtifactRoot,
    artifact_dir: &Path,
    command_bytes: &[u8],
    policy_bytes: &[u8],
    dependency_graph_bytes: &[u8],
    check_status_bytes: &[u8],
    function_address_bytes: &[u8],
    receipt: &EffectClosureReceipt,
) -> Result<(), String> {
    let receipt_bytes = pretty_json_bytes(receipt)?;
    let status = status_from_receipt(receipt);
    let status_bytes = pretty_json_bytes(&status)?;
    let summary_bytes = render_summary(receipt).into_bytes();
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(effect_closure::RECEIPT_FILE)?, &receipt_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, &summary_bytes)?;
    root.write(&relative(WORKSPACE_POLICY_ARTIFACT)?, policy_bytes)?;
    root.write(&relative(DEPENDENCY_GRAPH_ARTIFACT)?, dependency_graph_bytes)?;

    let mut artifacts = BTreeMap::from([
        (COMMAND_FILE.to_owned(), command_bytes.to_vec()),
        (effect_closure::RECEIPT_FILE.to_owned(), receipt_bytes),
        (STATUS_FILE.to_owned(), status_bytes),
        (SUMMARY_FILE.to_owned(), summary_bytes),
        (WORKSPACE_POLICY_ARTIFACT.to_owned(), policy_bytes.to_vec()),
        (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), dependency_graph_bytes.to_vec()),
        (CHECK_STATUS_ARTIFACT.to_owned(), check_status_bytes.to_vec()),
        (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), function_address_bytes.to_vec()),
    ]);
    let provenance = render_provenance(&artifacts, &receipt.inputs.profile_id)?;
    root.write(&relative(PROVENANCE_FILE)?, provenance.as_bytes())?;
    artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());

    let verification = verify_bundle(artifact_dir)?;
    if verification.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "new effect-closure artifact bundle failed self-verification:\n- {}",
            verification.diagnostics.join("\n- ")
        ))
    }
}

fn status_from_receipt(receipt: &EffectClosureReceipt) -> AdmissionStatusArtifact {
    AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: match receipt.status {
            AdmissionStatus::Admitted => STATUS_ADMITTED,
            AdmissionStatus::Denied => STATUS_DENIED,
        }
        .to_owned(),
        exit_code: match receipt.status {
            AdmissionStatus::Admitted => EXIT_ADMITTED,
            AdmissionStatus::Denied => EXIT_DENIED,
        },
        phase: PHASE_CLOSURE.to_owned(),
        profile_id: receipt.inputs.profile_id.clone(),
        receipt_identity: Some(receipt.receipt_identity.clone()),
        diagnostics: receipt
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect(),
        non_claims: receipt.non_claims.clone(),
    }
}

fn render_summary(receipt: &EffectClosureReceipt) -> String {
    let mut summary = String::new();
    let status = match receipt.status {
        AdmissionStatus::Admitted => STATUS_ADMITTED,
        AdmissionStatus::Denied => STATUS_DENIED,
    };
    let _ = writeln!(&mut summary, "Octet effect closure admission");
    let _ = writeln!(&mut summary, "Status: {status}");
    let _ = writeln!(&mut summary, "Claim scope: {}", receipt.claim_scope);
    let _ = writeln!(&mut summary, "Profile: {}", receipt.inputs.profile_id);
    let _ = writeln!(&mut summary, "Profile hash: {}", receipt.inputs.profile_hash);
    let _ = writeln!(&mut summary, "Config hash: {}", receipt.inputs.config_hash);
    let _ = writeln!(&mut summary, "Receipt identity: {}", receipt.receipt_identity);
    let _ = writeln!(&mut summary, "Selected sources: {}", receipt.sources.len());
    let _ = writeln!(&mut summary, "Functions: {}", receipt.nodes.len());
    let _ = writeln!(&mut summary, "Reviewed providers: {}", receipt.providers.len());
    let _ = writeln!(&mut summary, "Diagnostics: {}", receipt.diagnostics.len());
    for diagnostic in &receipt.diagnostics {
        let call_site = diagnostic.call_site_id.as_deref().unwrap_or("-");
        let _ = writeln!(&mut summary, "- {} [{}] {}", diagnostic.code, call_site, diagnostic.message);
    }
    let _ = writeln!(&mut summary, "Non-claims:");
    for non_claim in &receipt.non_claims {
        let _ = writeln!(&mut summary, "- {non_claim}");
    }
    summary
}

fn render_admission_output(
    output_format: OutputFormat,
    artifact_dir: &Path,
    receipt: &EffectClosureReceipt,
) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => eprint!("{}", render_summary(receipt)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&status_from_receipt(receipt))
                .map_err(|error| format!("serialize admission status output: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    eprintln!("Admission artifacts: {}", artifact_dir.display());
    Ok(())
}

fn render_provenance(artifacts: &BTreeMap<String, Vec<u8>>, profile_id: &str) -> Result<String, String> {
    let checked_scope = format!("{CHECKED_SCOPE_PREFIX}{profile_id}");
    let records = REQUIRED_PROVENANCE_ARTIFACTS
        .iter()
        .map(|artifact| {
            let bytes = artifacts
                .get(*artifact)
                .ok_or_else(|| format!("missing artifact `{artifact}` while rendering admission provenance"))?;
            let parent_artifacts = match *artifact {
                effect_closure::RECEIPT_FILE => vec![
                    COMMAND_FILE.to_owned(),
                    WORKSPACE_POLICY_ARTIFACT.to_owned(),
                    DEPENDENCY_GRAPH_ARTIFACT.to_owned(),
                    CHECK_STATUS_ARTIFACT.to_owned(),
                    CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(),
                ],
                STATUS_FILE | SUMMARY_FILE => vec![effect_closure::RECEIPT_FILE.to_owned()],
                _ => Vec::new(),
            };
            Ok(ProvenanceRecord {
                schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
                source_id: PROVENANCE_SOURCE_ID.to_owned(),
                checker_id: CHECKER_ID.to_owned(),
                evidence_tier: EvidenceTier::PolicyCheck,
                artifact_locator: (*artifact).to_owned(),
                blake3: crate::evidence::artifact_blake3_hex(bytes),
                checked_scope: checked_scope.clone(),
                verifier_identity: None,
                proof_artifact_identity: None,
                human_action: None,
                parent_artifacts,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut output = String::new();
    for record in records {
        output.push_str(
            &serde_json::to_string(&record).map_err(|error| format!("serialize admission provenance: {error}"))?,
        );
        output.push('\n');
    }
    Ok(output)
}

// r[impl octet.effect_closure.evidence.tamper]
fn verify_bundle(artifact_dir: &Path) -> Result<AdmissionVerificationReport, String> {
    verify_bundle_inner(artifact_dir, true)
}

fn verify_bundle_inner(
    artifact_dir: &Path,
    verify_strict_profile_parent: bool,
) -> Result<AdmissionVerificationReport, String> {
    let root = ArtifactRoot::open_existing(artifact_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| (file.locator, file.contents))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    if verify_strict_profile_parent {
        match verify_check_bundle(&artifact_dir.join(CHECK_DIR)) {
            Ok(()) => {}
            Err(error) => diagnostics.push(error),
        }
    }
    let receipt = parse_required::<EffectClosureReceipt>(&files, effect_closure::RECEIPT_FILE, &mut diagnostics);
    let status = parse_required::<AdmissionStatusArtifact>(&files, STATUS_FILE, &mut diagnostics);
    let policy = parse_required::<WorkspacePolicy>(&files, WORKSPACE_POLICY_ARTIFACT, &mut diagnostics);
    let graph = parse_required::<DependencyGraphArtifact>(&files, DEPENDENCY_GRAPH_ARTIFACT, &mut diagnostics);
    let check_status = parse_required::<CheckStatus>(&files, CHECK_STATUS_ARTIFACT, &mut diagnostics);
    let function_report =
        parse_required::<FunctionAddressReport>(&files, CHECK_FUNCTION_ADDRESS_ARTIFACT, &mut diagnostics);

    if let Some(receipt) = &receipt {
        diagnostics.extend(effect_closure::verify_receipt_identity(receipt));
        verify_bound_artifact_hash(receipt.inputs.command_blake3.as_str(), &files, COMMAND_FILE, &mut diagnostics);
        verify_bound_artifact_hash(
            receipt.inputs.check_status_blake3.as_str(),
            &files,
            CHECK_STATUS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.function_address_artifact_blake3.as_str(),
            &files,
            CHECK_FUNCTION_ADDRESS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.workspace_policy_blake3.as_str(),
            &files,
            WORKSPACE_POLICY_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.cargo_graph_blake3.as_str(),
            &files,
            DEPENDENCY_GRAPH_ARTIFACT,
            &mut diagnostics,
        );
        if let Some(graph) = &graph {
            if graph.schema_version != DEPENDENCY_GRAPH_SCHEMA
                || graph.cargo_lock_blake3 != receipt.inputs.cargo_lock_blake3
                || graph.target != receipt.inputs.target
                || graph.toolchain != receipt.inputs.toolchain
            {
                diagnostics.push("dependency-graph identity does not match the effect-closure receipt".to_owned());
            }
            let providers = graph.providers.iter().collect::<BTreeSet<_>>();
            if receipt.providers.iter().any(|provider| !providers.contains(provider)) {
                diagnostics.push("receipt uses a provider absent from dependency-graph.json".to_owned());
            }
        }
        if let Some(policy) = &policy {
            diagnostics.extend(effect_closure::validate_workspace_policy(policy));
        }
        if let Some(check_status) = &check_status {
            let parent_matches = check_status.exit_code == EXIT_ADMITTED
                && check_status.error_findings == 0
                && check_status.metadata.profile_name == receipt.inputs.profile_id
                && check_status.metadata.profile_hash == receipt.inputs.profile_hash
                && check_status.metadata.config_hash == receipt.inputs.config_hash
                && check_status.metadata.toolchain == receipt.inputs.toolchain;
            if !parent_matches {
                diagnostics.push("direct check status identity does not match the effect-closure receipt".to_owned());
            }
        }
        if let Some(function_report) = &function_report {
            if function_report.mode != FUNCTION_ADDRESS_MODE_DENY {
                diagnostics.push("function-address parent is not deny mode".to_owned());
            }
            match function_address_record_root(function_report) {
                Ok(root) if root == receipt.inputs.function_address_record_root => {}
                Ok(_) => diagnostics.push("function-address record root does not match the receipt".to_owned()),
                Err(error) => diagnostics.push(error),
            }
            let receipt_sources = receipt
                .sources
                .iter()
                .map(|source| FunctionAddressSourceEnvelope {
                    path: source.path.clone(),
                    source_identity: source.source_identity.clone(),
                })
                .collect::<BTreeSet<_>>();
            let report_sources = function_report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
            if receipt_sources != report_sources {
                diagnostics.push("receipt sources do not match function-address source envelopes".to_owned());
            }
        }
        if let Some(status) = &status {
            let status_matches = status == &status_from_receipt(receipt);
            if !status_matches {
                diagnostics.push("status.json is stale or does not match effect-closure.json".to_owned());
            }
        }
        if let Some(summary_bytes) = files.get(SUMMARY_FILE) {
            if summary_bytes.as_slice() != render_summary(receipt).as_bytes() {
                diagnostics.push("summary.txt is stale or does not match effect-closure.json".to_owned());
            }
        } else {
            diagnostics.push(format!("missing admission artifact `{SUMMARY_FILE}`"));
        }
        match render_provenance(&files, &receipt.inputs.profile_id) {
            Ok(expected) => match files.get(PROVENANCE_FILE) {
                Some(actual) if actual.as_slice() == expected.as_bytes() => {}
                Some(_) => {
                    diagnostics.push("provenance.jsonl is stale or contains an artifact hash mismatch".to_owned())
                }
                None => diagnostics.push(format!("missing admission artifact `{PROVENANCE_FILE}`")),
            },
            Err(error) => diagnostics.push(error),
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    let checked_artifacts = REQUIRED_PROVENANCE_ARTIFACTS.iter().map(|name| (*name).to_owned()).collect();
    Ok(AdmissionVerificationReport {
        schema_version: ADMISSION_VERIFICATION_SCHEMA.to_owned(),
        artifact_dir: artifact_dir.display().to_string(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        }
        .to_owned(),
        receipt_identity: receipt.map(|receipt| receipt.receipt_identity),
        checked_artifacts,
        diagnostics,
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    })
}

fn verify_bound_artifact_hash(
    expected: &str,
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) {
    match files.get(artifact) {
        Some(bytes) if crate::evidence::artifact_blake3_hex(bytes) == expected => {}
        Some(_) => diagnostics.push(format!("artifact `{artifact}` BLAKE3 identity does not match the receipt")),
        None => diagnostics.push(format!("missing admission artifact `{artifact}`")),
    }
}

fn parse_required<T: for<'de> Deserialize<'de>>(
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) -> Option<T> {
    let Some(bytes) = files.get(artifact) else {
        diagnostics.push(format!("missing admission artifact `{artifact}`"));
        return None;
    };
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{artifact}`: {error}"));
            None
        }
    }
}

fn checked_workspace_path(workspace_root: &Path, path: &Path, kind: &str) -> Result<PathBuf, String> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map_err(|_| format!("{kind} path must stay within workspace root: {}", path.display()))?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("{kind} path must be a non-empty workspace-relative path: {}", path.display()));
    }
    Ok(workspace_root.join(relative))
}

fn absolute_from_workspace(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn relative(path: &str) -> Result<RootRelativePath, String> {
    RootRelativePath::new(path)
}

fn read_required(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn normalized_text_bytes(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(b'\n');
    bytes
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("serialize admission artifact: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROFILE: &str = "deterministic-core";
    const TEST_HASH: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // r[verify octet.effect_closure.dependency_summaries.target]
    #[test]
    fn metadata_feature_selection_is_narrow_and_deterministic() {
        let args = vec![
            "--lib".to_owned(),
            "--features".to_owned(),
            "serde,proof".to_owned(),
            "--all-targets".to_owned(),
            "--no-default-features".to_owned(),
        ];
        assert_eq!(metadata_feature_args(&args), vec!["--features", "serde,proof", "--no-default-features"]);
        assert!(selected_target(&args).is_err());
        assert_eq!(
            selected_target(&["--target=x86_64-unknown-linux-gnu".to_owned()]).expect("explicit target"),
            "x86_64-unknown-linux-gnu"
        );
        assert!(selected_target(&["--target".to_owned(), "a".to_owned(), "--target=b".to_owned()]).is_err());
    }

    #[test]
    fn workspace_path_validation_rejects_escape_and_accepts_relative() {
        let root = Path::new("/workspace");
        assert_eq!(
            checked_workspace_path(root, Path::new("policy.json"), "policy").expect("relative policy"),
            PathBuf::from("/workspace/policy.json")
        );
        assert!(checked_workspace_path(root, Path::new("../policy.json"), "policy").is_err());
        assert!(checked_workspace_path(root, Path::new("/outside/policy.json"), "policy").is_err());
    }

    // r[verify octet.effect_closure.evidence.visible]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn complete_admission_bundle_verifies_and_tampering_is_invalid() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(CHECK_DIR)).expect("create check dir");
        let command_bytes = normalized_text_bytes("cargo octet admit --profile deterministic-core --closure");
        let source = "fn root() {}\n";
        let source_identity = crate::determinism::full_hash(source);
        let source_path = "src/lib.rs";
        let check_status_bytes = pretty_json_bytes(&serde_json::json!({
            "status": "clean",
            "exit_code": EXIT_ADMITTED,
            "metadata": {
                "rustc_version": "rustc-test",
                "toolchain": "test-toolchain",
                "profile_name": TEST_PROFILE,
                "profile_hash": TEST_HASH,
                "config_hash": TEST_HASH
            },
            "error_findings": 0
        }))
        .expect("check status JSON");
        let function_address_bytes = pretty_json_bytes(&serde_json::json!({
            "mode": FUNCTION_ADDRESS_MODE_DENY,
            "source_envelopes": [{
                "path": source_path,
                "source_identity": source_identity
            }],
            "records": [{
                "path": source_path,
                "name": "root",
                "function_address": TEST_HASH,
                "source_identity": source_identity
            }],
            "evidence_chain_handoff": {
                "canonical_roots": [{
                    "kind": FUNCTION_ADDRESS_ROOT_KIND,
                    "digest": TEST_HASH
                }]
            }
        }))
        .expect("function-address JSON");
        let policy = WorkspacePolicy {
            schema_version: effect_closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
            profile_id: TEST_PROFILE.to_owned(),
            function_address_baseline: "function-addresses.json".to_owned(),
            module_roles: vec![effect_closure::ModuleRoleAssignment {
                path: source_path.to_owned(),
                role: effect_closure::ModuleRole::Core,
            }],
            call_summaries: Vec::new(),
            non_claims: vec!["bounded test policy".to_owned()],
        };
        let policy_bytes = pretty_json_bytes(&policy).expect("policy JSON");
        let graph = DependencyGraphArtifact {
            schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
            target: "test-target".to_owned(),
            toolchain: "test-toolchain".to_owned(),
            rustc_version: "rustc-test".to_owned(),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            providers: Vec::new(),
        };
        let graph_bytes = pretty_json_bytes(&graph).expect("graph JSON");
        let receipt = effect_closure::compute_closure(ClosureInput {
            profile_id: TEST_PROFILE.to_owned(),
            profile_hash: TEST_HASH.to_owned(),
            config_hash: TEST_HASH.to_owned(),
            command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
            check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
            function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
            function_address_record_root: TEST_HASH.to_owned(),
            workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&policy_bytes),
            cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&graph_bytes),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            target: "test-target".to_owned(),
            toolchain: "test-toolchain".to_owned(),
            sources: vec![SourceEnvelope {
                path: source_path.to_owned(),
                source: source.to_owned(),
                source_identity: source_identity.clone(),
            }],
            function_addresses: vec![FunctionAddressRecord {
                path: source_path.to_owned(),
                name: "root".to_owned(),
                function_address: TEST_HASH.to_owned(),
                source_identity,
            }],
            resolved_providers: Vec::new(),
            catalog: effect_closure::embedded_catalog().expect("catalog"),
            policy,
        });
        assert_eq!(receipt.status, AdmissionStatus::Admitted);

        let receipt_bytes = pretty_json_bytes(&receipt).expect("receipt JSON");
        let status_bytes = pretty_json_bytes(&status_from_receipt(&receipt)).expect("status JSON");
        let summary_bytes = render_summary(&receipt).into_bytes();
        let mut artifacts = BTreeMap::from([
            (COMMAND_FILE.to_owned(), command_bytes),
            (STATUS_FILE.to_owned(), status_bytes),
            (SUMMARY_FILE.to_owned(), summary_bytes),
            (effect_closure::RECEIPT_FILE.to_owned(), receipt_bytes),
            (WORKSPACE_POLICY_ARTIFACT.to_owned(), policy_bytes),
            (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), graph_bytes),
            (CHECK_STATUS_ARTIFACT.to_owned(), check_status_bytes),
            (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), function_address_bytes),
        ]);
        let provenance = render_provenance(&artifacts, TEST_PROFILE).expect("provenance");
        artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());
        for (locator, bytes) in &artifacts {
            let path = temp.path().join(locator);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create artifact parent");
            }
            fs::write(path, bytes).expect("write artifact");
        }

        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);
        let incomplete_parent = verify_bundle(temp.path()).expect("parent verification report");
        assert_eq!(incomplete_parent.status, STATUS_INVALID);
        assert!(
            incomplete_parent
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("strict-profile parent artifact verification failed"))
        );

        fs::write(temp.path().join(SUMMARY_FILE), b"tampered\n").expect("tamper summary");
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("summary.txt is stale")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("provenance.jsonl is stale")));
    }
}
