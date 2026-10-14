//! Pure dependency-direction graph classification.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;

const NORMAL_DEPENDENCY_KIND: &str = "normal";
const BUILD_DEPENDENCY_KIND: &str = "build";
const DEV_DEPENDENCY_KIND: &str = "dev";
const CARGO_MANIFEST_SUFFIX: &str = "/Cargo.toml";
const RUST_LIB_SUFFIX: &str = "/src/lib.rs";

/// Dependency edge kind selected from cargo metadata.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DependencyEdgeKind {
    /// Normal runtime dependency edge.
    Normal,
    /// Build-script dependency edge.
    Build,
    /// Development/test dependency edge.
    Dev,
}

impl DependencyEdgeKind {
    /// Parses a registry config value.
    #[must_use]
    pub fn from_config_value(value: &str) -> Option<Self> {
        match value {
            NORMAL_DEPENDENCY_KIND => Some(Self::Normal),
            BUILD_DEPENDENCY_KIND => Some(Self::Build),
            DEV_DEPENDENCY_KIND => Some(Self::Dev),
            _ => None,
        }
    }

    fn from_metadata_kind(kind: Option<&str>) -> Option<Self> {
        match kind {
            None | Some(NORMAL_DEPENDENCY_KIND) => Some(Self::Normal),
            Some(BUILD_DEPENDENCY_KIND) => Some(Self::Build),
            Some(DEV_DEPENDENCY_KIND) => Some(Self::Dev),
            Some(_) => None,
        }
    }
}

/// Returns the default edge scope: normal + build dependencies.
#[must_use]
pub fn default_dependency_edge_kinds() -> BTreeSet<DependencyEdgeKind> {
    [DependencyEdgeKind::Normal, DependencyEdgeKind::Build].into_iter().collect()
}

/// A directed dependency edge in a normalized package graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyEdge {
    /// Dependency package name.
    pub package: String,
    /// Cargo dependency kind for this edge.
    pub kind: DependencyEdgeKind,
}

/// A package node in a normalized package graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyNode {
    /// Package name.
    pub package: String,
    /// Direct dependency edges from this package.
    pub dependencies: Vec<DependencyEdge>,
}

/// Parsed dependency graph keyed by package name.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DependencyGraph {
    /// Package nodes keyed by package name.
    pub nodes: BTreeMap<String, DependencyNode>,
}

/// Inputs to the pure dependency-direction classifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyDirectionInput {
    /// Normalized dependency graph.
    pub graph: DependencyGraph,
    /// Packages declared as core crates.
    pub core_packages: BTreeSet<String>,
    /// Packages declared as shell crates.
    pub shell_packages: BTreeSet<String>,
    /// Packages declared as adapter crates.
    pub adapter_packages: BTreeSet<String>,
    /// Dependency edge kinds to inspect.
    pub included_edge_kinds: BTreeSet<DependencyEdgeKind>,
}

/// Directional violation kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyDirectionViolationKind {
    /// Core package reaches an adapter package.
    CoreToAdapter,
    /// Core package depends on a non-core infrastructure package.
    CoreToExternalInfrastructure,
    /// Adapter package does not depend inward on any core package.
    AdapterMissingCore,
    /// Shell package depends outward on an adapter package.
    ShellOutwardToAdapter,
}

/// Direct-vs-transitive edge classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DependencyEdgeClassification {
    /// The violating dependency is listed directly on the dependent package.
    Direct,
    /// The violating dependency is reached through a non-core intermediate package.
    Transitive,
}

/// A dependency-direction violation returned by the pure classifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencyDirectionViolation {
    /// Violation kind.
    pub kind: DependencyDirectionViolationKind,
    /// Package that owns the violating edge or missing edge.
    pub dependent: String,
    /// Violating dependency package, when an edge exists.
    pub dependency: Option<String>,
    /// Intermediate package for transitive leakage.
    pub intermediate: Option<String>,
    /// Edge classification, when an edge exists.
    pub edge: Option<DependencyEdgeClassification>,
}

/// Cargo metadata JSON subset consumed by the dependency-direction shell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadata {
    /// Package records emitted by cargo metadata.
    #[serde(default)]
    pub packages: Vec<CargoMetadataPackage>,
    /// Resolve graph emitted by cargo metadata.
    #[serde(default)]
    pub resolve: Option<CargoMetadataResolve>,
    /// Workspace member package IDs.
    #[serde(default)]
    pub workspace_members: Vec<String>,
}

/// Cargo metadata package subset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadataPackage {
    /// Cargo package ID.
    pub id: String,
    /// Cargo package name.
    pub name: String,
    /// Manifest path.
    pub manifest_path: String,
}

/// Cargo metadata resolve subset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadataResolve {
    /// Resolve nodes.
    #[serde(default)]
    pub nodes: Vec<CargoMetadataNode>,
}

/// Cargo metadata resolve node subset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadataNode {
    /// Package ID for this node.
    pub id: String,
    /// Resolved dependency edges.
    #[serde(default)]
    pub deps: Vec<CargoMetadataDependency>,
}

/// Cargo metadata resolved dependency subset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadataDependency {
    /// Dependency package ID.
    pub pkg: String,
    /// Dependency kinds for this edge.
    #[serde(default)]
    pub dep_kinds: Vec<CargoMetadataDependencyKind>,
}

/// Cargo metadata dependency-kind subset.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct CargoMetadataDependencyKind {
    /// Cargo dependency kind. `null` means normal.
    pub kind: Option<String>,
}

/// Purely classifies dependency-direction violations from an already parsed graph.
#[must_use]
pub fn dependency_direction(input: &DependencyDirectionInput) -> Vec<DependencyDirectionViolation> {
    let mut violations = Vec::new();
    append_core_violations(input, &mut violations);
    append_adapter_violations(input, &mut violations);
    append_shell_violations(input, &mut violations);
    violations
}

/// Converts already-parsed cargo metadata into classifier input using declared crate path sets.
#[must_use]
pub fn dependency_input_from_metadata(
    metadata: &CargoMetadata,
    core_crates: &[String],
    shell_crates: &[String],
    adapter_crates: &[String],
    included_edge_kinds: BTreeSet<DependencyEdgeKind>,
) -> DependencyDirectionInput {
    let packages_by_id = packages_by_id(metadata);
    let workspace_members = workspace_member_names(metadata, &packages_by_id);
    let graph = dependency_graph_from_metadata(metadata, &packages_by_id, &workspace_members);
    DependencyDirectionInput {
        graph,
        core_packages: packages_matching_declared_paths(metadata, core_crates),
        shell_packages: packages_matching_declared_paths(metadata, shell_crates),
        adapter_packages: packages_matching_declared_paths(metadata, adapter_crates),
        included_edge_kinds,
    }
}

fn append_core_violations(input: &DependencyDirectionInput, violations: &mut Vec<DependencyDirectionViolation>) {
    for core_package in &input.core_packages {
        let Some(node) = input.graph.nodes.get(core_package) else {
            continue;
        };
        for edge in included_edges(node, &input.included_edge_kinds) {
            if input.adapter_packages.contains(&edge.package) {
                violations.push(edge_violation(
                    DependencyDirectionViolationKind::CoreToAdapter,
                    core_package,
                    &edge.package,
                    None,
                    DependencyEdgeClassification::Direct,
                ));
                continue;
            }
            if !input.core_packages.contains(&edge.package) {
                violations.push(edge_violation(
                    DependencyDirectionViolationKind::CoreToExternalInfrastructure,
                    core_package,
                    &edge.package,
                    None,
                    DependencyEdgeClassification::Direct,
                ));
            }
            append_transitive_core_adapter_violations(input, core_package, &edge.package, violations);
        }
    }
}

fn append_transitive_core_adapter_violations(
    input: &DependencyDirectionInput,
    core_package: &str,
    intermediate_package: &str,
    violations: &mut Vec<DependencyDirectionViolation>,
) {
    if input.core_packages.contains(intermediate_package) || input.adapter_packages.contains(intermediate_package) {
        return;
    }
    let Some(intermediate_node) = input.graph.nodes.get(intermediate_package) else {
        return;
    };
    for transitive_edge in included_edges(intermediate_node, &input.included_edge_kinds) {
        if !input.adapter_packages.contains(&transitive_edge.package) {
            continue;
        }
        violations.push(edge_violation(
            DependencyDirectionViolationKind::CoreToAdapter,
            core_package,
            &transitive_edge.package,
            Some(intermediate_package),
            DependencyEdgeClassification::Transitive,
        ));
    }
}

fn append_adapter_violations(input: &DependencyDirectionInput, violations: &mut Vec<DependencyDirectionViolation>) {
    for adapter_package in &input.adapter_packages {
        let Some(node) = input.graph.nodes.get(adapter_package) else {
            continue;
        };
        if included_edges(node, &input.included_edge_kinds).any(|edge| input.core_packages.contains(&edge.package)) {
            continue;
        }
        violations.push(DependencyDirectionViolation {
            kind: DependencyDirectionViolationKind::AdapterMissingCore,
            dependent: adapter_package.clone(),
            dependency: None,
            intermediate: None,
            edge: None,
        });
    }
}

fn append_shell_violations(input: &DependencyDirectionInput, violations: &mut Vec<DependencyDirectionViolation>) {
    for shell_package in &input.shell_packages {
        let Some(node) = input.graph.nodes.get(shell_package) else {
            continue;
        };
        for edge in included_edges(node, &input.included_edge_kinds) {
            if !input.adapter_packages.contains(&edge.package) {
                continue;
            }
            violations.push(edge_violation(
                DependencyDirectionViolationKind::ShellOutwardToAdapter,
                shell_package,
                &edge.package,
                None,
                DependencyEdgeClassification::Direct,
            ));
        }
    }
}

fn included_edges<'a>(
    node: &'a DependencyNode,
    included_edge_kinds: &'a BTreeSet<DependencyEdgeKind>,
) -> impl Iterator<Item = &'a DependencyEdge> {
    node.dependencies.iter().filter(move |edge| included_edge_kinds.contains(&edge.kind))
}

fn edge_violation(
    kind: DependencyDirectionViolationKind,
    dependent: &str,
    dependency: &str,
    intermediate: Option<&str>,
    edge: DependencyEdgeClassification,
) -> DependencyDirectionViolation {
    DependencyDirectionViolation {
        kind,
        dependent: dependent.to_owned(),
        dependency: Some(dependency.to_owned()),
        intermediate: intermediate.map(str::to_owned),
        edge: Some(edge),
    }
}

fn packages_by_id(metadata: &CargoMetadata) -> BTreeMap<String, &CargoMetadataPackage> {
    metadata.packages.iter().map(|package| (package.id.clone(), package)).collect()
}

fn workspace_member_names(
    metadata: &CargoMetadata,
    packages_by_id: &BTreeMap<String, &CargoMetadataPackage>,
) -> BTreeSet<String> {
    metadata
        .workspace_members
        .iter()
        .filter_map(|id| packages_by_id.get(id).map(|package| package.name.clone()))
        .collect()
}

fn dependency_graph_from_metadata(
    metadata: &CargoMetadata,
    packages_by_id: &BTreeMap<String, &CargoMetadataPackage>,
    workspace_members: &BTreeSet<String>,
) -> DependencyGraph {
    let mut nodes = BTreeMap::new();
    let Some(resolve) = &metadata.resolve else {
        return DependencyGraph { nodes };
    };
    for node in &resolve.nodes {
        let Some(package) = packages_by_id.get(&node.id) else {
            continue;
        };
        if !workspace_members.contains(&package.name) {
            continue;
        }
        nodes.insert(package.name.clone(), DependencyNode {
            package: package.name.clone(),
            dependencies: dependency_edges_from_metadata_node(node, packages_by_id),
        });
    }
    DependencyGraph { nodes }
}

fn dependency_edges_from_metadata_node(
    node: &CargoMetadataNode,
    packages_by_id: &BTreeMap<String, &CargoMetadataPackage>,
) -> Vec<DependencyEdge> {
    let mut edges = BTreeSet::new();
    for dependency in &node.deps {
        let Some(package) = packages_by_id.get(&dependency.pkg) else {
            continue;
        };
        for dep_kind in &dependency.dep_kinds {
            let Some(kind) = DependencyEdgeKind::from_metadata_kind(dep_kind.kind.as_deref()) else {
                continue;
            };
            edges.insert((package.name.clone(), kind));
        }
    }
    edges.into_iter().map(|(package, kind)| DependencyEdge { package, kind }).collect()
}

fn packages_matching_declared_paths(metadata: &CargoMetadata, declared_paths: &[String]) -> BTreeSet<String> {
    metadata
        .packages
        .iter()
        .filter(|package| {
            declared_paths
                .iter()
                .any(|declared_path| declared_path_matches_manifest(declared_path, &package.manifest_path))
        })
        .map(|package| package.name.clone())
        .collect()
}

fn declared_path_matches_manifest(declared_path: &str, manifest_path: &str) -> bool {
    let normalized_declared = declared_path.replace('\\', "/");
    let normalized_manifest = manifest_path.replace('\\', "/");
    if normalized_manifest.ends_with(&normalized_declared) {
        return true;
    }
    let Some(crate_root) = normalized_declared.strip_suffix(RUST_LIB_SUFFIX) else {
        return false;
    };
    normalized_manifest.ends_with(&format!("{crate_root}{CARGO_MANIFEST_SUFFIX}"))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    const CORE: &str = "core";
    const SHELL: &str = "shell";
    const ADAPTER: &str = "adapter";
    const EXTERNAL: &str = "sqlx";
    const INTERMEDIATE: &str = "middle";
    const EXPECTED_NO_VIOLATIONS: usize = 0;
    const EXPECTED_ONE_VIOLATION: usize = 1;

    #[test]
    fn pure_core_accepts_clean_inward_graph() {
        let input = input_with_edges([
            (CORE, vec![]),
            (SHELL, vec![(CORE, DependencyEdgeKind::Normal)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        assert_eq!(dependency_direction(&input).len(), EXPECTED_NO_VIOLATIONS);
    }

    #[test]
    fn pure_core_rejects_core_to_adapter_direct_edge() {
        let input = input_with_edges([
            (CORE, vec![(ADAPTER, DependencyEdgeKind::Normal)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        let violations = dependency_direction(&input);

        assert_eq!(violations.len(), EXPECTED_ONE_VIOLATION);
        assert_eq!(violations[0].kind, DependencyDirectionViolationKind::CoreToAdapter);
        assert_eq!(violations[0].edge, Some(DependencyEdgeClassification::Direct));
    }

    #[test]
    fn pure_core_rejects_core_to_adapter_transitive_edge() {
        let input = input_with_edges([
            (CORE, vec![(INTERMEDIATE, DependencyEdgeKind::Normal)]),
            (INTERMEDIATE, vec![(ADAPTER, DependencyEdgeKind::Normal)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        let violations = dependency_direction(&input);

        assert!(violations.iter().any(|violation| {
            violation.kind == DependencyDirectionViolationKind::CoreToAdapter
                && violation.edge == Some(DependencyEdgeClassification::Transitive)
                && violation.intermediate.as_deref() == Some(INTERMEDIATE)
        }));
    }

    #[test]
    fn pure_core_rejects_core_to_external_infrastructure() {
        let input = input_with_edges([
            (CORE, vec![(EXTERNAL, DependencyEdgeKind::Normal)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        let violations = dependency_direction(&input);

        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == DependencyDirectionViolationKind::CoreToExternalInfrastructure)
        );
    }

    #[test]
    fn pure_core_rejects_adapter_without_inward_core_dependency() {
        let input = input_with_edges([(CORE, vec![]), (ADAPTER, vec![(EXTERNAL, DependencyEdgeKind::Normal)])]);

        let violations = dependency_direction(&input);

        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == DependencyDirectionViolationKind::AdapterMissingCore)
        );
    }

    #[test]
    fn pure_core_rejects_shell_outward_adapter_edge() {
        let input = input_with_edges([
            (CORE, vec![]),
            (SHELL, vec![(ADAPTER, DependencyEdgeKind::Normal)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        let violations = dependency_direction(&input);

        assert!(
            violations
                .iter()
                .any(|violation| violation.kind == DependencyDirectionViolationKind::ShellOutwardToAdapter)
        );
    }

    #[test]
    fn pure_core_excludes_dev_dependencies_by_default() {
        let input = input_with_edges([
            (CORE, vec![(ADAPTER, DependencyEdgeKind::Dev)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);

        assert_eq!(dependency_direction(&input).len(), EXPECTED_NO_VIOLATIONS);
    }

    #[test]
    fn pure_core_can_include_dev_dependencies() {
        let mut input = input_with_edges([
            (CORE, vec![(ADAPTER, DependencyEdgeKind::Dev)]),
            (ADAPTER, vec![(CORE, DependencyEdgeKind::Normal)]),
        ]);
        input.included_edge_kinds.insert(DependencyEdgeKind::Dev);

        let violations = dependency_direction(&input);

        assert_eq!(violations.len(), EXPECTED_ONE_VIOLATION);
        assert_eq!(violations[0].kind, DependencyDirectionViolationKind::CoreToAdapter);
    }

    fn input_with_edges<const N: usize>(
        edges: [(&str, Vec<(&str, DependencyEdgeKind)>); N],
    ) -> DependencyDirectionInput {
        let nodes = edges
            .into_iter()
            .map(|(package, dependencies)| {
                (package.to_owned(), DependencyNode {
                    package: package.to_owned(),
                    dependencies: dependencies
                        .into_iter()
                        .map(|(dependency, kind)| DependencyEdge {
                            package: dependency.to_owned(),
                            kind,
                        })
                        .collect(),
                })
            })
            .collect();
        DependencyDirectionInput {
            graph: DependencyGraph { nodes },
            core_packages: BTreeSet::from([CORE.to_owned()]),
            shell_packages: BTreeSet::from([SHELL.to_owned()]),
            adapter_packages: BTreeSet::from([ADAPTER.to_owned()]),
            included_edge_kinds: default_dependency_edge_kinds(),
        }
    }
}
