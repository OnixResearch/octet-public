use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION;
use octet_architecture_ir::CollectorCompilationUnit;
use octet_architecture_ir::CompilationUnit;
use octet_architecture_ir::CompilerArchitectureCollectorContext;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CompilerArchitectureShard;
use octet_architecture_ir::CompilerValueFlowSelection;
use octet_architecture_ir::ExpectedCompilationUnit;
use octet_architecture_ir::InvalidCompilerArchitectureShard;
use octet_architecture_ir::NormalizationInput;
use octet_architecture_ir::SourceRoleScope;
use octet_standards::project_architecture_gate::ArchitectureCargoEdge;
use octet_standards::project_architecture_gate::ArchitectureCargoGraph;
use octet_standards::project_architecture_gate::PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION;
use octet_standards::project_architecture_gate::architecture_cargo_graph_identity;
use octet_standards::project_architecture_policy::ArchitectureRole;
use octet_standards::project_architecture_policy::ArchitectureScopeKind;
use serde::Deserialize;

use crate::config::SelectedProjectArchitecturePolicy;

pub(crate) const ARCHITECTURE_SHARD_DIRECTORY: &str = ".octet/compiler-architecture/shards";
pub(crate) const ARCHITECTURE_IR_ARTIFACT_NAME: &str = "compiler-architecture-ir.json";
pub(crate) const ARCHITECTURE_COVERAGE_ARTIFACT_NAME: &str = "compiler-architecture-coverage.json";
pub(crate) const ARCHITECTURE_ISSUES_ARTIFACT_NAME: &str = "compiler-architecture-issues.json";
pub(crate) const ARCHITECTURE_RECEIPT_ARTIFACT_NAME: &str = "compiler-architecture-receipt.json";
const CARGO_METADATA_SUBCOMMAND: &str = "metadata";
const FORMAT_VERSION_FLAG: &str = "--format-version";
const METADATA_FORMAT_VERSION: &str = "1";
const LOCKED_FLAG: &str = "--locked";
const LIBRARY_TARGET_KINDS: &[&str] = &["lib", "rlib", "dylib", "cdylib", "staticlib"];
const CUSTOM_BUILD_TARGET_KIND: &str = "custom-build";

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    resolve: Option<CargoResolve>,
}

#[derive(Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoResolveNode>,
}

#[derive(Debug, Deserialize)]
struct CargoResolveNode {
    id: String,
    deps: Vec<CargoResolvedDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoResolvedDependency {
    pkg: String,
    dep_kinds: Vec<CargoResolvedDependencyKind>,
}

#[derive(Debug, Deserialize)]
struct CargoResolvedDependencyKind {
    kind: Option<String>,
    target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    manifest_path: String,
    targets: Vec<CargoTarget>,
    dependencies: Vec<CargoDependency>,
}

#[derive(Debug, Deserialize)]
struct CargoDependency {
    name: String,
    kind: Option<String>,
    optional: bool,
    uses_default_features: bool,
    features: Vec<String>,
    target: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
    kind: Vec<String>,
    crate_types: Vec<String>,
}

pub(crate) struct PreparedCollector {
    pub context_path: PathBuf,
    pub shard_directory: PathBuf,
    pub policy_blake3: String,
    pub limits: octet_architecture_ir::ArchitectureCollectionLimits,
    pub expected_units: Vec<ExpectedCompilationUnit>,
    pub cargo_graph: ArchitectureCargoGraph,
    pub non_claims: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct ArchitectureArtifacts {
    pub architecture_ir: String,
    pub coverage: String,
    pub issues: String,
    pub receipt: String,
    pub incomplete_error: Option<String>,
}

pub(crate) fn prepare_collector(
    workspace_root: &Path,
    policy: &SelectedProjectArchitecturePolicy,
    toolchain: &str,
    collector_path: &Path,
    scope_args: &[String],
    cargo_check_args: &[String],
) -> Result<PreparedCollector, String> {
    let package_roles = package_roles(policy)?;
    let source_roles = source_roles(policy)?;
    let selected_packages = selected_packages(scope_args, &policy.policy.production.packages)?;
    let (toolchain_blake3, host_triple) = toolchain_identity(toolchain)?;
    let collector_blake3 = file_blake3(collector_path, "compiler architecture collector")?;
    let target = selected_policy_target(cargo_check_args, &policy.policy.production.targets, &host_triple)?;
    let (metadata, cargo_graph_blake3) = load_locked_metadata(workspace_root, toolchain, target)?;
    let lockfile_bytes = fs::read(workspace_root.join("Cargo.lock"))
        .map_err(|error| format!("read Cargo.lock for architecture identity: {error}"))?;
    let lockfile_blake3 = blake3::hash(&lockfile_bytes).to_hex().to_string();
    let cargo_graph = resolved_architecture_cargo_graph(
        &metadata,
        &selected_packages,
        &lockfile_blake3,
        &cargo_graph_blake3,
        target,
    )?;
    let invocation_blake3 = octet_architecture_ir::canonical_identity(
        "octet.compiler-architecture-ir.invocation/v1",
        &(scope_args, cargo_check_args, &lockfile_blake3, &cargo_graph_blake3),
    )?;

    let mut selected_units = Vec::new();
    for package in metadata.packages {
        if !selected_packages.contains(package.name.as_str()) {
            continue;
        }
        let Some(role) = package_roles.get(package.name.as_str()).copied() else {
            continue;
        };
        let manifest_dir = Path::new(&package.manifest_path)
            .parent()
            .ok_or_else(|| format!("Cargo package manifest has no parent: {}", package.manifest_path))?;
        let manifest_dir = manifest_dir
            .strip_prefix(workspace_root)
            .map_err(|_| format!("Cargo package manifest is outside workspace: {}", package.manifest_path))?;
        let manifest_dir = normalized_relative(manifest_dir)?;
        let source_blake3 = package_source_identity(
            workspace_root,
            &manifest_dir,
            policy.policy.collection_limits.max_spans,
            policy.policy.collection_limits.max_artifact_bytes,
        )?;
        for cargo_target in package.targets {
            if cargo_target.kind.iter().any(|kind| kind == CUSTOM_BUILD_TARGET_KIND) {
                continue;
            }
            if !cargo_target.kind.iter().any(|kind| LIBRARY_TARGET_KINDS.contains(&kind.as_str())) {
                continue;
            }
            let mut features = target.features.clone();
            features.sort();
            features.dedup();
            let mut crate_types = canonical_rustc_crate_types(&cargo_target.kind, cargo_target.crate_types);
            crate_types.sort();
            crate_types.dedup();
            let mut unit = CompilationUnit {
                unit_id: String::new(),
                package_name: package.name.clone(),
                package_version: package.version.clone(),
                target_name: cargo_target.name.clone(),
                target_kind: cargo_target.kind.join("+"),
                crate_types,
                target_triple: target.triple.clone(),
                features,
                default_features: target.default_features,
                role,
                toolchain_blake3: toolchain_blake3.clone(),
                collector_blake3: collector_blake3.clone(),
                policy_blake3: policy.policy_blake3.clone(),
                source_blake3: source_blake3.clone(),
                invocation_blake3: invocation_blake3.clone(),
            };
            unit.unit_id = octet_architecture_ir::compilation_unit_identity(&unit);
            let package_source_root = if manifest_dir.is_empty() {
                "src".to_owned()
            } else {
                format!("{manifest_dir}/src")
            };
            let unit_source_roles = source_roles
                .iter()
                .filter(|scope| path_scopes_overlap(&scope.source_scope, &package_source_root))
                .cloned()
                .collect::<Vec<_>>();
            if unit_source_roles.is_empty() {
                return Err(format!("package `{}` has no explicit compiler source-role scope", package.name));
            }
            if selected_units.len() >= policy.policy.collection_limits.max_units {
                return Err(format!(
                    "selected compiler architecture units exceed policy limit {}",
                    policy.policy.collection_limits.max_units
                ));
            }
            selected_units.push(CollectorCompilationUnit {
                rustc_crate_name: cargo_target.name.replace('-', "_"),
                manifest_dir: manifest_dir.clone(),
                source_roles: unit_source_roles,
                unit,
            });
        }
    }
    selected_units.sort_by(|left, right| left.unit.unit_id.cmp(&right.unit.unit_id));
    if selected_units.is_empty() {
        return Err("activated architecture policy selected no Cargo library compilation units".to_owned());
    }
    let shard_directory = workspace_root.join(ARCHITECTURE_SHARD_DIRECTORY);
    prepare_shard_directory(
        &shard_directory,
        &expected_unit_ids(&selected_units),
        policy.policy.collection_limits.max_shards,
        policy.policy.collection_limits.max_shard_bytes,
    )?;
    let context_path = shard_directory
        .parent()
        .expect("architecture shard directory has a parent")
        .join("collector-context.json");
    let expected_units = selected_units
        .iter()
        .map(|selected| ExpectedCompilationUnit {
            unit: selected.unit.clone(),
        })
        .collect::<Vec<_>>();
    let context = CompilerArchitectureCollectorContext {
        schema_version: COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION.to_owned(),
        output_directory: shard_directory.display().to_string(),
        limits: policy.policy.collection_limits.clone(),
        selected_units,
        value_flow_selections: protected_value_flow_selections(&policy.policy),
        non_claims: policy.policy.non_claims.clone(),
    };
    let context_bytes = serde_json::to_vec_pretty(&context)
        .map_err(|error| format!("serialize compiler architecture collector context: {error}"))?;
    fs::write(&context_path, context_bytes).map_err(|error| {
        format!("write compiler architecture collector context {}: {error}", context_path.display())
    })?;

    Ok(PreparedCollector {
        context_path,
        shard_directory,
        policy_blake3: policy.policy_blake3.clone(),
        limits: policy.policy.collection_limits.clone(),
        expected_units,
        cargo_graph,
        non_claims: policy.policy.non_claims.clone(),
    })
}

fn protected_value_flow_selections(
    policy: &octet_standards::project_architecture_policy::ProjectArchitecturePolicy,
) -> Vec<CompilerValueFlowSelection> {
    let mut selections = policy
        .outbound_effects
        .iter()
        .filter_map(|classification| classification.protected.as_ref().map(|protocol| (classification, protocol)))
        .map(|(classification, protocol)| {
            let mut tracked_callee_definitions = classification.executor_methods.clone();
            tracked_callee_definitions.extend(protocol.receipt_constructors.clone());
            tracked_callee_definitions.extend(protocol.failure_receipt_constructors.clone());
            tracked_callee_definitions.sort();
            tracked_callee_definitions.dedup();
            let mut tracked_type_definitions = protocol.authority_input_types.clone();
            tracked_type_definitions.extend(protocol.authorized_output_types.clone());
            tracked_type_definitions.extend(protocol.witness_binding_types.clone());
            tracked_type_definitions.extend([
                protocol.witness_type.clone(),
                protocol.plan_type.clone(),
                protocol.attempt_type.clone(),
                protocol.observed_outcome_type.clone(),
                protocol.receipt_type.clone(),
            ]);
            tracked_type_definitions.sort();
            tracked_type_definitions.dedup();
            let mut root_origin_definitions = protocol.decision_functions.clone();
            root_origin_definitions.extend(protocol.success_observation_constructors.clone());
            root_origin_definitions.extend(protocol.failure_observation_constructors.clone());
            root_origin_definitions.sort();
            root_origin_definitions.dedup();
            let mut propagation_definitions = protocol.witness_constructors.clone();
            propagation_definitions.extend(protocol.receipt_constructors.clone());
            propagation_definitions.extend(protocol.failure_receipt_constructors.clone());
            propagation_definitions.sort();
            propagation_definitions.dedup();
            CompilerValueFlowSelection {
                tracked_callee_definitions,
                tracked_type_definitions,
                root_origin_definitions,
                propagation_definitions,
            }
        })
        .collect::<Vec<_>>();
    selections.sort_by(|left, right| {
        left.tracked_callee_definitions
            .cmp(&right.tracked_callee_definitions)
            .then_with(|| left.tracked_type_definitions.cmp(&right.tracked_type_definitions))
            .then_with(|| left.root_origin_definitions.cmp(&right.root_origin_definitions))
            .then_with(|| left.propagation_definitions.cmp(&right.propagation_definitions))
    });
    selections.dedup();
    selections
}

pub(crate) fn normalize_collected_shards(prepared: &PreparedCollector) -> Result<ArchitectureArtifacts, String> {
    let ingestion =
        read_shards(&prepared.shard_directory, prepared.limits.max_shards, prepared.limits.max_shard_bytes)?;
    let outcome = octet_architecture_ir::normalize_architecture_shards(NormalizationInput {
        policy_blake3: prepared.policy_blake3.clone(),
        limits: prepared.limits.clone(),
        expected_units: prepared.expected_units.clone(),
        shards: ingestion.shards,
        invalid_shards: ingestion.invalid_shards,
        non_claims: prepared.non_claims.clone(),
    })?;
    let architecture_ir = render_optional_ir(outcome.architecture_ir.as_ref())?;
    let coverage = render_json(&outcome.coverage, "compiler architecture coverage")?;
    let issues = render_json(&outcome.issues, "compiler architecture issues")?;
    let receipt = render_json(&outcome.receipt, "compiler architecture receipt")?;
    let incomplete_error = (outcome.receipt.status != octet_architecture_ir::CollectionStatus::Complete).then(|| {
        let issue_summary = outcome
            .issues
            .iter()
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect::<Vec<_>>()
            .join("; ");
        format!("compiler architecture collection is incomplete: {issue_summary}")
    });
    Ok(ArchitectureArtifacts {
        architecture_ir,
        coverage,
        issues,
        receipt,
        incomplete_error,
    })
}

fn canonical_rustc_crate_types(target_kinds: &[String], crate_types: Vec<String>) -> Vec<String> {
    crate_types
        .into_iter()
        .map(|crate_type| {
            if crate_type == "lib" && target_kinds.iter().any(|kind| kind == "lib") {
                "rlib".to_owned()
            } else {
                crate_type
            }
        })
        .collect()
}

fn file_blake3(path: &Path, label: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {label} {}: {error}", path.display()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

fn expected_unit_ids(selected_units: &[CollectorCompilationUnit]) -> std::collections::BTreeSet<&str> {
    selected_units.iter().map(|selected| selected.unit.unit_id.as_str()).collect()
}

fn prepare_shard_directory(
    directory: &Path,
    expected_unit_ids: &std::collections::BTreeSet<&str>,
    max_shards: usize,
    max_shard_bytes: usize,
) -> Result<(), String> {
    fs::create_dir_all(directory)
        .map_err(|error| format!("create architecture shard directory {}: {error}", directory.display()))?;
    let (paths, count_overflow) = bounded_shard_paths(directory, max_shards)?;
    if count_overflow {
        clear_shard_directory(directory)?;
        return Ok(());
    }
    let mut retained_unit_ids = std::collections::BTreeSet::new();
    for path in paths {
        let cached = read_cached_shard(&path, max_shard_bytes);
        let keep = cached.is_ok_and(|shard| {
            expected_unit_ids.contains(shard.unit.unit_id.as_str()) && retained_unit_ids.insert(shard.unit.unit_id)
        });
        if !keep {
            remove_shard_entry(&path)?;
        }
    }
    remove_non_json_shard_entries(directory)?;
    Ok(())
}

fn clear_shard_directory(directory: &Path) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("read architecture shard directory {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry.map_err(|error| format!("read architecture shard directory entry: {error}"))?.path();
        remove_shard_entry(&path)?;
    }
    Ok(())
}

fn remove_non_json_shard_entries(directory: &Path) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("read architecture shard directory {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry.map_err(|error| format!("read architecture shard directory entry: {error}"))?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            remove_shard_entry(&path)?;
        }
    }
    Ok(())
}

fn remove_shard_entry(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|error| format!("remove stale architecture shard directory {}: {error}", path.display()))
    } else {
        fs::remove_file(path).map_err(|error| format!("remove stale architecture shard {}: {error}", path.display()))
    }
}

fn read_cached_shard(path: &Path, max_shard_bytes: usize) -> Result<CompilerArchitectureShard, String> {
    if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
        return Err("cached architecture shard is not JSON".to_owned());
    }
    let metadata = fs::metadata(path)
        .map_err(|error| format!("read cached architecture shard metadata {}: {error}", path.display()))?;
    let max_shard_bytes = u64::try_from(max_shard_bytes)
        .map_err(|error| format!("convert cached architecture shard byte limit: {error}"))?;
    if metadata.len() > max_shard_bytes {
        return Err(format!("cached architecture shard exceeds byte limit: {}", path.display()));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("read cached architecture shard {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse cached architecture shard {}: {error}", path.display()))
}

struct ShardIngestion {
    shards: Vec<CompilerArchitectureShard>,
    invalid_shards: Vec<InvalidCompilerArchitectureShard>,
}

fn read_shards(directory: &Path, max_shards: usize, max_shard_bytes: usize) -> Result<ShardIngestion, String> {
    let (paths, count_overflow) = bounded_shard_paths(directory, max_shards)?;
    if count_overflow {
        let limit = max_shards.to_string();
        return Ok(ShardIngestion {
            shards: Vec::new(),
            invalid_shards: vec![InvalidCompilerArchitectureShard {
                shard_id: octet_architecture_ir::framed_blake3(
                    "octet.compiler-architecture-ir.shard-count-overflow/v1",
                    &[limit.as_bytes()],
                ),
                code: "shard-count-overflow".to_owned(),
                message: format!("compiler architecture shard count exceeds policy limit {max_shards}"),
            }],
        });
    }
    let max_shard_bytes = u64::try_from(max_shard_bytes)
        .map_err(|error| format!("convert compiler architecture shard byte limit: {error}"))?;
    let mut shards = Vec::with_capacity(paths.len());
    let mut invalid_shards = Vec::with_capacity(paths.len());
    for path in paths {
        let shard_id = shard_file_identity(&path)?;
        let metadata = fs::metadata(&path)
            .map_err(|error| format!("read compiler architecture shard metadata {}: {error}", path.display()))?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("compiler architecture shard file name is not UTF-8: {}", path.display()))?;
        if metadata.len() > max_shard_bytes {
            invalid_shards.push(InvalidCompilerArchitectureShard {
                shard_id,
                code: "shard-byte-overflow".to_owned(),
                message: format!(
                    "compiler architecture shard {file_name} is {} bytes; policy limit is {max_shard_bytes}",
                    metadata.len()
                ),
            });
            continue;
        }
        let bytes =
            fs::read(&path).map_err(|error| format!("read compiler architecture shard {}: {error}", path.display()))?;
        match serde_json::from_slice(&bytes) {
            Ok(shard) => shards.push(shard),
            Err(_) => invalid_shards.push(InvalidCompilerArchitectureShard {
                shard_id,
                code: "shard-malformed".to_owned(),
                message: format!("compiler architecture shard {file_name} is malformed"),
            }),
        }
    }
    Ok(ShardIngestion { shards, invalid_shards })
}

fn bounded_shard_paths(directory: &Path, max_shards: usize) -> Result<(Vec<PathBuf>, bool), String> {
    let overflow_limit = max_shards
        .checked_add(1)
        .ok_or_else(|| "compiler architecture shard admission limit overflowed usize".to_owned())?;
    let mut paths = Vec::new();
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("read compiler architecture shard directory {}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry.map_err(|error| format!("read compiler architecture shard entry: {error}"))?.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        paths.push(path);
        if paths.len() == overflow_limit {
            return Ok((Vec::new(), true));
        }
    }
    paths.sort();
    Ok((paths, false))
}

fn shard_file_identity(path: &Path) -> Result<String, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("compiler architecture shard file name is not UTF-8: {}", path.display()))?;
    Ok(octet_architecture_ir::framed_blake3("octet.compiler-architecture-ir.ingested-shard/v1", &[
        file_name.as_bytes(),
    ]))
}

fn render_optional_ir(value: Option<&CompilerArchitectureIr>) -> Result<String, String> {
    match value {
        Some(value) => render_json(value, "compiler architecture IR"),
        None => Ok("null\n".to_owned()),
    }
}

fn render_json<T: serde::Serialize>(value: &T, label: &str) -> Result<String, String> {
    let mut rendered = serde_json::to_string_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn load_locked_metadata(
    workspace_root: &Path,
    toolchain: &str,
    target: &octet_standards::project_architecture_policy::ArchitectureTarget,
) -> Result<(CargoMetadata, String), String> {
    let mut command = Command::new("cargo");
    command
        .arg(CARGO_METADATA_SUBCOMMAND)
        .arg(FORMAT_VERSION_FLAG)
        .arg(METADATA_FORMAT_VERSION)
        .arg(LOCKED_FLAG)
        .arg("--filter-platform")
        .arg(&target.triple)
        .current_dir(workspace_root)
        .env("RUSTUP_TOOLCHAIN", toolchain);
    if !target.features.is_empty() {
        command.arg("--features").arg(target.features.join(","));
    }
    if !target.default_features {
        command.arg("--no-default-features");
    }
    let output = command.output().map_err(|error| format!("run locked cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!("locked cargo metadata failed: {}", String::from_utf8_lossy(&output.stderr).trim()));
    }
    let metadata: CargoMetadata =
        serde_json::from_slice(&output.stdout).map_err(|error| format!("parse locked Cargo metadata: {error}"))?;
    let cargo_graph_blake3 = octet_architecture_ir::canonical_identity(
        "octet.compiler-architecture-ir.cargo-graph/v1",
        &canonical_cargo_graph(&metadata),
    )?;
    Ok((metadata, cargo_graph_blake3))
}

fn resolved_architecture_cargo_graph(
    metadata: &CargoMetadata,
    selected_packages: &std::collections::BTreeSet<&str>,
    cargo_lock_blake3: &str,
    cargo_metadata_blake3: &str,
    target: &octet_standards::project_architecture_policy::ArchitectureTarget,
) -> Result<ArchitectureCargoGraph, String> {
    let resolve = metadata.resolve.as_ref().ok_or_else(|| "locked Cargo metadata omitted resolve nodes".to_owned())?;
    let package_names = metadata
        .packages
        .iter()
        .map(|package| (package.id.as_str(), package.name.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for node in &resolve.nodes {
        let caller_package = package_names
            .get(node.id.as_str())
            .ok_or_else(|| format!("Cargo resolve node has unknown package ID `{}`", node.id))?;
        if !selected_packages.contains(*caller_package) {
            continue;
        }
        for dependency in &node.deps {
            let callee_package = package_names
                .get(dependency.pkg.as_str())
                .ok_or_else(|| format!("Cargo dependency has unknown package ID `{}`", dependency.pkg))?;
            for kind in &dependency.dep_kinds {
                if kind.kind.as_deref() == Some("dev") {
                    continue;
                }
                edges.push(ArchitectureCargoEdge {
                    caller_package: (*caller_package).to_owned(),
                    callee_package: (*callee_package).to_owned(),
                    edge_kind: kind.kind.clone().unwrap_or_else(|| "normal".to_owned()),
                    target_predicate: kind.target.clone(),
                });
            }
        }
    }
    edges.sort();
    edges.dedup();
    let mut selected_package_names = selected_packages.iter().map(|package| (*package).to_owned()).collect::<Vec<_>>();
    selected_package_names.sort();
    let mut features = target.features.clone();
    features.sort();
    features.dedup();
    let mut graph = ArchitectureCargoGraph {
        schema_version: PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION.to_owned(),
        cargo_graph_id: String::new(),
        cargo_lock_blake3: cargo_lock_blake3.to_owned(),
        cargo_metadata_blake3: cargo_metadata_blake3.to_owned(),
        selected_packages: selected_package_names,
        target_triple: target.triple.clone(),
        features,
        default_features: target.default_features,
        edges,
    };
    graph.cargo_graph_id = architecture_cargo_graph_identity(&graph);
    Ok(graph)
}

#[derive(Eq, Ord, PartialEq, PartialOrd, serde::Serialize)]
struct CanonicalCargoDependency {
    name: String,
    kind: Option<String>,
    optional: bool,
    uses_default_features: bool,
    features: Vec<String>,
    target: Option<String>,
}

#[derive(serde::Serialize)]
struct CanonicalCargoPackage {
    name: String,
    version: String,
    targets: Vec<(String, Vec<String>, Vec<String>)>,
    dependencies: Vec<CanonicalCargoDependency>,
}

fn canonical_cargo_graph(metadata: &CargoMetadata) -> Vec<CanonicalCargoPackage> {
    let mut packages = metadata
        .packages
        .iter()
        .map(|package| {
            let mut targets = package
                .targets
                .iter()
                .map(|target| {
                    let mut kinds = target.kind.clone();
                    let mut crate_types = target.crate_types.clone();
                    kinds.sort();
                    kinds.dedup();
                    crate_types.sort();
                    crate_types.dedup();
                    (target.name.clone(), kinds, crate_types)
                })
                .collect::<Vec<_>>();
            targets.sort();
            let mut dependencies = package
                .dependencies
                .iter()
                .map(|dependency| {
                    let mut features = dependency.features.clone();
                    features.sort();
                    features.dedup();
                    CanonicalCargoDependency {
                        name: dependency.name.clone(),
                        kind: dependency.kind.clone(),
                        optional: dependency.optional,
                        uses_default_features: dependency.uses_default_features,
                        features,
                        target: dependency.target.clone(),
                    }
                })
                .collect::<Vec<_>>();
            dependencies.sort();
            CanonicalCargoPackage {
                name: package.name.clone(),
                version: package.version.clone(),
                targets,
                dependencies,
            }
        })
        .collect::<Vec<_>>();
    packages.sort_by(|left, right| left.name.cmp(&right.name).then_with(|| left.version.cmp(&right.version)));
    packages
}

fn toolchain_identity(toolchain: &str) -> Result<(String, String), String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .output()
        .map_err(|error| format!("run rustc -vV for architecture identity: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "rustc -vV failed for architecture identity: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let version_text =
        String::from_utf8(output.stdout.clone()).map_err(|error| format!("rustc -vV output is not UTF-8: {error}"))?;
    let host_triple = version_text
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or_else(|| "rustc -vV output has no host triple".to_owned())?
        .to_owned();
    let identity = octet_architecture_ir::framed_blake3("octet.compiler-architecture-ir.toolchain/v1", &[
        toolchain.as_bytes(),
        &output.stdout,
    ]);
    Ok((identity, host_triple))
}

fn path_scopes_overlap(left: &str, right: &str) -> bool {
    left == right
        || left.strip_prefix(right).is_some_and(|suffix| suffix.starts_with('/'))
        || right.strip_prefix(left).is_some_and(|suffix| suffix.starts_with('/'))
}

fn selected_packages<'a>(
    scope_args: &[String],
    production_packages: &'a [String],
) -> Result<std::collections::BTreeSet<&'a str>, String> {
    let mut selected = std::collections::BTreeSet::new();
    let mut index = 0;
    let mut workspace_selected = false;
    while index < scope_args.len() {
        match scope_args[index].as_str() {
            "--workspace" => {
                workspace_selected = true;
                index += 1;
            }
            "-p" | "--package" => {
                let package = scope_args
                    .get(index + 1)
                    .ok_or_else(|| format!("missing package name after `{}`", scope_args[index]))?;
                let Some(production) = production_packages.iter().find(|candidate| *candidate == package) else {
                    return Err(format!(
                        "Cargo invocation selects package `{package}` outside architecture production scope"
                    ));
                };
                selected.insert(production.as_str());
                index += 2;
            }
            argument => return Err(format!("unsupported architecture collection scope argument `{argument}`")),
        }
    }
    if workspace_selected {
        selected.extend(production_packages.iter().map(String::as_str));
    }
    if selected.len() != production_packages.len() {
        let missing = production_packages
            .iter()
            .filter(|package| !selected.contains(package.as_str()))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(format!("Cargo invocation omits architecture production packages: {missing}"));
    }
    Ok(selected)
}

fn selected_policy_target<'a>(
    cargo_check_args: &[String],
    targets: &'a [octet_standards::project_architecture_policy::ArchitectureTarget],
    host_triple: &str,
) -> Result<&'a octet_standards::project_architecture_policy::ArchitectureTarget, String> {
    let requested_target = option_value(cargo_check_args, "--target").unwrap_or(host_triple);
    let matches = targets.iter().filter(|target| target.triple == requested_target).collect::<Vec<_>>();
    let [target] = matches.as_slice() else {
        return Err(format!(
            "Cargo invocation target `{requested_target}` must match exactly one architecture policy target"
        ));
    };
    validate_cargo_selection(cargo_check_args, target)?;
    Ok(target)
}

fn validate_cargo_selection(
    cargo_check_args: &[String],
    target: &octet_standards::project_architecture_policy::ArchitectureTarget,
) -> Result<(), String> {
    if !cargo_check_args.iter().any(|argument| argument == "--lib") {
        return Err("architecture collection requires `--lib` to bind expected library units".to_owned());
    }
    let requested_target = option_value(cargo_check_args, "--target");
    if let Some(requested_target) = requested_target
        && requested_target != target.triple
    {
        return Err(format!(
            "Cargo invocation target `{requested_target}` does not match architecture policy target `{}`",
            target.triple
        ));
    }
    let requested_features = option_value(cargo_check_args, "--features")
        .map(|value| value.split(',').filter(|feature| !feature.is_empty()).map(str::to_owned).collect::<Vec<_>>())
        .unwrap_or_default();
    let mut requested_features = requested_features;
    requested_features.sort();
    requested_features.dedup();
    let mut expected_features = target.features.clone();
    expected_features.sort();
    expected_features.dedup();
    if requested_features != expected_features {
        return Err(format!(
            "Cargo invocation features {:?} do not match architecture policy features {:?}",
            requested_features, expected_features
        ));
    }
    let no_default_features = cargo_check_args.iter().any(|argument| argument == "--no-default-features");
    if target.default_features == no_default_features {
        return Err(format!(
            "Cargo invocation default-feature selection does not match architecture policy default_features={}",
            target.default_features
        ));
    }
    Ok(())
}

fn option_value<'a>(arguments: &'a [String], option: &str) -> Option<&'a str> {
    let prefix = format!("{option}=");
    arguments.iter().enumerate().find_map(|(index, argument)| {
        if argument == option {
            arguments.get(index + 1).map(String::as_str)
        } else {
            argument.strip_prefix(&prefix)
        }
    })
}

fn source_roles(selected: &SelectedProjectArchitecturePolicy) -> Result<Vec<SourceRoleScope>, String> {
    let mut roles = Vec::with_capacity(selected.policy.roles.len());
    for assignment in &selected.policy.roles {
        if assignment.scope_kind != ArchitectureScopeKind::Source || !assignment.role.is_production() {
            continue;
        }
        roles.push(SourceRoleScope {
            source_scope: assignment.scope.clone(),
            role: convert_role(assignment.role)?,
        });
    }
    roles.sort_by(|left, right| left.source_scope.cmp(&right.source_scope));
    Ok(roles)
}

fn package_roles(
    selected: &SelectedProjectArchitecturePolicy,
) -> Result<BTreeMap<&str, CompilerArchitectureRole>, String> {
    let mut roles = BTreeMap::new();
    for assignment in &selected.policy.roles {
        if assignment.scope_kind != ArchitectureScopeKind::Package || !assignment.role.is_production() {
            continue;
        }
        let role = convert_role(assignment.role)?;
        roles.insert(assignment.scope.as_str(), role);
    }
    Ok(roles)
}

fn convert_role(role: ArchitectureRole) -> Result<CompilerArchitectureRole, String> {
    match role {
        ArchitectureRole::DomainCore => Ok(CompilerArchitectureRole::DomainCore),
        ArchitectureRole::ApplicationShell => Ok(CompilerArchitectureRole::ApplicationShell),
        ArchitectureRole::InboundAdapter => Ok(CompilerArchitectureRole::InboundAdapter),
        ArchitectureRole::OutboundAdapter => Ok(CompilerArchitectureRole::OutboundAdapter),
        ArchitectureRole::CompositionRoot => Ok(CompilerArchitectureRole::CompositionRoot),
        ArchitectureRole::Test | ArchitectureRole::Generated => {
            Err("non-production role reached production compiler selection".to_owned())
        }
    }
}

fn package_source_identity(
    workspace_root: &Path,
    relative_dir: &str,
    max_source_files: usize,
    max_source_bytes: usize,
) -> Result<String, String> {
    let package_root = workspace_root.join(relative_dir);
    if max_source_files == 0 {
        return Err("package source file limit must be positive".to_owned());
    }
    let mut files = Vec::new();
    files.push(package_root.join("Cargo.toml"));
    let mut observed_source_entries = 1usize;
    collect_rust_sources(&package_root.join("src"), &mut files, max_source_files, &mut observed_source_entries)?;
    files.sort();
    let frame_capacity = files
        .len()
        .checked_mul(2)
        .ok_or_else(|| "package source frame capacity overflowed usize".to_owned())?;
    let mut framed = Vec::with_capacity(frame_capacity);
    let mut observed_source_bytes = 0usize;
    for path in files {
        let relative = path
            .strip_prefix(workspace_root)
            .map_err(|_| format!("package source is outside workspace: {}", path.display()))?;
        let relative = normalized_relative(relative)?;
        let bytes = fs::read(&path).map_err(|error| format!("read package source {}: {error}", path.display()))?;
        observed_source_bytes = observed_source_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| "package source envelope byte count overflowed usize".to_owned())?;
        if observed_source_bytes > max_source_bytes {
            return Err(format!(
                "package source envelope is {observed_source_bytes} bytes; policy artifact limit is {max_source_bytes}"
            ));
        }
        framed.push(relative.into_bytes());
        framed.push(bytes);
    }
    let mut references = Vec::with_capacity(framed.len());
    for frame in &framed {
        references.push(frame.as_slice());
    }
    Ok(octet_architecture_ir::framed_blake3(
        "octet.compiler-architecture-ir.source-envelope/v1",
        &references,
    ))
}

fn collect_rust_sources(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    max_source_entries: usize,
    observed_source_entries: &mut usize,
) -> Result<(), String> {
    if !directory.exists() {
        return Ok(());
    }
    let mut paths = Vec::new();
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("read package source directory {}: {error}", directory.display()))?;
    for entry in entries {
        *observed_source_entries = observed_source_entries
            .checked_add(1)
            .ok_or_else(|| "package source entry count overflowed usize".to_owned())?;
        if *observed_source_entries > max_source_entries {
            return Err(format!("package source entry count exceeds span limit {max_source_entries}"));
        }
        paths.push(entry.map_err(|error| format!("read package source entry: {error}"))?.path());
    }
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, files, max_source_entries, observed_source_entries)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn normalized_relative(path: &Path) -> Result<String, String> {
    let parts = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("workspace-relative package path is not UTF-8: {}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use octet_standards::project_architecture_policy::ArchitectureTarget;

    use super::*;

    const MAX_INGESTION_FIXTURE_SHARDS: usize = 2;
    const MAX_COUNT_OVERFLOW_FIXTURE_SHARDS: usize = 3;
    const MAX_INGESTION_FIXTURE_SHARD_BYTES: usize = 8;
    const MAX_INGESTION_FIXTURE_BYTES: usize = MAX_INGESTION_FIXTURE_SHARD_BYTES + 1;
    const MAX_CACHE_FIXTURE_SHARD_BYTES: usize = 4096;
    const FIXTURE_HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const FIXTURE_HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const FIXTURE_HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

    fn packages() -> Vec<String> {
        vec!["domain".to_owned(), "application".to_owned()]
    }

    fn target() -> ArchitectureTarget {
        ArchitectureTarget {
            triple: "x86_64-unknown-linux-gnu".to_owned(),
            features: vec!["production".to_owned()],
            default_features: false,
        }
    }

    fn cached_shard(unit_id: &str, shard_id: &str) -> CompilerArchitectureShard {
        CompilerArchitectureShard {
            schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION.to_owned(),
            shard_id: shard_id.to_owned(),
            unit: CompilationUnit {
                unit_id: unit_id.to_owned(),
                package_name: "fixture".to_owned(),
                package_version: "0.1.0".to_owned(),
                target_name: "fixture".to_owned(),
                target_kind: "lib".to_owned(),
                crate_types: vec!["rlib".to_owned()],
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                features: Vec::new(),
                default_features: false,
                role: CompilerArchitectureRole::ApplicationShell,
                toolchain_blake3: FIXTURE_HASH_B.to_owned(),
                collector_blake3: FIXTURE_HASH_C.to_owned(),
                policy_blake3: FIXTURE_HASH_B.to_owned(),
                source_blake3: FIXTURE_HASH_C.to_owned(),
                invocation_blake3: FIXTURE_HASH_B.to_owned(),
            },
            collection_status: octet_architecture_ir::CollectionStatus::Complete,
            spans: Vec::new(),
            facts: Vec::new(),
            unknowns: Vec::new(),
            non_claims: vec!["whole-program-correctness".to_owned()],
        }
    }

    fn write_cached_shard(directory: &Path, file_name: &str, shard: &CompilerArchitectureShard) {
        let bytes = serde_json::to_vec(shard).expect("serialize cached shard fixture");
        fs::write(directory.join(file_name), bytes).expect("write cached shard fixture");
    }

    #[test]
    fn exact_workspace_selection_covers_all_production_packages() {
        let packages = packages();
        let selected = selected_packages(&["--workspace".to_owned()], &packages).expect("workspace selection");

        assert_eq!(selected.len(), packages.len());
    }

    #[test]
    fn partial_package_selection_fails_closed() {
        let packages = packages();
        let error = selected_packages(&["-p".to_owned(), "domain".to_owned()], &packages)
            .expect_err("partial selection must fail");

        assert!(error.contains("omits architecture production packages"));
    }

    #[test]
    fn implicit_cargo_library_kind_matches_rustc_rlib() {
        assert_eq!(canonical_rustc_crate_types(&["lib".to_owned()], vec!["lib".to_owned()]), vec!["rlib".to_owned()]);
        assert_eq!(canonical_rustc_crate_types(&["cdylib".to_owned()], vec!["cdylib".to_owned()]), vec![
            "cdylib".to_owned()
        ]);
    }

    #[test]
    fn malformed_and_oversized_shards_remain_reviewable() {
        let fixture = tempfile::tempdir().expect("create shard fixture");
        fs::write(fixture.path().join("malformed.json"), b"not json").expect("write malformed shard");
        fs::write(fixture.path().join("oversized.json"), vec![b'x'; MAX_INGESTION_FIXTURE_BYTES])
            .expect("write oversized shard");

        let ingestion = read_shards(fixture.path(), MAX_INGESTION_FIXTURE_SHARDS, MAX_INGESTION_FIXTURE_SHARD_BYTES)
            .expect("ingest invalid shards");

        assert!(ingestion.shards.is_empty());
        assert_eq!(ingestion.invalid_shards.len(), MAX_INGESTION_FIXTURE_SHARDS);
        assert!(ingestion.invalid_shards.iter().any(|shard| shard.code == "shard-malformed"));
        assert!(ingestion.invalid_shards.iter().any(|shard| shard.code == "shard-byte-overflow"));
    }

    #[test]
    fn shard_count_overflow_is_bounded_and_reviewable() {
        let fixture = tempfile::tempdir().expect("create shard fixture");
        for index in 0..MAX_COUNT_OVERFLOW_FIXTURE_SHARDS {
            fs::write(fixture.path().join(format!("{index}.json")), b"not json").expect("write shard fixture");
        }

        let ingestion = read_shards(fixture.path(), MAX_INGESTION_FIXTURE_SHARDS, MAX_INGESTION_FIXTURE_SHARD_BYTES)
            .expect("ingest shard count overflow");

        assert!(ingestion.shards.is_empty());
        assert_eq!(ingestion.invalid_shards.len(), 1);
        assert_eq!(ingestion.invalid_shards[0].code, "shard-count-overflow");
    }

    #[test]
    fn cache_preparation_keeps_at_most_one_shard_per_expected_unit() {
        let fixture = tempfile::tempdir().expect("create cache fixture");
        let first = cached_shard(FIXTURE_HASH_A, FIXTURE_HASH_B);
        let second = cached_shard(FIXTURE_HASH_A, FIXTURE_HASH_C);
        write_cached_shard(fixture.path(), "first.json", &first);
        write_cached_shard(fixture.path(), "second.json", &second);
        let expected = std::collections::BTreeSet::from([FIXTURE_HASH_A]);

        prepare_shard_directory(fixture.path(), &expected, MAX_INGESTION_FIXTURE_SHARDS, MAX_CACHE_FIXTURE_SHARD_BYTES)
            .expect("prepare duplicate cache");

        let retained = fs::read_dir(fixture.path()).expect("read prepared cache").count();
        assert_eq!(retained, 1);
    }

    #[test]
    fn cache_preparation_clears_an_over_limit_shard_set() {
        let fixture = tempfile::tempdir().expect("create overflow cache fixture");
        for index in 0..MAX_COUNT_OVERFLOW_FIXTURE_SHARDS {
            let shard = cached_shard(FIXTURE_HASH_A, FIXTURE_HASH_B);
            write_cached_shard(fixture.path(), &format!("{index}.json"), &shard);
        }
        let expected = std::collections::BTreeSet::from([FIXTURE_HASH_A]);

        prepare_shard_directory(fixture.path(), &expected, MAX_INGESTION_FIXTURE_SHARDS, MAX_CACHE_FIXTURE_SHARD_BYTES)
            .expect("prepare overflow cache");

        assert_eq!(fs::read_dir(fixture.path()).expect("read cleared cache").count(), 0);
    }

    #[test]
    fn cargo_target_features_and_defaults_must_match_policy() {
        let valid = vec![
            "--lib".to_owned(),
            "--target".to_owned(),
            "x86_64-unknown-linux-gnu".to_owned(),
            "--features=production".to_owned(),
            "--no-default-features".to_owned(),
        ];
        assert_eq!(validate_cargo_selection(&valid, &target()), Ok(()));

        let invalid = vec![
            "--lib".to_owned(),
            "--features=staging".to_owned(),
            "--no-default-features".to_owned(),
        ];
        assert!(validate_cargo_selection(&invalid, &target()).is_err());
    }
}
