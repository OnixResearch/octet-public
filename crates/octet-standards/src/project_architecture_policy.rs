//! Pure project architecture policy model and validation.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

/// Project architecture policy schema accepted by this Octet release.
pub const PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION: &str = "octet-project-architecture-policy/v1";

const BLAKE3_HEX_LENGTH: usize = 64;
const PATH_SEPARATOR: char = '/';

/// Project-selected architecture enforcement mode.
// r[impl octet.project_architecture_policy.modes]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureMode {
    /// Record declared facts without promotion blocking.
    Inventory,
    /// Report architecture findings without promotion blocking.
    Advisory,
    /// Block declared architecture violations and required unknown facts.
    Gate,
    /// Add transitive effect-closure admission after the architecture gate.
    Admission,
}

/// Explicit architecture role for a package or source scope.
// r[impl octet.project_architecture_policy.roles]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureRole {
    /// Deterministic domain meaning and policy.
    DomainCore,
    /// Application orchestration and application-owned ports.
    ApplicationShell,
    /// Adapter that invokes the application.
    InboundAdapter,
    /// Adapter that implements an application-owned outbound port.
    OutboundAdapter,
    /// Wiring scope that selects concrete adapters.
    CompositionRoot,
    /// Test-only scope excluded from production claims.
    Test,
    /// Generated scope excluded from authored-source claims.
    Generated,
}

impl ArchitectureRole {
    /// Return true for roles that are part of the production architecture graph.
    #[must_use]
    pub fn is_production(self) -> bool {
        !matches!(self, Self::Test | Self::Generated)
    }
}

/// Kind of project scope bound to an architecture role.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureScopeKind {
    /// Cargo package name.
    Package,
    /// Workspace-relative source scope.
    Source,
}

/// Target and feature selection covered by architecture evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureTarget {
    /// Rust target triple.
    pub triple: String,
    /// Exact enabled Cargo features for this target selection.
    #[serde(default)]
    pub features: Vec<String>,
    /// True when Cargo default features are enabled.
    pub default_features: bool,
}

/// Declared production inventory for architecture coverage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureProductionScope {
    /// Cargo packages included in production claims.
    pub packages: Vec<String>,
    /// Workspace-relative source scopes included in production claims.
    pub source_scopes: Vec<String>,
    /// Target and feature selections included in production claims.
    pub targets: Vec<ArchitectureTarget>,
}

/// Explicit role assignment for one package or source scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureRoleAssignment {
    /// Scope kind.
    pub scope_kind: ArchitectureScopeKind,
    /// Package name or workspace-relative source scope.
    pub scope: String,
    /// Explicit role.
    pub role: ArchitectureRole,
}

/// Permitted dependency edges for one caller role.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureDependencyRule {
    /// Role that owns the outgoing dependency edge.
    pub caller_role: ArchitectureRole,
    /// Roles that the caller may depend on.
    pub allowed_callee_roles: Vec<ArchitectureRole>,
    /// Cargo dependency edge kinds covered by this rule.
    pub edge_kinds: Vec<String>,
}

/// Project classification for one external Cargo provider.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureProviderKind {
    /// Deterministic value or mechanism provider permitted by explicit caller roles.
    PureValue,
    /// Application contract provider permitted by explicit caller roles.
    ApplicationContract,
    /// Infrastructure provider restricted to adapters and composition roots.
    Infrastructure,
}

/// Exact external Cargo provider classification.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureProviderClassification {
    /// Cargo package name.
    pub package: String,
    /// Provider kind.
    pub kind: ArchitectureProviderKind,
    /// Production roles permitted to depend on this provider.
    pub permitted_caller_roles: Vec<ArchitectureRole>,
    /// Cargo dependency edge kinds covered by this classification.
    pub edge_kinds: Vec<String>,
    /// Reviewable classification reason.
    pub rationale: String,
}

/// Application-owned Hexagonal port declaration.
// r[impl octet.project_architecture_policy.ports]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitecturePort {
    /// Compiler-qualified port trait name.
    pub port_trait: String,
    /// Application-shell source scope that owns the port contract.
    pub application_owner_scope: String,
    /// Adapter source scopes permitted to implement the port.
    pub permitted_adapter_scopes: Vec<String>,
    /// Concrete infrastructure type origins forbidden in inward contracts.
    #[serde(default)]
    pub forbidden_concrete_types: Vec<String>,
    /// True when the port is reserved and does not require an implementation.
    pub reserved: bool,
}

/// Effect-family classification declared by project policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureCapabilityFamily {
    /// Time observation.
    Time,
    /// Random value generation.
    Randomness,
    /// Filesystem access.
    Filesystem,
    /// Network access.
    Network,
    /// Child-process control.
    Process,
    /// Environment access.
    Environment,
    /// Secret access.
    Secrets,
    /// Unsafe Rust or foreign memory operations.
    UnsafeCode,
    /// Ambient mutable global state.
    GlobalState,
    /// Task, thread, lock, or synchronization effects.
    Concurrency,
    /// Logging, tracing, or metrics effects.
    Observability,
    /// Foreign-function calls.
    Ffi,
    /// Resource allocation governed by project policy.
    Allocation,
}

/// Domain-core treatment for one capability family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoreCapabilityHandling {
    /// The core receives a prior observation as explicit data.
    ExplicitFact,
    /// The core returns a plan that the shell can interpret.
    EffectPlan,
    /// The capability family is forbidden in the core interface.
    Forbidden,
}

/// Project classification for one capability family.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCapabilityClassification {
    /// Capability family.
    pub family: ArchitectureCapabilityFamily,
    /// Permitted domain-core treatment.
    pub core_handling: CoreCapabilityHandling,
    /// Production role that owns execution.
    pub executor_role: ArchitectureRole,
    /// Reviewable reason for this classification.
    pub rationale: String,
}

/// Authority treatment for one outbound effect family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureEffectAuthority {
    /// Execution does not require an authorization witness under this project policy.
    Unprotected,
    /// Execution requires the complete typed authorization protocol.
    Protected,
}

/// How a protected executor receives its authorization witness.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureWitnessUse {
    /// The executor consumes a one-shot witness.
    Consume,
    /// The executor borrows a reusable witness within an explicit scope.
    Borrow,
}

/// Complete compiler-visible protocol for one protected outbound effect family.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureProtectedEffectProtocol {
    /// Domain-core source scope that owns authority decisions.
    pub decision_scope: String,
    /// Domain-core source scope that owns the opaque witness type.
    pub witness_owner_scope: String,
    /// Application-shell source scope that owns observation and receipt meaning.
    pub receipt_scope: String,
    /// Exact authority input type names.
    pub authority_input_types: Vec<String>,
    /// Exact approved domain decision function names.
    pub decision_functions: Vec<String>,
    /// Exact authorized decision output type names.
    pub authorized_output_types: Vec<String>,
    /// Exact opaque authorization witness type name.
    pub witness_type: String,
    /// Exact approved witness constructor names.
    pub witness_constructors: Vec<String>,
    /// Exact types that bind the witness to this effect family or plan.
    pub witness_binding_types: Vec<String>,
    /// Exact planned-effect type name.
    pub plan_type: String,
    /// Exact execution-attempt type name.
    pub attempt_type: String,
    /// Exact observed-outcome type name.
    pub observed_outcome_type: String,
    /// Exact execution-receipt type name.
    pub receipt_type: String,
    /// Exact success-receipt constructor names that require a successful observation.
    pub receipt_constructors: Vec<String>,
    /// Exact failure-receipt constructor names that require a failed or denied observation.
    #[serde(default)]
    pub failure_receipt_constructors: Vec<String>,
    /// Exact constructors for successful observations.
    pub success_observation_constructors: Vec<String>,
    /// Exact constructors for failed or denied observations.
    pub failure_observation_constructors: Vec<String>,
    /// Witness use cardinality.
    pub witness_use: ArchitectureWitnessUse,
    /// Reusable witness scope, required only for `borrow`.
    #[serde(default)]
    pub borrow_scope: Option<String>,
    /// Reusable authority rationale, required only for `borrow`.
    #[serde(default)]
    pub borrow_rationale: Option<String>,
    /// Explicit protocol non-claims.
    pub non_claims: Vec<String>,
}

/// Explicit authority classification for one outbound effect family.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureOutboundEffectClassification {
    /// Effect family.
    pub family: ArchitectureCapabilityFamily,
    /// Source scope that owns protected execution.
    pub executor_scope: String,
    /// Exact protected or unprotected executor operation names.
    pub executor_methods: Vec<String>,
    /// Authority treatment.
    pub authority: ArchitectureEffectAuthority,
    /// Reviewable classification reason.
    pub rationale: String,
    /// Complete protocol, required only for protected effects.
    #[serde(default)]
    pub protected: Option<ArchitectureProtectedEffectProtocol>,
}

/// Optional host-independence obligation for one domain-core scope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureNoStdObligation {
    /// Domain-core package or source scope that must remain `no_std`.
    pub scope: String,
}

/// Explicit binding to the existing effect-closure rail for admission mode.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureAdmissionBinding {
    /// Strict profile used by the direct prerequisite check.
    pub strict_profile: String,
    /// Workspace-relative effect-closure policy path.
    pub effect_closure_policy: String,
}

/// Exact, reviewable architecture finding waiver.
// r[impl octet.project_architecture_policy.waivers]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureWaiver {
    /// Stable architecture finding identity.
    pub finding_id: String,
    /// BLAKE3 identity of the policy used to create the finding.
    pub policy_blake3: String,
    /// BLAKE3 identity of the affected source material.
    pub source_blake3: String,
    /// Named waiver owner.
    pub owner: String,
    /// Reviewable waiver reason.
    pub reason: String,
    /// Condition that requires review or removal.
    pub review_condition: String,
    /// Exact claim affected by this waiver.
    pub affected_claim: String,
}

/// Typed project architecture policy consumed by Octet runtime code.
// r[impl octet.project_architecture_policy.schema]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitecturePolicy {
    /// Policy schema version.
    pub schema_version: String,
    /// Stable project identity.
    pub project_id: String,
    /// Project-selected enforcement mode.
    pub mode: ArchitectureMode,
    /// Production inventory and target coverage.
    pub production: ArchitectureProductionScope,
    /// Complete package, source, test, and generated role assignments.
    pub roles: Vec<ArchitectureRoleAssignment>,
    /// Role dependency rules.
    pub dependency_rules: Vec<ArchitectureDependencyRule>,
    /// Exact external Cargo provider classifications.
    #[serde(default)]
    pub provider_classifications: Vec<ArchitectureProviderClassification>,
    /// Application-owned ports.
    #[serde(default)]
    pub ports: Vec<ArchitecturePort>,
    /// Capability-family classifications.
    pub capability_classifications: Vec<ArchitectureCapabilityClassification>,
    /// Explicit outbound effect authority classifications.
    #[serde(default)]
    pub outbound_effects: Vec<ArchitectureOutboundEffectClassification>,
    /// Optional `no_std` obligations, separate from FCIS role meaning.
    #[serde(default)]
    pub core_no_std: Vec<ArchitectureNoStdObligation>,
    /// Explicit effect-closure binding required only by admission mode.
    #[serde(default)]
    pub admission: Option<ArchitectureAdmissionBinding>,
    /// Exact architecture finding waivers.
    #[serde(default)]
    pub waivers: Vec<ArchitectureWaiver>,
    /// Named bounds for compiler architecture evidence collection.
    pub collection_limits: octet_architecture_ir::ArchitectureCollectionLimits,
    /// Explicit project architecture non-claims.
    pub non_claims: Vec<String>,
}

/// Stable policy validation diagnostic.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ProjectArchitecturePolicyDiagnostic {
    /// Stable diagnostic class.
    pub code: String,
    /// Policy field or relation that failed.
    pub field: String,
    /// Reviewable diagnostic message.
    pub message: String,
}

/// Facts required to decide whether one exact waiver is current.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchitectureFindingIdentity<'a> {
    /// Stable architecture finding identity.
    pub finding_id: &'a str,
    /// BLAKE3 identity of the evaluated policy.
    pub policy_blake3: &'a str,
    /// BLAKE3 identity of the affected source.
    pub source_blake3: &'a str,
    /// Exact claim affected by the finding.
    pub affected_claim: &'a str,
}

/// Parse and validate one deterministic project architecture JSON export.
///
/// # Errors
///
/// Returns sorted deterministic diagnostics for malformed JSON or invalid
/// cross-field policy state.
pub fn parse_project_architecture_policy(
    bytes: &[u8],
) -> Result<ProjectArchitecturePolicy, Vec<ProjectArchitecturePolicyDiagnostic>> {
    let policy = serde_json::from_slice::<ProjectArchitecturePolicy>(bytes).map_err(|error| {
        vec![diagnostic(
            "policy-json",
            "export",
            format!("project architecture export is not valid policy JSON: {error}"),
        )]
    })?;
    let diagnostics = validate_project_architecture_policy(&policy);
    if diagnostics.is_empty() {
        Ok(policy)
    } else {
        Err(diagnostics)
    }
}

/// Validate one parsed project architecture policy without I/O.
#[must_use]
pub fn validate_project_architecture_policy(
    policy: &ProjectArchitecturePolicy,
) -> Vec<ProjectArchitecturePolicyDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_scalar_fields(policy, &mut diagnostics);
    let assignments = validate_scopes_and_roles(policy, &mut diagnostics);
    validate_targets(policy, &mut diagnostics);
    validate_dependency_rules(policy, &mut diagnostics);
    validate_provider_classifications(policy, &mut diagnostics);
    validate_ports(policy, &assignments, &mut diagnostics);
    validate_capabilities(policy, &mut diagnostics);
    validate_outbound_effects(policy, &assignments, &mut diagnostics);
    validate_no_std(policy, &assignments, &mut diagnostics);
    validate_admission(policy, &mut diagnostics);
    validate_waivers(policy, &mut diagnostics);
    validate_collection_limits(policy, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Validate one waiver against one current finding identity.
#[must_use]
pub fn validate_architecture_waiver_binding(
    waiver: &ArchitectureWaiver,
    finding: &ArchitectureFindingIdentity<'_>,
) -> Vec<ProjectArchitecturePolicyDiagnostic> {
    let mut diagnostics = Vec::new();
    push_binding_mismatch(&mut diagnostics, "finding_id", &waiver.finding_id, finding.finding_id);
    push_binding_mismatch(&mut diagnostics, "policy_blake3", &waiver.policy_blake3, finding.policy_blake3);
    push_binding_mismatch(&mut diagnostics, "source_blake3", &waiver.source_blake3, finding.source_blake3);
    push_binding_mismatch(&mut diagnostics, "affected_claim", &waiver.affected_claim, finding.affected_claim);
    diagnostics.sort();
    diagnostics
}

fn validate_scalar_fields(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if policy.schema_version != PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            "schema-version",
            "schema_version",
            format!(
                "unsupported project architecture schema `{}`; expected `{PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION}`",
                policy.schema_version
            ),
        ));
    }
    push_blank(diagnostics, "project_id", &policy.project_id);
    if policy.production.packages.is_empty() {
        diagnostics.push(diagnostic(
            "production-coverage",
            "production.packages",
            "project architecture policy must declare at least one production package",
        ));
    }
    if policy.production.source_scopes.is_empty() {
        diagnostics.push(diagnostic(
            "production-coverage",
            "production.source_scopes",
            "project architecture policy must declare at least one production source scope",
        ));
    }
    if policy.non_claims.is_empty() {
        diagnostics.push(diagnostic(
            "missing-non-claims",
            "non_claims",
            "project architecture policy must declare at least one non-claim",
        ));
    }
    validate_non_empty_unique_strings("non_claims", &policy.non_claims, diagnostics);
}

fn validate_collection_limits(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let limits = &policy.collection_limits;
    for (field, value) in [
        ("max_units", limits.max_units),
        ("max_shards", limits.max_shards),
        ("max_modules", limits.max_modules),
        ("max_items", limits.max_items),
        ("max_types", limits.max_types),
        ("max_traits", limits.max_traits),
        ("max_implementations", limits.max_implementations),
        ("max_signatures", limits.max_signatures),
        ("max_calls", limits.max_calls),
        ("max_constructors", limits.max_constructors),
        ("max_effects", limits.max_effects),
        ("max_expansions", limits.max_expansions),
        ("max_configurations", limits.max_configurations),
        ("max_type_shapes", limits.max_type_shapes),
        ("max_function_safety_facts", limits.max_function_safety_facts),
        ("max_value_provenance_facts", limits.max_value_provenance_facts),
        ("max_value_flow_bodies", limits.max_value_flow_bodies),
        ("max_value_flow_steps", limits.max_value_flow_steps),
        ("max_value_flow_rounds", limits.max_value_flow_rounds),
        ("max_unknowns", limits.max_unknowns),
        ("max_spans", limits.max_spans),
        ("max_span_bytes", limits.max_span_bytes),
        ("max_shard_bytes", limits.max_shard_bytes),
        ("max_artifact_bytes", limits.max_artifact_bytes),
    ] {
        if value == 0 {
            diagnostics.push(diagnostic(
                "collection-limit",
                format!("collection_limits.{field}"),
                "compiler architecture collection limits must be positive",
            ));
        }
    }
    if limits.max_shards < limits.max_units {
        diagnostics.push(diagnostic(
            "collection-limit",
            "collection_limits.max_shards",
            "compiler architecture shard limit must be at least the compilation-unit limit",
        ));
    }
}

fn validate_scopes_and_roles(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) -> BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole> {
    validate_non_empty_unique_strings("production.packages", &policy.production.packages, diagnostics);
    validate_non_empty_unique_strings("production.source_scopes", &policy.production.source_scopes, diagnostics);
    validate_workspace_relative_paths("production.source_scopes", &policy.production.source_scopes, diagnostics);

    let expected_packages = policy.production.packages.iter().cloned().collect::<BTreeSet<_>>();
    let expected_sources = policy.production.source_scopes.iter().cloned().collect::<BTreeSet<_>>();
    let mut assignments = BTreeMap::new();
    for assignment in &policy.roles {
        push_blank(diagnostics, "roles.scope", &assignment.scope);
        if assignment.scope_kind == ArchitectureScopeKind::Source {
            validate_workspace_relative_path("roles.scope", &assignment.scope, diagnostics);
        }
        let key = (assignment.scope_kind, assignment.scope.clone());
        if assignments.insert(key, assignment.role).is_some() {
            diagnostics.push(diagnostic(
                "ambiguous-role",
                "roles",
                format!("scope `{}` has more than one architecture role", assignment.scope),
            ));
        }
        let is_production_scope = match assignment.scope_kind {
            ArchitectureScopeKind::Package => expected_packages.contains(&assignment.scope),
            ArchitectureScopeKind::Source => expected_sources.contains(&assignment.scope),
        };
        if is_production_scope != assignment.role.is_production() {
            diagnostics.push(diagnostic(
                "role-coverage",
                "roles",
                format!(
                    "scope `{}` has role `{:?}` that does not match its production inclusion",
                    assignment.scope, assignment.role
                ),
            ));
        }
    }

    for package in &expected_packages {
        require_assignment(&assignments, ArchitectureScopeKind::Package, package, diagnostics);
    }
    for source in &expected_sources {
        require_assignment(&assignments, ArchitectureScopeKind::Source, source, diagnostics);
    }

    let source_assignments = policy
        .roles
        .iter()
        .filter(|assignment| assignment.scope_kind == ArchitectureScopeKind::Source)
        .collect::<Vec<_>>();
    for (index, left) in source_assignments.iter().enumerate() {
        for right in source_assignments.iter().skip(index.saturating_add(1)) {
            if paths_overlap(&left.scope, &right.scope) {
                diagnostics.push(diagnostic(
                    "overlapping-scope",
                    "roles",
                    format!("source scopes `{}` and `{}` overlap", left.scope, right.scope),
                ));
            }
        }
    }
    assignments
}

fn validate_targets(policy: &ProjectArchitecturePolicy, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    if policy.production.targets.is_empty() {
        diagnostics.push(diagnostic(
            "target-coverage",
            "production.targets",
            "project architecture policy must declare at least one target selection",
        ));
        return;
    }
    let mut target_keys = BTreeSet::new();
    for target in &policy.production.targets {
        push_blank(diagnostics, "production.targets.triple", &target.triple);
        validate_non_empty_unique_strings("production.targets.features", &target.features, diagnostics);
        let mut features = target.features.clone();
        features.sort();
        let key = (target.triple.clone(), features, target.default_features);
        if !target_keys.insert(key) {
            diagnostics.push(diagnostic(
                "duplicate-target",
                "production.targets",
                format!("target selection `{}` is declared more than once", target.triple),
            ));
        }
    }
}

fn validate_dependency_rules(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let production_roles = policy
        .roles
        .iter()
        .filter(|assignment| assignment.role.is_production())
        .map(|assignment| assignment.role)
        .collect::<BTreeSet<_>>();
    let mut callers = BTreeSet::new();
    for rule in &policy.dependency_rules {
        if !rule.caller_role.is_production() {
            diagnostics.push(diagnostic(
                "invalid-role-rule",
                "dependency_rules.caller_role",
                "test and generated roles cannot own production dependency rules",
            ));
        }
        if !callers.insert(rule.caller_role) {
            diagnostics.push(diagnostic(
                "duplicate-role-rule",
                "dependency_rules",
                format!("role `{:?}` has more than one dependency rule", rule.caller_role),
            ));
        }
        if rule.allowed_callee_roles.iter().any(|role| !role.is_production()) {
            diagnostics.push(diagnostic(
                "invalid-role-rule",
                "dependency_rules.allowed_callee_roles",
                "production dependency rules cannot permit test or generated roles",
            ));
        }
        if rule.edge_kinds.is_empty() {
            diagnostics.push(diagnostic(
                "missing-edge-kinds",
                "dependency_rules.edge_kinds",
                "each dependency rule must select at least one Cargo edge kind",
            ));
        }
        validate_non_empty_unique_strings("dependency_rules.edge_kinds", &rule.edge_kinds, diagnostics);
    }
    for role in production_roles {
        if !callers.contains(&role) {
            diagnostics.push(diagnostic(
                "missing-role-rule",
                "dependency_rules",
                format!("production role `{role:?}` has no dependency rule"),
            ));
        }
    }
}

fn validate_provider_classifications(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let production_packages = policy.production.packages.iter().collect::<BTreeSet<_>>();
    let mut packages = BTreeSet::new();
    for provider in &policy.provider_classifications {
        push_blank(diagnostics, "provider_classifications.package", &provider.package);
        push_blank(diagnostics, "provider_classifications.rationale", &provider.rationale);
        if !packages.insert(provider.package.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-provider-classification",
                "provider_classifications.package",
                format!("external provider `{}` is classified more than once", provider.package),
            ));
        }
        if production_packages.contains(&provider.package) {
            diagnostics.push(diagnostic(
                "internal-provider-classification",
                "provider_classifications.package",
                format!(
                    "production package `{}` must use its role, not an external provider classification",
                    provider.package
                ),
            ));
        }
        if provider.permitted_caller_roles.is_empty()
            || provider.permitted_caller_roles.iter().any(|role| !role.is_production())
        {
            diagnostics.push(diagnostic(
                "provider-caller-roles",
                "provider_classifications.permitted_caller_roles",
                "external provider classification must permit at least one production caller role",
            ));
        }
        if provider.edge_kinds.is_empty() {
            diagnostics.push(diagnostic(
                "provider-edge-kinds",
                "provider_classifications.edge_kinds",
                "external provider classification must select at least one Cargo edge kind",
            ));
        }
        validate_non_empty_unique_strings("provider_classifications.edge_kinds", &provider.edge_kinds, diagnostics);
    }
}

fn validate_ports(
    policy: &ProjectArchitecturePolicy,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut traits = BTreeSet::new();
    for port in &policy.ports {
        push_blank(diagnostics, "ports.port_trait", &port.port_trait);
        if !traits.insert(port.port_trait.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-port",
                "ports.port_trait",
                format!("port trait `{}` is declared more than once", port.port_trait),
            ));
        }
        require_scope_role(
            assignments,
            &port.application_owner_scope,
            ArchitectureRole::ApplicationShell,
            "ports.application_owner_scope",
            diagnostics,
        );
        if !port.reserved && port.permitted_adapter_scopes.is_empty() {
            diagnostics.push(diagnostic(
                "missing-adapter-scope",
                "ports.permitted_adapter_scopes",
                format!("non-reserved port `{}` has no permitted adapter scope", port.port_trait),
            ));
        }
        validate_non_empty_unique_strings(
            "ports.permitted_adapter_scopes",
            &port.permitted_adapter_scopes,
            diagnostics,
        );
        validate_non_empty_unique_strings(
            "ports.forbidden_concrete_types",
            &port.forbidden_concrete_types,
            diagnostics,
        );
        for adapter_scope in &port.permitted_adapter_scopes {
            let role = assignments.get(&(ArchitectureScopeKind::Source, adapter_scope.clone()));
            if !matches!(role, Some(ArchitectureRole::InboundAdapter | ArchitectureRole::OutboundAdapter)) {
                diagnostics.push(diagnostic(
                    "invalid-adapter-role",
                    "ports.permitted_adapter_scopes",
                    format!("port adapter scope `{adapter_scope}` is not assigned an adapter role"),
                ));
            }
        }
    }
}

fn validate_capabilities(
    policy: &ProjectArchitecturePolicy,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if policy.capability_classifications.is_empty() {
        diagnostics.push(diagnostic(
            "missing-capability-classification",
            "capability_classifications",
            "project architecture policy must classify at least one capability family",
        ));
    }
    let mut families = BTreeSet::new();
    for classification in &policy.capability_classifications {
        if !families.insert(classification.family) {
            diagnostics.push(diagnostic(
                "duplicate-capability-family",
                "capability_classifications.family",
                format!("capability family `{:?}` is classified more than once", classification.family),
            ));
        }
        if !matches!(
            classification.executor_role,
            ArchitectureRole::ApplicationShell | ArchitectureRole::OutboundAdapter | ArchitectureRole::CompositionRoot
        ) {
            diagnostics.push(diagnostic(
                "invalid-capability-executor",
                "capability_classifications.executor_role",
                "capability execution must belong to the application shell, an outbound adapter, or the composition root",
            ));
        }
        push_blank(diagnostics, "capability_classifications.rationale", &classification.rationale);
    }
}

fn validate_outbound_effects(
    policy: &ProjectArchitecturePolicy,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut families = BTreeSet::new();
    for classification in &policy.outbound_effects {
        if !families.insert(classification.family) {
            diagnostics.push(diagnostic(
                "duplicate-outbound-effect-family",
                "outbound_effects.family",
                format!("outbound effect family `{:?}` is classified more than once", classification.family),
            ));
        }
        validate_non_empty_unique_strings(
            "outbound_effects.executor_methods",
            &classification.executor_methods,
            diagnostics,
        );
        if classification.executor_methods.is_empty() {
            diagnostics.push(diagnostic(
                "missing-executor-method",
                "outbound_effects.executor_methods",
                "outbound effect classification must declare at least one exact executor method",
            ));
        }
        push_blank(diagnostics, "outbound_effects.executor_scope", &classification.executor_scope);
        push_blank(diagnostics, "outbound_effects.rationale", &classification.rationale);
        let executor_role = assignments.get(&(ArchitectureScopeKind::Source, classification.executor_scope.clone()));
        if !matches!(
            executor_role,
            Some(
                ArchitectureRole::ApplicationShell
                    | ArchitectureRole::OutboundAdapter
                    | ArchitectureRole::CompositionRoot
            )
        ) {
            diagnostics.push(diagnostic(
                "invalid-effect-executor-scope",
                "outbound_effects.executor_scope",
                format!(
                    "outbound executor scope `{}` must be application-shell, outbound-adapter, or composition-root",
                    classification.executor_scope
                ),
            ));
        }
        match (classification.authority, classification.protected.as_ref()) {
            (ArchitectureEffectAuthority::Unprotected, None) => {}
            (ArchitectureEffectAuthority::Unprotected, Some(_)) => diagnostics.push(diagnostic(
                "unexpected-protected-protocol",
                "outbound_effects.protected",
                "unprotected outbound effects cannot define a protected protocol",
            )),
            (ArchitectureEffectAuthority::Protected, None) => diagnostics.push(diagnostic(
                "missing-protected-protocol",
                "outbound_effects.protected",
                "protected outbound effects require a complete authorization protocol",
            )),
            (ArchitectureEffectAuthority::Protected, Some(protocol)) => {
                validate_protected_effect_protocol(protocol, assignments, diagnostics);
            }
        }
    }
}

#[allow(clippy::too_many_lines)]
fn validate_protected_effect_protocol(
    protocol: &ArchitectureProtectedEffectProtocol,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    for (field, scope, role) in [
        ("outbound_effects.protected.decision_scope", &protocol.decision_scope, ArchitectureRole::DomainCore),
        (
            "outbound_effects.protected.witness_owner_scope",
            &protocol.witness_owner_scope,
            ArchitectureRole::DomainCore,
        ),
        (
            "outbound_effects.protected.receipt_scope",
            &protocol.receipt_scope,
            ArchitectureRole::ApplicationShell,
        ),
    ] {
        push_blank(diagnostics, field, scope);
        require_scope_role(assignments, scope, role, field, diagnostics);
    }
    for (field, values) in [
        ("outbound_effects.protected.authority_input_types", protocol.authority_input_types.as_slice()),
        ("outbound_effects.protected.decision_functions", protocol.decision_functions.as_slice()),
        ("outbound_effects.protected.authorized_output_types", protocol.authorized_output_types.as_slice()),
        ("outbound_effects.protected.witness_constructors", protocol.witness_constructors.as_slice()),
        ("outbound_effects.protected.witness_binding_types", protocol.witness_binding_types.as_slice()),
        ("outbound_effects.protected.receipt_constructors", protocol.receipt_constructors.as_slice()),
        (
            "outbound_effects.protected.failure_receipt_constructors",
            protocol.failure_receipt_constructors.as_slice(),
        ),
        (
            "outbound_effects.protected.success_observation_constructors",
            protocol.success_observation_constructors.as_slice(),
        ),
        (
            "outbound_effects.protected.failure_observation_constructors",
            protocol.failure_observation_constructors.as_slice(),
        ),
        ("outbound_effects.protected.non_claims", protocol.non_claims.as_slice()),
    ] {
        validate_non_empty_unique_strings(field, values, diagnostics);
        if values.is_empty() {
            diagnostics.push(diagnostic(
                "missing-protocol-field",
                field,
                "protected effect protocol arrays must contain at least one exact value",
            ));
        }
    }
    for (left, right, code) in [
        (
            &protocol.receipt_constructors,
            &protocol.failure_receipt_constructors,
            "receipt-constructor-overlap",
        ),
        (
            &protocol.success_observation_constructors,
            &protocol.failure_observation_constructors,
            "observation-constructor-overlap",
        ),
    ] {
        if left.iter().any(|value| right.contains(value)) {
            diagnostics.push(diagnostic(
                code,
                "outbound_effects.protected",
                "success and failure constructor sets must be disjoint",
            ));
        }
    }
    for (field, value) in [
        ("outbound_effects.protected.witness_type", &protocol.witness_type),
        ("outbound_effects.protected.plan_type", &protocol.plan_type),
        ("outbound_effects.protected.attempt_type", &protocol.attempt_type),
        ("outbound_effects.protected.observed_outcome_type", &protocol.observed_outcome_type),
        ("outbound_effects.protected.receipt_type", &protocol.receipt_type),
    ] {
        push_blank(diagnostics, field, value);
    }
    let distinct_types = [
        protocol.witness_type.as_str(),
        protocol.plan_type.as_str(),
        protocol.attempt_type.as_str(),
        protocol.observed_outcome_type.as_str(),
        protocol.receipt_type.as_str(),
    ];
    if distinct_types.iter().copied().collect::<BTreeSet<_>>().len() != distinct_types.len() {
        diagnostics.push(diagnostic(
            "protocol-type-alias",
            "outbound_effects.protected",
            "witness, plan, attempt, observation, and receipt must use distinct qualified types",
        ));
    }
    match protocol.witness_use {
        ArchitectureWitnessUse::Consume => {
            if protocol.borrow_scope.is_some() || protocol.borrow_rationale.is_some() {
                diagnostics.push(diagnostic(
                    "unexpected-borrow-policy",
                    "outbound_effects.protected.witness_use",
                    "consume witnesses cannot define a reusable borrow scope or rationale",
                ));
            }
        }
        ArchitectureWitnessUse::Borrow => match (&protocol.borrow_scope, &protocol.borrow_rationale) {
            (Some(scope), Some(rationale)) => {
                push_blank(diagnostics, "outbound_effects.protected.borrow_scope", scope);
                push_blank(diagnostics, "outbound_effects.protected.borrow_rationale", rationale);
                if !assignments.contains_key(&(ArchitectureScopeKind::Source, scope.clone())) {
                    diagnostics.push(diagnostic(
                        "invalid-borrow-scope",
                        "outbound_effects.protected.borrow_scope",
                        format!("reusable witness scope `{scope}` has no explicit source role"),
                    ));
                }
            }
            _ => diagnostics.push(diagnostic(
                "missing-borrow-policy",
                "outbound_effects.protected.witness_use",
                "borrow witnesses require both a reusable-authority scope and rationale",
            )),
        },
    }
}

fn validate_no_std(
    policy: &ProjectArchitecturePolicy,
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut scopes = BTreeSet::new();
    for obligation in &policy.core_no_std {
        push_blank(diagnostics, "core_no_std.scope", &obligation.scope);
        if !scopes.insert(obligation.scope.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-no-std-scope",
                "core_no_std.scope",
                format!("no_std scope `{}` is declared more than once", obligation.scope),
            ));
        }
        let role = assignments
            .get(&(ArchitectureScopeKind::Package, obligation.scope.clone()))
            .or_else(|| assignments.get(&(ArchitectureScopeKind::Source, obligation.scope.clone())));
        if role != Some(&ArchitectureRole::DomainCore) {
            diagnostics.push(diagnostic(
                "invalid-no-std-role",
                "core_no_std.scope",
                format!("no_std scope `{}` is not assigned domain-core", obligation.scope),
            ));
        }
    }
}

fn validate_admission(policy: &ProjectArchitecturePolicy, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    match (&policy.mode, &policy.admission) {
        (ArchitectureMode::Admission, None) => diagnostics.push(diagnostic(
            "missing-admission-binding",
            "admission",
            "admission mode requires an explicit strict profile and effect-closure policy path",
        )),
        (ArchitectureMode::Admission, Some(binding)) => {
            push_blank(diagnostics, "admission.strict_profile", &binding.strict_profile);
            push_blank(diagnostics, "admission.effect_closure_policy", &binding.effect_closure_policy);
            if binding.strict_profile != "deterministic-core" {
                diagnostics.push(diagnostic(
                    "invalid-admission-profile",
                    "admission.strict_profile",
                    "project architecture admission requires the deterministic-core strict profile",
                ));
            }
            validate_workspace_relative_path(
                "admission.effect_closure_policy",
                &binding.effect_closure_policy,
                diagnostics,
            );
        }
        (_, Some(_)) => diagnostics.push(diagnostic(
            "unexpected-admission-binding",
            "admission",
            "only admission mode can define an admission binding",
        )),
        _ => {}
    }
}

fn validate_waivers(policy: &ProjectArchitecturePolicy, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    let mut finding_ids = BTreeSet::new();
    for waiver in &policy.waivers {
        push_blank(diagnostics, "waivers.finding_id", &waiver.finding_id);
        push_blank(diagnostics, "waivers.owner", &waiver.owner);
        push_blank(diagnostics, "waivers.reason", &waiver.reason);
        push_blank(diagnostics, "waivers.review_condition", &waiver.review_condition);
        push_blank(diagnostics, "waivers.affected_claim", &waiver.affected_claim);
        validate_blake3("waivers.policy_blake3", &waiver.policy_blake3, diagnostics);
        validate_blake3("waivers.source_blake3", &waiver.source_blake3, diagnostics);
        if !finding_ids.insert(waiver.finding_id.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-waiver",
                "waivers.finding_id",
                format!("finding `{}` has more than one architecture waiver", waiver.finding_id),
            ));
        }
    }
}

fn require_assignment(
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    kind: ArchitectureScopeKind,
    scope: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    if !assignments.contains_key(&(kind, scope.to_owned())) {
        diagnostics.push(diagnostic(
            "missing-role",
            "roles",
            format!("production scope `{scope}` has no architecture role"),
        ));
    }
}

fn require_scope_role(
    assignments: &BTreeMap<(ArchitectureScopeKind, String), ArchitectureRole>,
    scope: &str,
    required: ArchitectureRole,
    field: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let actual = assignments.get(&(ArchitectureScopeKind::Source, scope.to_owned()));
    if actual != Some(&required) {
        diagnostics.push(diagnostic(
            "invalid-scope-role",
            field,
            format!("scope `{scope}` must have role `{required:?}`; found {actual:?}"),
        ));
    }
}

fn validate_non_empty_unique_strings(
    field: &str,
    values: &[String],
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for value in values {
        push_blank(diagnostics, field, value);
        if !seen.insert(value.clone()) {
            diagnostics.push(diagnostic(
                "duplicate-value",
                field,
                format!("value `{value}` is declared more than once"),
            ));
        }
    }
}

fn validate_workspace_relative_paths(
    field: &str,
    paths: &[String],
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    for path in paths {
        validate_workspace_relative_path(field, path, diagnostics);
    }
}

fn validate_workspace_relative_path(
    field: &str,
    path: &str,
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
) {
    let normalized = path.replace('\\', "/");
    let escapes = normalized.starts_with(PATH_SEPARATOR)
        || normalized.split(PATH_SEPARATOR).any(|component| component == "..")
        || normalized.is_empty()
        || normalized == ".";
    if escapes {
        diagnostics.push(diagnostic(
            "invalid-workspace-path",
            field,
            format!("path `{path}` must be workspace-relative and must not escape the workspace"),
        ));
    }
}

fn paths_overlap(left: &str, right: &str) -> bool {
    let left = left.trim_end_matches(PATH_SEPARATOR);
    let right = right.trim_end_matches(PATH_SEPARATOR);
    left == right
        || left.strip_prefix(right).is_some_and(|suffix| suffix.starts_with(PATH_SEPARATOR))
        || right.strip_prefix(left).is_some_and(|suffix| suffix.starts_with(PATH_SEPARATOR))
}

fn validate_blake3(field: &str, value: &str, diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>) {
    let valid = value.len() == BLAKE3_HEX_LENGTH
        && value.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
    if !valid {
        diagnostics.push(diagnostic(
            "invalid-blake3",
            field,
            format!("BLAKE3 identity must contain exactly {BLAKE3_HEX_LENGTH} lowercase hexadecimal characters"),
        ));
    }
}

fn push_blank(diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(diagnostic(
            "empty-field",
            field,
            format!("project architecture field `{field}` must not be empty"),
        ));
    }
}

fn push_binding_mismatch(
    diagnostics: &mut Vec<ProjectArchitecturePolicyDiagnostic>,
    field: &str,
    actual: &str,
    expected: &str,
) {
    if actual != expected {
        diagnostics.push(diagnostic(
            "stale-waiver",
            format!("waiver.{field}"),
            format!("architecture waiver `{field}` is stale; expected `{expected}`, found `{actual}`"),
        ));
    }
}

fn diagnostic(
    code: impl Into<String>,
    field: impl Into<String>,
    message: impl Into<String>,
) -> ProjectArchitecturePolicyDiagnostic {
    ProjectArchitecturePolicyDiagnostic {
        code: code.into(),
        field: field.into(),
        message: message.into(),
    }
}

// r[impl octet.project_architecture_policy.verification]
#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const STALE_BINDING_DIAGNOSTICS: usize = 2;

    #[test]
    fn valid_policy_accepts_every_mode_and_role() {
        let mut policy = valid_policy();
        let modes = [
            ArchitectureMode::Inventory,
            ArchitectureMode::Advisory,
            ArchitectureMode::Gate,
            ArchitectureMode::Admission,
        ];
        for mode in modes {
            policy.mode = mode;
            policy.admission = (mode == ArchitectureMode::Admission).then(|| ArchitectureAdmissionBinding {
                strict_profile: "deterministic-core".to_owned(),
                effect_closure_policy: "octet-effect-closure.json".to_owned(),
            });
            assert_eq!(validate_project_architecture_policy(&policy), Vec::new());
        }
        policy.mode = ArchitectureMode::Inventory;
        policy.admission = Some(ArchitectureAdmissionBinding {
            strict_profile: "deterministic-core".to_owned(),
            effect_closure_policy: "octet-effect-closure.json".to_owned(),
        });
        assert!(
            validate_project_architecture_policy(&policy)
                .iter()
                .any(|diagnostic| diagnostic.code == "unexpected-admission-binding")
        );

        let roles = policy.roles.iter().map(|assignment| assignment.role).collect::<BTreeSet<_>>();
        assert_eq!(
            roles,
            BTreeSet::from([
                ArchitectureRole::DomainCore,
                ArchitectureRole::ApplicationShell,
                ArchitectureRole::InboundAdapter,
                ArchitectureRole::OutboundAdapter,
                ArchitectureRole::CompositionRoot,
                ArchitectureRole::Test,
                ArchitectureRole::Generated,
            ])
        );
    }

    #[test]
    fn invalid_policy_rejects_unknown_role_overlap_and_missing_coverage() {
        let mut value = serde_json::to_value(valid_policy()).expect("serialize valid policy");
        value["roles"][0]["role"] = serde_json::Value::String("core-ish".to_owned());
        let unknown = serde_json::to_vec(&value).expect("serialize malformed policy");
        let unknown_diagnostics = parse_project_architecture_policy(&unknown).expect_err("unknown role must fail");
        assert!(unknown_diagnostics.iter().any(|diagnostic| diagnostic.code == "policy-json"));

        let mut policy = valid_policy();
        policy.roles.retain(|assignment| assignment.scope != "crates/domain/src");
        policy.roles.push(ArchitectureRoleAssignment {
            scope_kind: ArchitectureScopeKind::Source,
            scope: "crates/application/src/nested".to_owned(),
            role: ArchitectureRole::Generated,
        });
        let diagnostics = validate_project_architecture_policy(&policy);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "missing-role"));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "overlapping-scope"));
    }

    #[test]
    fn protected_effect_protocol_accepts_complete_consume_and_borrow_forms() {
        let mut policy = valid_policy();
        policy.outbound_effects.push(protected_filesystem_effect(ArchitectureWitnessUse::Consume));
        assert_eq!(validate_project_architecture_policy(&policy), Vec::new());

        let mut borrow = protected_filesystem_effect(ArchitectureWitnessUse::Borrow);
        let protocol = borrow.protected.as_mut().expect("protected protocol");
        protocol.borrow_scope = Some("crates/application/src".to_owned());
        protocol.borrow_rationale = Some("bounded transaction authority is reused within one request".to_owned());
        policy.outbound_effects = vec![borrow];
        assert_eq!(validate_project_architecture_policy(&policy), Vec::new());
    }

    #[test]
    fn protected_effect_protocol_rejects_missing_protocol_aliases_and_implicit_borrow() {
        let mut policy = valid_policy();
        let mut classification = protected_filesystem_effect(ArchitectureWitnessUse::Borrow);
        let protocol = classification.protected.as_mut().expect("protected protocol");
        protocol.receipt_type.clone_from(&protocol.plan_type);
        policy.outbound_effects.push(classification);

        let diagnostics = validate_project_architecture_policy(&policy);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "protocol-type-alias"));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "missing-borrow-policy"));

        policy.outbound_effects[0].protected = None;
        let missing = validate_project_architecture_policy(&policy);
        assert!(missing.iter().any(|diagnostic| diagnostic.code == "missing-protected-protocol"));
    }

    #[test]
    fn waiver_binding_accepts_current_and_rejects_stale_facts() {
        let waiver = ArchitectureWaiver {
            finding_id: "architecture.port.owner".to_owned(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: HASH_B.to_owned(),
            owner: "architecture-team".to_owned(),
            reason: "migration is bounded".to_owned(),
            review_condition: "remove after adapter migration".to_owned(),
            affected_claim: "application-owned-port".to_owned(),
        };
        let current = ArchitectureFindingIdentity {
            finding_id: "architecture.port.owner",
            policy_blake3: HASH_A,
            source_blake3: HASH_B,
            affected_claim: "application-owned-port",
        };
        assert_eq!(validate_architecture_waiver_binding(&waiver, &current), Vec::new());

        let stale = ArchitectureFindingIdentity {
            finding_id: current.finding_id,
            policy_blake3: HASH_B,
            source_blake3: HASH_A,
            affected_claim: current.affected_claim,
        };
        let diagnostics = validate_architecture_waiver_binding(&waiver, &stale);
        assert_eq!(diagnostics.len(), STALE_BINDING_DIAGNOSTICS);
        assert!(diagnostics.iter().all(|diagnostic| diagnostic.code == "stale-waiver"));
    }

    #[allow(clippy::too_many_lines)]
    fn valid_policy() -> ProjectArchitecturePolicy {
        let production_packages = ["domain", "application", "inbound", "outbound", "host"];
        let production_sources = [
            "crates/domain/src",
            "crates/application/src",
            "crates/inbound/src",
            "crates/outbound/src",
            "crates/host/src",
        ];
        ProjectArchitecturePolicy {
            schema_version: PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION.to_owned(),
            project_id: "example-project".to_owned(),
            mode: ArchitectureMode::Gate,
            production: ArchitectureProductionScope {
                packages: production_packages.iter().map(ToString::to_string).collect(),
                source_scopes: production_sources.iter().map(ToString::to_string).collect(),
                targets: vec![ArchitectureTarget {
                    triple: "x86_64-unknown-linux-gnu".to_owned(),
                    features: vec!["production".to_owned()],
                    default_features: false,
                }],
            },
            roles: vec![
                role(ArchitectureScopeKind::Package, "domain", ArchitectureRole::DomainCore),
                role(ArchitectureScopeKind::Package, "application", ArchitectureRole::ApplicationShell),
                role(ArchitectureScopeKind::Package, "inbound", ArchitectureRole::InboundAdapter),
                role(ArchitectureScopeKind::Package, "outbound", ArchitectureRole::OutboundAdapter),
                role(ArchitectureScopeKind::Package, "host", ArchitectureRole::CompositionRoot),
                role(ArchitectureScopeKind::Source, "crates/domain/src", ArchitectureRole::DomainCore),
                role(ArchitectureScopeKind::Source, "crates/application/src", ArchitectureRole::ApplicationShell),
                role(ArchitectureScopeKind::Source, "crates/inbound/src", ArchitectureRole::InboundAdapter),
                role(ArchitectureScopeKind::Source, "crates/outbound/src", ArchitectureRole::OutboundAdapter),
                role(ArchitectureScopeKind::Source, "crates/host/src", ArchitectureRole::CompositionRoot),
                role(ArchitectureScopeKind::Source, "tests", ArchitectureRole::Test),
                role(ArchitectureScopeKind::Source, "target/generated", ArchitectureRole::Generated),
            ],
            dependency_rules: vec![
                dependency_rule(ArchitectureRole::DomainCore, &[ArchitectureRole::DomainCore]),
                dependency_rule(ArchitectureRole::ApplicationShell, &[
                    ArchitectureRole::ApplicationShell,
                    ArchitectureRole::DomainCore,
                ]),
                dependency_rule(ArchitectureRole::InboundAdapter, &[
                    ArchitectureRole::InboundAdapter,
                    ArchitectureRole::ApplicationShell,
                ]),
                dependency_rule(ArchitectureRole::OutboundAdapter, &[
                    ArchitectureRole::OutboundAdapter,
                    ArchitectureRole::ApplicationShell,
                ]),
                dependency_rule(ArchitectureRole::CompositionRoot, &[
                    ArchitectureRole::CompositionRoot,
                    ArchitectureRole::ApplicationShell,
                    ArchitectureRole::InboundAdapter,
                    ArchitectureRole::OutboundAdapter,
                ]),
            ],
            provider_classifications: Vec::new(),
            ports: vec![ArchitecturePort {
                port_trait: "application::Clock".to_owned(),
                application_owner_scope: "crates/application/src".to_owned(),
                permitted_adapter_scopes: vec!["crates/outbound/src".to_owned()],
                forbidden_concrete_types: vec!["std::time::SystemTime".to_owned()],
                reserved: false,
            }],
            capability_classifications: vec![ArchitectureCapabilityClassification {
                family: ArchitectureCapabilityFamily::Time,
                core_handling: CoreCapabilityHandling::ExplicitFact,
                executor_role: ArchitectureRole::OutboundAdapter,
                rationale: "the shell observes time before domain evaluation".to_owned(),
            }],
            outbound_effects: Vec::new(),
            core_no_std: vec![ArchitectureNoStdObligation {
                scope: "domain".to_owned(),
            }],
            admission: None,
            waivers: vec![ArchitectureWaiver {
                finding_id: "architecture.example".to_owned(),
                policy_blake3: HASH_A.to_owned(),
                source_blake3: HASH_B.to_owned(),
                owner: "architecture-team".to_owned(),
                reason: "bounded migration".to_owned(),
                review_condition: "remove after migration".to_owned(),
                affected_claim: "example-claim".to_owned(),
            }],
            collection_limits: octet_architecture_ir::ArchitectureCollectionLimits {
                max_units: 32,
                max_shards: 32,
                max_modules: 4096,
                max_items: 65_536,
                max_types: 65_536,
                max_traits: 8192,
                max_implementations: 16_384,
                max_signatures: 65_536,
                max_calls: 262_144,
                max_constructors: 65_536,
                max_effects: 65_536,
                max_expansions: 65_536,
                max_configurations: 8192,
                max_type_shapes: 65_536,
                max_function_safety_facts: 65_536,
                max_value_provenance_facts: 262_144,
                max_value_flow_bodies: 65_536,
                max_value_flow_steps: 1_048_576,
                max_value_flow_rounds: 64,
                max_unknowns: 65_536,
                max_spans: 262_144,
                max_span_bytes: 1_048_576,
                max_shard_bytes: 67_108_864,
                max_artifact_bytes: 268_435_456,
            },
            non_claims: vec!["runtime-correctness".to_owned(), "release-eligibility".to_owned()],
        }
    }

    fn protected_filesystem_effect(witness_use: ArchitectureWitnessUse) -> ArchitectureOutboundEffectClassification {
        ArchitectureOutboundEffectClassification {
            family: ArchitectureCapabilityFamily::Filesystem,
            executor_scope: "crates/outbound/src".to_owned(),
            executor_methods: vec!["outbound::FilesystemPort::publish".to_owned()],
            authority: ArchitectureEffectAuthority::Protected,
            rationale: "publication requires an application-owned authority decision".to_owned(),
            protected: Some(ArchitectureProtectedEffectProtocol {
                decision_scope: "crates/domain/src".to_owned(),
                witness_owner_scope: "crates/domain/src".to_owned(),
                receipt_scope: "crates/application/src".to_owned(),
                authority_input_types: vec!["domain::ActorAuthority".to_owned()],
                decision_functions: vec!["domain::authorize_publication".to_owned()],
                authorized_output_types: vec!["domain::AuthorizedPublication".to_owned()],
                witness_type: "domain::PublicationAuthority".to_owned(),
                witness_constructors: vec!["domain::PublicationAuthority::from_decision".to_owned()],
                witness_binding_types: vec!["domain::PublicationPlanId".to_owned()],
                plan_type: "domain::PublicationPlan".to_owned(),
                attempt_type: "application::PublicationAttempt".to_owned(),
                observed_outcome_type: "application::PublicationObservation".to_owned(),
                receipt_type: "application::PublicationReceipt".to_owned(),
                receipt_constructors: vec!["application::PublicationReceipt::from_observation".to_owned()],
                failure_receipt_constructors: vec!["application::PublicationReceipt::from_failure".to_owned()],
                success_observation_constructors: vec!["application::PublicationObservation::success".to_owned()],
                failure_observation_constructors: vec!["application::PublicationObservation::failure".to_owned()],
                witness_use,
                borrow_scope: None,
                borrow_rationale: None,
                non_claims: vec![
                    "credential-validity".to_owned(),
                    "external-execution-success".to_owned(),
                ],
            }),
        }
    }

    fn role(scope_kind: ArchitectureScopeKind, scope: &str, role: ArchitectureRole) -> ArchitectureRoleAssignment {
        ArchitectureRoleAssignment {
            scope_kind,
            scope: scope.to_owned(),
            role,
        }
    }

    fn dependency_rule(
        caller_role: ArchitectureRole,
        allowed_callee_roles: &[ArchitectureRole],
    ) -> ArchitectureDependencyRule {
        ArchitectureDependencyRule {
            caller_role,
            allowed_callee_roles: allowed_callee_roles.to_vec(),
            edge_kinds: vec!["normal".to_owned(), "build".to_owned()],
        }
    }
}
