//! Architecture and FCIS boundary checks.

use std::collections::BTreeSet;

use crate::dependency_direction::CargoMetadata;
use crate::dependency_direction::DependencyDirectionViolation;
use crate::dependency_direction::DependencyDirectionViolationKind;
use crate::dependency_direction::DependencyEdgeClassification;
use crate::dependency_direction::DependencyEdgeKind;
use crate::dependency_direction::default_dependency_edge_kinds;
use crate::dependency_direction::dependency_direction;
use crate::dependency_direction::dependency_input_from_metadata;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::Mechanism;
use crate::model::MechanismKind;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::source::SourceFile;

const ARCHITECTURE_FAMILY: &str = "architecture";
const IDIOMS_FAMILY: &str = "idioms";
const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID: &str = "tigerstyle.architecture.external-domain-leakage";
const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
const EFFECT_CAPABILITY_POLICY_ID: &str = "tigerstyle.architecture.effect-capability-boundaries";
const HEXAGONAL_PORT_POLICY_ID: &str = "tigerstyle.architecture.hexagonal-ports";
const TYPE_STATE_POLICY_ID: &str = "tigerstyle.idioms.nominal-type-boundaries";
const FORBIDDEN_RUNTIME_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const STD_SHELL_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const RUST_SOURCE_EXTENSION: &str = ".rs";
const ROOT_LIB_PATH_SUFFIX: &str = "src/lib.rs";
const ROOT_MAIN_PATH_SUFFIX: &str = "src/main.rs";
const NO_STD_ATTRIBUTE: &str = "#![no_std]";
const CORE_DEPENDENCIES_SECTION: &str = "[dependencies]";
const DEV_DEPENDENCIES_SECTION: &str = "[dev-dependencies]";
const DEFAULT_DEPENDENCY_EDGE_KIND_NAMES: &[&str] = &["normal", "build"];

/// Synthetic source path used by the CLI shell for `cargo metadata --format-version 1` output.
pub const CARGO_METADATA_SOURCE_PATH: &str = "__octet/cargo-metadata.json";

// Octet-layout default fallback constants (policy-driven-crate-boundaries)
const FALLBACK_CORE_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-core/src/lib.rs";
const FALLBACK_CORE_MANIFEST_PATH_SUFFIX: &str = "crates/tigerstyle-core/Cargo.toml";
const FALLBACK_STD_MANIFEST_PATH_SUFFIX: &str = "crates/tigerstyle-std/Cargo.toml";
const FALLBACK_CORE_RUST_PATH_SEGMENT: &str = "crates/tigerstyle-core/src/";
const FALLBACK_STD_LIB_PATH_SUFFIX: &str = "crates/tigerstyle-std/src/lib.rs";
const FALLBACK_CORE_FORBIDDEN_DEPENDENCIES: &[&str] = &[
    "tokio",
    "tracing",
    "tracing-subscriber",
    "clap",
    "parking_lot",
    "url",
    "tigerstyle-std",
];
const FALLBACK_CORE_CRATE_DEPENDENCY: &str = "tigerstyle-core";
const FALLBACK_EXTERNAL_DOMAIN_TYPE_PATTERNS: &[&str] = &[
    "axum::",
    "hyper::",
    "prost::",
    "reqwest::",
    "serde_json::",
    "sqlx::",
    "tonic::",
];
const FALLBACK_AMBIENT_CAPABILITY_PATTERNS: &[&str] = &[
    "std::env::",
    "std::fs::",
    "std::net::",
    "std::process::",
    "SystemTime::now(",
    "Instant::now(",
    "File::open(",
];
const FALLBACK_CORE_SOURCE_SCOPES: &[&str] = &[FALLBACK_CORE_RUST_PATH_SEGMENT];
const FALLBACK_TYPE_STATE_SOURCE_SCOPES: &[&str] = &["typestate-raw/src/", "typestate-protocol/src/"];
const FALLBACK_RAW_STATE_ALIAS_PATTERNS: &[&str] = &["type ConnectionState = bool"];
const FALLBACK_RAW_TRANSITION_PATTERNS: &[&str] = &["send_raw(", "raw_send(", "open_raw("];
const FALLBACK_REQUIRED_STATE_MARKERS: &[&str] = &["Disconnected", "Connected"];
const FALLBACK_REQUIRED_TRANSITION_METHODS: &[&str] = &["connect(", "send("];
const PUBLIC_API_CHUNK_TERMINATORS: &[char] = &['{', ';'];
const PUBLIC_API_ITEM_KEYWORDS: &[&str] = &[
    "async fn ",
    "const ",
    "enum ",
    "fn ",
    "mod ",
    "static ",
    "struct ",
    "trait ",
    "type ",
    "use ",
];
const PUBLIC_SCOPED_VISIBILITY_END: char = ')';
const PUBLIC_SCOPED_VISIBILITY_PREFIX: &str = "pub(";
const PUBLIC_UNSCOPED_VISIBILITY_PREFIX: &str = "pub ";
const ROOT_LIB_MAX_LINES: usize = 40;
const ROOT_MAIN_MAX_LINES: usize = 80;
const ROOT_PUBLIC_API_ITEM_BUDGET: usize = 8;
const FIRST_PUBLIC_API_OVER_BUDGET_INDEX: usize = ROOT_PUBLIC_API_ITEM_BUDGET;
const FIRST_LINE: u32 = 1;
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const PUBLIC_VISIBILITY_PREFIXES: &[&str] = &["pub ", "pub("];
const EXTERNAL_LEAK_REMEDIATION: &str =
    "wrap the external type behind a domain type or trait adapter and translate at the boundary";

#[derive(Clone, Debug, Eq, PartialEq)]
struct PublicApiChunk {
    line: u32,
    text: String,
}

#[derive(Debug)]
struct CapabilityApiObligations {
    forbidden_ambient_apis: Vec<String>,
    core_source_scopes: Vec<String>,
    shell_constructor_scopes: Vec<String>,
    approved_capability_names: Vec<String>,
}

#[derive(Debug)]
struct TypeStateObligations {
    source_scopes: Vec<String>,
    raw_state_alias_patterns: Vec<String>,
    raw_transition_patterns: Vec<String>,
    required_state_markers: Vec<String>,
    required_transition_methods: Vec<String>,
}

#[derive(Debug)]
struct PortObligations {
    port_traits: Vec<String>,
    core_owner_scope: String,
    permitted_adapter_scopes: Vec<String>,
    forbidden_concrete_types: Vec<String>,
    reserved: bool,
}

#[derive(Debug)]
struct CrateBoundaryObligations {
    core_lib_path_suffix: String,
    core_manifest_path_suffix: String,
    std_manifest_path_suffix: String,
    std_lib_path_suffix: String,
    core_rust_path_segment: String,
    core_crates: Vec<String>,
    shell_crates: Vec<String>,
    adapter_crates: Vec<String>,
    core_forbidden_dependencies: Vec<String>,
    core_crate_dependency: String,
    external_domain_patterns: Vec<String>,
    dependency_edge_kinds: Vec<String>,
    #[allow(dead_code)]
    core_no_std: bool,
}

impl CrateBoundaryObligations {
    fn from_policy(policy: &PolicyEntry) -> Self {
        let crate_boundary_mechanism =
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::CrateBoundary);
        let fallback_core = || FALLBACK_CORE_LIB_PATH_SUFFIX.to_owned();
        let fallback_core_manifest = || FALLBACK_CORE_MANIFEST_PATH_SUFFIX.to_owned();
        let fallback_std_manifest = || FALLBACK_STD_MANIFEST_PATH_SUFFIX.to_owned();
        let fallback_std_lib = || FALLBACK_STD_LIB_PATH_SUFFIX.to_owned();
        let fallback_core_rust = || FALLBACK_CORE_RUST_PATH_SEGMENT.to_owned();

        // Derive manifest path from core lib path: replace /src/lib.rs with /Cargo.toml
        let core_crates = mechanism_field_or_fallback(crate_boundary_mechanism, |mechanism| &mechanism.core_crates, &[
            FALLBACK_CORE_LIB_PATH_SUFFIX,
        ]);
        let shell_crates =
            mechanism_field_or_fallback(crate_boundary_mechanism, |mechanism| &mechanism.shell_crates, &[
                FALLBACK_STD_MANIFEST_PATH_SUFFIX,
            ]);
        let adapter_crates =
            crate_boundary_mechanism.map_or_else(Vec::new, |mechanism| mechanism.adapter_crates.clone());

        let core_suffix = core_crates.first().cloned();
        let core_manifest = core_suffix.as_ref().map(|path| {
            if let Some(crate_root) = path.strip_suffix("/src/lib.rs") {
                format!("{crate_root}/Cargo.toml")
            } else {
                path.clone()
            }
        });

        // Derive lib path from shell manifest: replace /Cargo.toml with /src/lib.rs
        let shell_suffix = shell_crates.first().cloned();
        let shell_lib = shell_suffix.as_ref().map(|path| {
            if let Some(crate_root) = path.strip_suffix("/Cargo.toml") {
                format!("{crate_root}/src/lib.rs")
            } else {
                path.clone()
            }
        });

        // Derive core crate dep name from manifest path
        let core_dep_name = core_manifest.as_ref().and_then(|manifest| {
            manifest
                .strip_suffix("/Cargo.toml")
                .map(|crate_root| crate_root.rsplit('/').next().unwrap_or(crate_root).to_owned())
        });

        Self {
            core_lib_path_suffix: core_suffix.clone().unwrap_or_else(fallback_core),
            core_manifest_path_suffix: core_manifest.unwrap_or_else(fallback_core_manifest),
            std_manifest_path_suffix: shell_suffix.clone().unwrap_or_else(fallback_std_manifest),
            std_lib_path_suffix: shell_lib.unwrap_or_else(fallback_std_lib),
            core_rust_path_segment: core_suffix.map_or_else(fallback_core_rust, |path| {
                if let Some(crate_root) = path.strip_suffix("/src/lib.rs") {
                    format!("{crate_root}/src/")
                } else {
                    path
                }
            }),
            core_crates,
            shell_crates,
            adapter_crates,
            core_forbidden_dependencies: mechanism_field_or_fallback(
                crate_boundary_mechanism,
                |mechanism| &mechanism.forbidden_core_deps,
                FALLBACK_CORE_FORBIDDEN_DEPENDENCIES,
            ),
            core_crate_dependency: core_dep_name.unwrap_or_else(|| FALLBACK_CORE_CRATE_DEPENDENCY.to_owned()),
            external_domain_patterns: mechanism_field_or_fallback(
                crate_boundary_mechanism,
                |mechanism| &mechanism.external_domain_patterns,
                FALLBACK_EXTERNAL_DOMAIN_TYPE_PATTERNS,
            ),
            dependency_edge_kinds: mechanism_field_or_fallback(
                crate_boundary_mechanism,
                |mechanism| &mechanism.dependency_edge_kinds,
                DEFAULT_DEPENDENCY_EDGE_KIND_NAMES,
            ),
            core_no_std: crate_boundary_mechanism.is_none_or(|m| m.core_no_std),
        }
    }
}

impl PortObligations {
    fn from_policy(policy: &PolicyEntry) -> Option<Self> {
        let mechanism = policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::Port)?;
        Some(Self {
            port_traits: mechanism.port_traits.clone(),
            core_owner_scope: mechanism.core_owner_scope.clone(),
            permitted_adapter_scopes: mechanism.permitted_adapter_scopes.clone(),
            forbidden_concrete_types: mechanism.forbidden_concrete_types.clone(),
            reserved: mechanism.reserved,
        })
    }

    fn enabled(&self) -> bool {
        !self.port_traits.is_empty() && !self.core_owner_scope.is_empty()
    }
}

impl TypeStateObligations {
    fn from_policy(policy: &PolicyEntry) -> Self {
        let type_state_mechanism =
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::TypeState);
        Self {
            source_scopes: mechanism_field_or_fallback(
                type_state_mechanism,
                |mechanism| &mechanism.type_state_source_scopes,
                FALLBACK_TYPE_STATE_SOURCE_SCOPES,
            ),
            raw_state_alias_patterns: mechanism_field_or_fallback(
                type_state_mechanism,
                |mechanism| &mechanism.raw_state_alias_patterns,
                FALLBACK_RAW_STATE_ALIAS_PATTERNS,
            ),
            raw_transition_patterns: mechanism_field_or_fallback(
                type_state_mechanism,
                |mechanism| &mechanism.raw_transition_patterns,
                FALLBACK_RAW_TRANSITION_PATTERNS,
            ),
            required_state_markers: mechanism_field_or_fallback(
                type_state_mechanism,
                |mechanism| &mechanism.required_state_markers,
                FALLBACK_REQUIRED_STATE_MARKERS,
            ),
            required_transition_methods: mechanism_field_or_fallback(
                type_state_mechanism,
                |mechanism| &mechanism.required_transition_methods,
                FALLBACK_REQUIRED_TRANSITION_METHODS,
            ),
        }
    }
}

impl CapabilityApiObligations {
    fn from_policy(policy: &PolicyEntry) -> Self {
        let capability_mechanism =
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::CapabilityApi);
        Self {
            forbidden_ambient_apis: mechanism_field_or_fallback(
                capability_mechanism,
                |mechanism| &mechanism.forbidden_ambient_apis,
                FALLBACK_AMBIENT_CAPABILITY_PATTERNS,
            ),
            core_source_scopes: mechanism_field_or_fallback(
                capability_mechanism,
                |mechanism| &mechanism.core_source_scopes,
                FALLBACK_CORE_SOURCE_SCOPES,
            ),
            shell_constructor_scopes: mechanism_field_or_fallback(
                capability_mechanism,
                |mechanism| &mechanism.shell_constructor_scopes,
                &[],
            ),
            approved_capability_names: mechanism_field_or_fallback(
                capability_mechanism,
                |mechanism| &mechanism.approved_capability_names,
                &[],
            ),
        }
    }

    fn remediation(&self) -> String {
        if self.approved_capability_names.is_empty() {
            return "move ambient authority constructors to the shell and inject data or narrow capability traits into core"
                .to_owned();
        }
        format!(
            "move ambient authority constructors to the shell and inject data or named capabilities such as {} into core",
            self.approved_capability_names.join(", ")
        )
    }
}

fn mechanism_field_or_fallback(
    mechanism: Option<&Mechanism>,
    field: impl FnOnce(&Mechanism) -> &Vec<String>,
    fallback: &[&str],
) -> Vec<String> {
    mechanism.map_or_else(
        || fallback.iter().map(|value| (*value).to_owned()).collect(),
        |mechanism| {
            let values = field(mechanism);
            if values.is_empty() {
                fallback.iter().map(|value| (*value).to_owned()).collect()
            } else {
                values.clone()
            }
        },
    )
}

/// Runs architecture policy checks over source files.
#[must_use]
pub fn check_architecture(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    let includes_architecture = selection.includes(ARCHITECTURE_FAMILY);
    let includes_idioms = selection.includes(IDIOMS_FAMILY);
    if !includes_architecture && !includes_idioms {
        return Vec::new();
    }

    let boundary_obligations = registry.policy_by_id(FCIS_POLICY_ID).map(CrateBoundaryObligations::from_policy);

    let mut diagnostics = Vec::new();
    if includes_architecture {
        check_core_no_std(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_core_forbidden_dependencies(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_std_shell_owns_std(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_shell_depends_on_core(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_dependency_direction(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_core_capability_constructors(registry, sources, &mut diagnostics);
        check_hexagonal_ports(registry, sources, &mut diagnostics);
        check_external_domain_leakage(registry, sources, boundary_obligations.as_ref(), &mut diagnostics);
        check_root_thinness(registry, sources, &mut diagnostics);
        check_public_api_budget(registry, sources, &mut diagnostics);
    }
    if includes_idioms {
        check_raw_type_state_operations(registry, sources, &mut diagnostics);
    }
    diagnostics
}

fn check_core_no_std(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    let lib_suffix = boundary.map_or(FALLBACK_CORE_LIB_PATH_SUFFIX, |b| b.core_lib_path_suffix.as_str());
    for source in sources_by_suffix(sources, lib_suffix) {
        if source.text.contains(NO_STD_ATTRIBUTE) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(FIRST_LINE),
            &format!("core crate `{lib_suffix}` is missing #![no_std]"),
            "add #![no_std] to the core crate root or move std-dependent code to the shell",
        ));
    }
}

fn check_core_forbidden_dependencies(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FORBIDDEN_RUNTIME_POLICY_ID) else {
        return;
    };
    let manifest_suffix = boundary.map_or(FALLBACK_CORE_MANIFEST_PATH_SUFFIX, |b| b.core_manifest_path_suffix.as_str());
    let forbidden_deps: &[String] = boundary.map_or(&[], |b| &b.core_forbidden_dependencies);
    for source in sources_by_suffix(sources, manifest_suffix) {
        let dependency_section = dependency_section(&source.text);
        if forbidden_deps.is_empty() {
            for dep in FALLBACK_CORE_FORBIDDEN_DEPENDENCIES {
                if dependency_section_contains(dependency_section, dep) {
                    diagnostics.push(policy_diagnostic(
                        policy,
                        source,
                        line_for_text(&source.text, dep),
                        &format!("core crate depends on runtime crate `{dep}`"),
                        "move runtime, async, logging, CLI, or I/O dependencies to the shell crate",
                    ));
                }
            }
        } else {
            for dep in forbidden_deps {
                if dependency_section_contains(dependency_section, dep) {
                    diagnostics.push(policy_diagnostic(
                        policy,
                        source,
                        line_for_text(&source.text, dep),
                        &format!("core crate depends on forbidden dependency `{dep}`"),
                        "move runtime, async, logging, CLI, or I/O dependencies to the shell crate",
                    ));
                }
            }
        }
    }
}

fn check_std_shell_owns_std(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(STD_SHELL_POLICY_ID) else {
        return;
    };
    let lib_suffix = boundary.map_or(FALLBACK_STD_LIB_PATH_SUFFIX, |b| b.std_lib_path_suffix.as_str());
    for source in sources_by_suffix(sources, lib_suffix) {
        if !source.text.contains(NO_STD_ATTRIBUTE) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(FIRST_LINE),
            &format!("shell crate `{lib_suffix}` must not declare #![no_std]"),
            "keep std/runtime shell code in the shell crate and no_std code in core",
        ));
    }
}

fn check_shell_depends_on_core(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    let manifest_suffix = boundary.map_or(FALLBACK_STD_MANIFEST_PATH_SUFFIX, |b| b.std_manifest_path_suffix.as_str());
    let core_dep = boundary.map_or(FALLBACK_CORE_CRATE_DEPENDENCY, |b| b.core_crate_dependency.as_str());
    for source in sources_by_suffix(sources, manifest_suffix) {
        let dependency_section = dependency_section(&source.text);
        if dependency_section_contains(dependency_section, core_dep) {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            None,
            &format!("shell crate must depend on core crate `{core_dep}`"),
            "make shell crates depend inward on the core crate and keep the reverse dependency forbidden",
        ));
    }
}

fn check_dependency_direction(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(FCIS_POLICY_ID) else {
        return;
    };
    let Some(boundary) = boundary else {
        return;
    };
    let Some(metadata_source) = sources.iter().find(|source| source.path.ends_with(CARGO_METADATA_SOURCE_PATH)) else {
        return;
    };
    let metadata = match serde_json::from_str::<CargoMetadata>(&metadata_source.text) {
        Ok(metadata) => metadata,
        Err(error) => {
            diagnostics.push(policy_diagnostic(
                policy,
                metadata_source,
                None,
                &format!("cargo metadata fixture could not be parsed: {error}"),
                "regenerate cargo metadata with `cargo metadata --format-version 1`",
            ));
            return;
        }
    };
    let input = dependency_input_from_metadata(
        &metadata,
        &boundary.core_crates,
        &boundary.shell_crates,
        &boundary.adapter_crates,
        dependency_edge_kind_set(&boundary.dependency_edge_kinds),
    );
    diagnostics.extend(
        dependency_direction(&input)
            .into_iter()
            .map(|violation| dependency_direction_diagnostic(policy, metadata_source, &violation)),
    );
}

fn dependency_edge_kind_set(configured: &[String]) -> BTreeSet<DependencyEdgeKind> {
    let kinds = configured
        .iter()
        .filter_map(|kind| DependencyEdgeKind::from_config_value(kind))
        .collect::<BTreeSet<_>>();
    if kinds.is_empty() {
        default_dependency_edge_kinds()
    } else {
        kinds
    }
}

fn dependency_direction_diagnostic(
    policy: &PolicyEntry,
    source: &SourceFile,
    violation: &DependencyDirectionViolation,
) -> Diagnostic {
    let (message, remediation) = dependency_direction_message(violation);
    policy_diagnostic(policy, source, None, &message, &remediation)
}

fn dependency_direction_message(violation: &DependencyDirectionViolation) -> (String, String) {
    match violation.kind {
        DependencyDirectionViolationKind::CoreToAdapter => core_to_adapter_message(violation),
        DependencyDirectionViolationKind::CoreToExternalInfrastructure => {
            let dependency = violation.dependency.as_deref().unwrap_or("unknown");
            (
                format!(
                    "core crate `{}` depends on external infrastructure crate `{dependency}` via {} dependency edge",
                    violation.dependent,
                    edge_label(violation.edge),
                ),
                "move infrastructure dependencies out of core or inject a port owned by core".to_owned(),
            )
        }
        DependencyDirectionViolationKind::AdapterMissingCore => (
            format!("adapter crate `{}` must depend inward on a declared core crate", violation.dependent),
            "make adapter crates depend on core traits/types and keep core independent of adapters".to_owned(),
        ),
        DependencyDirectionViolationKind::ShellOutwardToAdapter => {
            let dependency = violation.dependency.as_deref().unwrap_or("unknown");
            (
                format!(
                    "shell crate `{}` depends outward on adapter crate `{dependency}` via {} dependency edge",
                    violation.dependent,
                    edge_label(violation.edge),
                ),
                "keep the shell thin and route adapter details behind core-owned ports".to_owned(),
            )
        }
    }
}

fn core_to_adapter_message(violation: &DependencyDirectionViolation) -> (String, String) {
    let dependency = violation.dependency.as_deref().unwrap_or("unknown");
    if violation.edge == Some(DependencyEdgeClassification::Transitive) {
        let intermediate = violation.intermediate.as_deref().unwrap_or("unknown");
        return (
            format!(
                "core crate `{}` reaches adapter crate `{dependency}` through `{intermediate}` via transitive dependency leakage",
                violation.dependent
            ),
            "move the dependency to an adapter or inject a core-owned port instead of leaking through an intermediate crate"
                .to_owned(),
        );
    }
    (
        format!(
            "core crate `{}` depends on adapter crate `{dependency}` via direct dependency edge",
            violation.dependent
        ),
        "move the dependency to an adapter or inject a core-owned port".to_owned(),
    )
}

fn edge_label(edge: Option<DependencyEdgeClassification>) -> &'static str {
    match edge {
        Some(DependencyEdgeClassification::Transitive) => "transitive",
        Some(DependencyEdgeClassification::Direct) | None => "direct",
    }
}

fn check_core_capability_constructors(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(EFFECT_CAPABILITY_POLICY_ID) else {
        return;
    };
    let obligations = CapabilityApiObligations::from_policy(policy);
    let remediation = obligations.remediation();
    for source in capability_core_sources(sources, &obligations) {
        for (line_index, line) in source.text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }
            let Some(pattern) =
                obligations.forbidden_ambient_apis.iter().find(|pattern| line.contains(pattern.as_str()))
            else {
                continue;
            };
            diagnostics.push(policy_diagnostic(
                policy,
                source,
                Some(line_number_from_index(line_index)),
                &format!(
                    "core scope performs undeclared direct {} capability operation `{pattern}` instead of accepting an injected capability",
                    capability_family_for_pattern(pattern),
                ),
                &remediation,
            ));
        }
    }
}

fn check_hexagonal_ports(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(HEXAGONAL_PORT_POLICY_ID) else {
        return;
    };
    let Some(obligations) = PortObligations::from_policy(policy) else {
        return;
    };
    if !obligations.enabled() || sources.is_empty() {
        return;
    }

    for port_trait in &obligations.port_traits {
        append_port_defined_in_core_diagnostic(policy, sources, &obligations, port_trait, diagnostics);
        append_port_adapter_impl_diagnostic(policy, sources, &obligations, port_trait, diagnostics);
        append_core_concrete_port_impl_diagnostics(policy, sources, &obligations, port_trait, diagnostics);
    }
}

fn append_port_defined_in_core_diagnostic(
    policy: &PolicyEntry,
    sources: &[SourceFile],
    obligations: &PortObligations,
    port_trait: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let trait_name = short_type_name(port_trait);
    if scoped_sources(sources, std::slice::from_ref(&obligations.core_owner_scope))
        .any(|source| source.text.contains(&format!("trait {trait_name}")))
    {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        fallback_source(sources),
        None,
        &format!(
            "declared port `{port_trait}` must be defined as a trait in core scope `{}`",
            obligations.core_owner_scope
        ),
        "declare the port trait in core or update the port mechanism core_owner_scope",
    ));
}

fn append_port_adapter_impl_diagnostic(
    policy: &PolicyEntry,
    sources: &[SourceFile],
    obligations: &PortObligations,
    port_trait: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if obligations.reserved {
        return;
    }
    let trait_name = short_type_name(port_trait);
    if scoped_sources(sources, &obligations.permitted_adapter_scopes).any(|source| {
        source.text.contains(&format!("impl {trait_name} for"))
            || source.text.contains(&format!("impl {port_trait} for"))
    }) {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        fallback_source(sources),
        None,
        &format!(
            "declared port `{port_trait}` has no implementation in permitted adapter scopes [{}]",
            obligations.permitted_adapter_scopes.join(", ")
        ),
        "implement the port in an adapter scope or mark it reserved while the adapter is intentionally absent",
    ));
}

fn append_core_concrete_port_impl_diagnostics(
    policy: &PolicyEntry,
    sources: &[SourceFile],
    obligations: &PortObligations,
    port_trait: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let trait_name = short_type_name(port_trait);
    for source in scoped_sources(sources, std::slice::from_ref(&obligations.core_owner_scope)) {
        for (line_index, line) in source.text.lines().enumerate() {
            let Some(concrete_type) = concrete_port_impl_type(line, trait_name, &obligations.forbidden_concrete_types)
            else {
                continue;
            };
            diagnostics.push(policy_diagnostic(
                policy,
                source,
                Some(line_number_from_index(line_index)),
                &format!("core implements port `{port_trait}` for concrete infrastructure type `{concrete_type}`"),
                "keep port traits abstract in core and move concrete implementations to permitted adapter scopes",
            ));
        }
    }
}

fn concrete_port_impl_type(line: &str, trait_name: &str, forbidden_patterns: &[String]) -> Option<String> {
    let trimmed = line.trim();
    let after_impl = trimmed.strip_prefix("impl ")?;
    let (implemented_trait, concrete) = after_impl.split_once(" for ")?;
    if implemented_trait.trim().rsplit("::").next()? != trait_name {
        return None;
    }
    let concrete = concrete.trim().trim_end_matches('{').trim().to_owned();
    if forbidden_patterns.is_empty() || forbidden_patterns.iter().any(|pattern| concrete.contains(pattern)) {
        Some(concrete)
    } else {
        None
    }
}

fn short_type_name(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

fn scoped_sources<'a>(sources: &'a [SourceFile], scopes: &'a [String]) -> impl Iterator<Item = &'a SourceFile> {
    sources.iter().filter(move |source| scopes.iter().any(|scope| source.path.contains(scope)))
}

fn fallback_source(sources: &[SourceFile]) -> &SourceFile {
    sources.first().expect("architecture checks require at least one source file")
}

fn check_raw_type_state_operations(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(TYPE_STATE_POLICY_ID) else {
        return;
    };
    let obligations = TypeStateObligations::from_policy(policy);
    for source in type_state_sources(sources, &obligations) {
        append_raw_type_state_bypass_diagnostics(policy, source, &obligations, diagnostics);
        append_missing_type_state_evidence_diagnostics(policy, source, &obligations, diagnostics);
    }
}

fn append_raw_type_state_bypass_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    obligations: &TypeStateObligations,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (line_index, line) in source.text.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }
        let Some(pattern) = obligations
            .raw_state_alias_patterns
            .iter()
            .chain(obligations.raw_transition_patterns.iter())
            .find(|pattern| line.contains(pattern.as_str()))
        else {
            continue;
        };
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(line_number_from_index(line_index)),
            &format!("raw protocol bypass `{pattern}` violates artifact-backed type-state obligations"),
            "model protocol states with nominal marker wrappers and consuming transition methods instead of raw shortcuts",
        ));
    }
}

fn append_missing_type_state_evidence_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    obligations: &TypeStateObligations,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let missing_markers = obligations
        .required_state_markers
        .iter()
        .filter(|marker| !source.text.contains(marker.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let missing_methods = obligations
        .required_transition_methods
        .iter()
        .filter(|method| !source.text.contains(method.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if missing_markers.is_empty() && missing_methods.is_empty() {
        return;
    }
    diagnostics.push(policy_diagnostic(
        policy,
        source,
        Some(FIRST_LINE),
        &format!(
            "type-state source is missing required marker or transition evidence: markers=[{}], methods=[{}]",
            missing_markers.join(", "),
            missing_methods.join(", ")
        ),
        "declare nominal state marker types and transition methods for the configured protocol surface",
    ));
}

fn check_external_domain_leakage(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    boundary: Option<&CrateBoundaryObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(EXTERNAL_DOMAIN_LEAKAGE_POLICY_ID) else {
        return;
    };
    let core_segment = boundary.map_or(FALLBACK_CORE_RUST_PATH_SEGMENT, |b| b.core_rust_path_segment.as_str());
    let patterns: Vec<String> = boundary.map_or_else(
        || FALLBACK_EXTERNAL_DOMAIN_TYPE_PATTERNS.iter().map(|s| (*s).to_owned()).collect(),
        |b| b.external_domain_patterns.clone(),
    );
    for source in sources.iter().filter(|s| s.path.contains(core_segment) && s.path.ends_with(RUST_SOURCE_EXTENSION)) {
        append_external_domain_leakage_diagnostics(policy, source, &patterns, diagnostics);
    }
}

fn append_external_domain_leakage_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    patterns: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for chunk in public_api_chunks(&source.text) {
        let Some(pattern) = external_domain_pattern(&chunk.text, patterns) else {
            continue;
        };
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(chunk.line),
            &format!("core public API leaks external domain type pattern `{pattern}`"),
            EXTERNAL_LEAK_REMEDIATION,
        ));
    }
}

fn check_root_thinness(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(ROOT_THINNESS_POLICY_ID) else {
        return;
    };
    check_root_file_thinness(policy, sources, ROOT_LIB_PATH_SUFFIX, ROOT_LIB_MAX_LINES, diagnostics);
    check_root_file_thinness(policy, sources, ROOT_MAIN_PATH_SUFFIX, ROOT_MAIN_MAX_LINES, diagnostics);
}

fn check_root_file_thinness(
    policy: &PolicyEntry,
    sources: &[SourceFile],
    path_suffix: &str,
    max_lines: usize,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for source in root_sources_by_suffix(sources, path_suffix) {
        let line_count = source.text.lines().count();
        if line_count <= max_lines {
            continue;
        }
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            None,
            &format!("root file has {line_count} lines, exceeding thin-shell limit {max_lines}"),
            "move non-trivial logic from the root crate into tigerstyle-core or tigerstyle-std",
        ));
    }
}

fn check_public_api_budget(registry: &PolicyRegistry, sources: &[SourceFile], diagnostics: &mut Vec<Diagnostic>) {
    let Some(policy) = registry.policy_by_id(PUBLIC_API_BUDGET_POLICY_ID) else {
        return;
    };
    for source in root_sources_by_suffix(sources, ROOT_LIB_PATH_SUFFIX) {
        let public_item_lines = public_api_item_lines(&source.text);
        if public_item_lines.len() <= ROOT_PUBLIC_API_ITEM_BUDGET {
            continue;
        }
        assert!(public_item_lines.len() > FIRST_PUBLIC_API_OVER_BUDGET_INDEX);
        diagnostics.push(policy_diagnostic(
            policy,
            source,
            Some(public_item_lines[FIRST_PUBLIC_API_OVER_BUDGET_INDEX]),
            &format!(
                "root public API has {} visible items, exceeding advisory budget {}",
                public_item_lines.len(),
                ROOT_PUBLIC_API_ITEM_BUDGET
            ),
            "move public surface into domain crates or document the expansion in API review",
        ));
    }
}

fn public_api_item_lines(text: &str) -> Vec<u32> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| is_public_api_item_line(line).then_some(line_number_from_index(line_index)))
        .collect()
}

fn public_api_chunks(text: &str) -> Vec<PublicApiChunk> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut chunks = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        let line = lines[line_index];
        if !is_public_api_chunk_start(line) {
            line_index = line_index.saturating_add(FIRST_LINE_NUMBER_OFFSET);
            continue;
        }

        let start_line = line_number_from_index(line_index);
        let (chunk_text, next_line_index) = collect_public_api_chunk(&lines, line_index);
        chunks.push(PublicApiChunk {
            line: start_line,
            text: chunk_text,
        });
        line_index = next_line_index;
    }

    chunks
}

fn collect_public_api_chunk(lines: &[&str], start_index: usize) -> (String, usize) {
    let mut chunk = lines[start_index].trim().to_owned();
    let mut cursor = start_index;
    while !public_api_chunk_is_complete(&chunk) && cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET) < lines.len() {
        cursor = cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET);
        chunk.push(' ');
        chunk.push_str(lines[cursor].trim());
    }
    (chunk, cursor.saturating_add(FIRST_LINE_NUMBER_OFFSET))
}

fn public_api_chunk_is_complete(chunk: &str) -> bool {
    chunk.chars().any(|character| PUBLIC_API_CHUNK_TERMINATORS.contains(&character))
}

fn is_public_api_item_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    PUBLIC_VISIBILITY_PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix))
}

fn is_public_api_chunk_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(after_visibility) = text_after_public_visibility(trimmed) else {
        return false;
    };
    PUBLIC_API_ITEM_KEYWORDS.iter().any(|keyword| after_visibility.starts_with(keyword))
}

fn text_after_public_visibility(trimmed: &str) -> Option<&str> {
    if let Some(after_pub) = trimmed.strip_prefix(PUBLIC_UNSCOPED_VISIBILITY_PREFIX) {
        return Some(after_pub);
    }
    if !trimmed.starts_with(PUBLIC_SCOPED_VISIBILITY_PREFIX) {
        return None;
    }
    let (_, after_scope) = trimmed.split_once(PUBLIC_SCOPED_VISIBILITY_END)?;
    Some(after_scope.trim_start())
}

fn external_domain_pattern<'a>(text: &str, patterns: &'a [String]) -> Option<&'a str> {
    patterns.iter().find(|pattern| text.contains(pattern.as_str())).map(String::as_str)
}

fn sources_by_suffix<'a>(sources: &'a [SourceFile], suffix: &'a str) -> impl Iterator<Item = &'a SourceFile> {
    sources.iter().filter(move |source| source.path.ends_with(suffix))
}

fn root_sources_by_suffix<'a>(sources: &'a [SourceFile], suffix: &'a str) -> impl Iterator<Item = &'a SourceFile> {
    sources_by_suffix(sources, suffix).filter(|source| !source.path.contains("/crates/"))
}

fn capability_core_sources<'a>(
    sources: &'a [SourceFile],
    obligations: &'a CapabilityApiObligations,
) -> impl Iterator<Item = &'a SourceFile> {
    sources.iter().filter(|source| {
        source.path.ends_with(RUST_SOURCE_EXTENSION)
            && obligations.core_source_scopes.iter().any(|scope| source.path.contains(scope))
            && !obligations.shell_constructor_scopes.iter().any(|scope| source.path.contains(scope))
    })
}

fn capability_family_for_pattern(pattern: &str) -> &'static str {
    if pattern.contains("SystemTime::now(") || pattern.contains("Instant::now(") || pattern.contains("std::time::") {
        return "time";
    }
    if pattern.contains("rand::") || pattern.contains("getrandom::") {
        return "randomness";
    }
    if pattern.contains("std::fs::") || pattern.contains("File::open(") {
        return "filesystem";
    }
    if pattern.contains("std::net::") || pattern.contains("reqwest::") {
        return "network";
    }
    if pattern.contains("std::process::") {
        return "process";
    }
    if pattern.contains("std::env::") {
        return "environment";
    }
    if pattern.contains("Secret") {
        return "secrets";
    }
    if pattern.contains("remote_exec") {
        return "remote-execution";
    }
    if pattern.contains("unsafe {") {
        return "unsafe-code";
    }
    if pattern.contains("static mut") {
        return "global-state";
    }
    if pattern.contains("thread::spawn") || pattern.contains("tokio::spawn") {
        return "concurrency";
    }
    if pattern.contains("tracing::") || pattern.contains("log::") {
        return "observability";
    }
    "unknown"
}

fn type_state_sources<'a>(
    sources: &'a [SourceFile],
    obligations: &'a TypeStateObligations,
) -> impl Iterator<Item = &'a SourceFile> {
    sources.iter().filter(|source| {
        source.path.ends_with(RUST_SOURCE_EXTENSION)
            && obligations.source_scopes.iter().any(|scope| source.path.contains(scope))
    })
}

fn dependency_section(manifest: &str) -> &str {
    let Some((_, after_dependencies)) = manifest.split_once(CORE_DEPENDENCIES_SECTION) else {
        return "";
    };
    match after_dependencies.split_once(DEV_DEPENDENCIES_SECTION) {
        Some((dependencies, _)) => dependencies,
        None => after_dependencies,
    }
}

fn dependency_section_contains(dependency_section: &str, dependency_name: &str) -> bool {
    dependency_section
        .lines()
        .map(str::trim_start)
        .any(|line| line_has_dependency_name(line, dependency_name))
}

fn line_has_dependency_name(line: &str, dependency_name: &str) -> bool {
    if !line.starts_with(dependency_name) {
        return false;
    }
    matches!(line[dependency_name.len()..].chars().next(), Some(' ' | '='))
}

fn line_for_text(text: &str, needle: &str) -> Option<u32> {
    for (line_index, line) in text.lines().enumerate() {
        if line.contains(needle) {
            return Some(line_number_from_index(line_index));
        }
    }
    None
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}

fn policy_diagnostic(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: Option<u32>,
    message: &str,
    remediation: &str,
) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line,
        severity: policy.default_severity,
        message: message.to_owned(),
        remediation: remediation.to_owned(),
    })
}
