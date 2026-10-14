//! Pure project architecture gate evaluation over supplied Cargo and compiler facts.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use octet_architecture_ir::CapabilityFamily;
use octet_architecture_ir::CollectionStatus;
use octet_architecture_ir::CompilationUnit;
use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CompilerArchitectureReceipt;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CoverageStatus;
use octet_architecture_ir::SourceOrigin;
use octet_architecture_ir::SourceSpan;
use octet_architecture_ir::ValueProvenanceStatus;
use serde::Deserialize;
use serde::Serialize;

use crate::project_architecture_policy::ArchitectureCapabilityFamily;
use crate::project_architecture_policy::ArchitectureEffectAuthority;
use crate::project_architecture_policy::ArchitectureFindingIdentity;
use crate::project_architecture_policy::ArchitectureMode;
use crate::project_architecture_policy::ArchitectureOutboundEffectClassification;
use crate::project_architecture_policy::ArchitectureProtectedEffectProtocol;
use crate::project_architecture_policy::ArchitectureRole;
use crate::project_architecture_policy::ArchitectureScopeKind;
use crate::project_architecture_policy::ArchitectureWitnessUse;
use crate::project_architecture_policy::ProjectArchitecturePolicy;
use crate::project_architecture_policy::validate_architecture_waiver_binding;

/// Project architecture receipt schema emitted by this Octet release.
pub const PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION: &str = "octet-project-architecture-receipt/v1";
/// Repository path for the generated project architecture receipt contract.
pub const PROJECT_ARCHITECTURE_RECEIPT_CONTRACT_PATH: &str =
    "standards/project-architecture/contracts/project-architecture-receipt.generated.ncl";
/// Cargo graph schema consumed by the project architecture gate.
pub const PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION: &str = "octet-project-architecture-cargo-graph/v1";

const RECEIPT_ID_DOMAIN: &str = "octet.project-architecture-gate.receipt/v1";
const FINDING_ID_DOMAIN: &str = "octet.project-architecture-gate.finding/v1";
const CARGO_GRAPH_ID_DOMAIN: &str = "octet.project-architecture-gate.cargo-graph/v1";
const EVIDENCE_TIER: &str = "policy-check";
const CLAIM_ROLE_GRAPH: &str = "role-aware-dependency-graph";
const CLAIM_CORE_PURITY: &str = "domain-core-purity";
const CLAIM_PORT_OWNERSHIP: &str = "application-owned-port";
const CLAIM_ADAPTER_CONSTRUCTION: &str = "composition-root-construction";
const CLAIM_CONTRACT_BOUNDARY: &str = "inward-contract-boundary";
const CLAIM_CAPABILITY: &str = "capability-classification";
const CLAIM_NO_STD: &str = "no-std-obligation";
const CLAIM_COMPILER_COMPLETENESS: &str = "required-compiler-fact";
const CLAIM_INPUT_INTEGRITY: &str = "architecture-input-integrity";
const CLAIM_EFFECT_AUTHORITY_CLASSIFICATION: &str = "outbound-effect-authority-classification";
const CLAIM_AUTHORIZATION_WITNESS: &str = "authorization-witness-opacity";
const CLAIM_AUTHORIZATION_ORIGIN: &str = "authorization-witness-origin";
const CLAIM_PROTECTED_EXECUTOR: &str = "protected-effect-executor";
const CLAIM_EFFECT_RECEIPT: &str = "observed-effect-receipt";
const FORBIDDEN_WITNESS_TRAITS: &[&str] = &[
    "core::marker::Copy",
    "core::clone::Clone",
    "core::default::Default",
    "serde::de::Deserialize",
    "serde::Deserialize",
    "core::convert::From",
    "core::convert::Into",
];
const NON_CLAIMS: &[&str] = &[
    "complete-fcis-proof",
    "complete-hexagonal-proof",
    "complete-dependency-implementation-behavior",
    "runtime-determinism",
    "whole-program-correctness",
    "release-eligibility",
    "signer-authenticity",
    "credential-validity",
    "revocation-freshness",
    "actor-trust",
    "business-policy-correctness",
    "external-execution-success",
    "receipt-authenticity",
];

/// One resolved Cargo dependency edge selected for architecture evaluation.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCargoEdge {
    /// Calling package name.
    pub caller_package: String,
    /// Resolved dependency package name.
    pub callee_package: String,
    /// Cargo edge kind, such as `normal` or `build`.
    pub edge_kind: String,
    /// Target predicate reported by Cargo, when present.
    pub target_predicate: Option<String>,
}

/// Exact resolved Cargo graph supplied by the runner shell.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCargoGraph {
    /// Graph schema.
    pub schema_version: String,
    /// BLAKE3 graph identity.
    pub cargo_graph_id: String,
    /// Exact Cargo.lock byte identity.
    pub cargo_lock_blake3: String,
    /// Canonical locked Cargo metadata identity.
    pub cargo_metadata_blake3: String,
    /// Sorted selected production packages.
    pub selected_packages: Vec<String>,
    /// Exact selected target triple.
    pub target_triple: String,
    /// Sorted selected feature names.
    pub features: Vec<String>,
    /// Whether Cargo default features were selected.
    pub default_features: bool,
    /// Sorted resolved dependency edges.
    pub edges: Vec<ArchitectureCargoEdge>,
}

/// Stable architecture finding status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureFindingStatus {
    /// Current finding has no exact waiver.
    Open,
    /// One exact current waiver applies.
    Waived,
}

/// Stable architecture finding severity.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitectureFindingSeverity {
    /// Input failure blocks every active mode.
    Integration,
    /// Policy violation is retained and mode promotion decides blocking.
    Policy,
}

/// One deterministic architecture policy finding.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitectureFinding {
    /// Stable BLAKE3 finding identity.
    pub finding_id: String,
    /// Stable diagnostic class.
    pub code: String,
    /// Finding severity class.
    pub severity: ArchitectureFindingSeverity,
    /// Waiver result.
    pub status: ArchitectureFindingStatus,
    /// Exact affected claim.
    pub affected_claim: String,
    /// Package involved in the finding, when known.
    pub package_name: Option<String>,
    /// Compilation unit involved in the finding, when known.
    pub unit_id: Option<String>,
    /// Compiler fact involved in the finding, when known.
    pub fact_id: Option<String>,
    /// Workspace source path, when available.
    pub source_path: Option<String>,
    /// Source envelope identity used for exact waiver binding.
    pub source_blake3: String,
    /// Reviewable finding message.
    pub message: String,
}

/// Architecture phase status. This status does not alter lint or closure authority.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchitecturePhaseStatus {
    /// Complete evaluation found no open policy findings.
    Clean,
    /// Complete non-blocking evaluation retained open findings.
    Findings,
    /// Strict policy findings or required unknowns blocked the phase.
    Blocked,
    /// Required evaluation input was incomplete or mismatched.
    IntegrationFailure,
    /// Architecture gate passed but admission still requires separate closure evidence.
    ClosureRequired,
    /// Architecture gate and linked effect closure both passed.
    Admitted,
}

impl ArchitecturePhaseStatus {
    /// Stable machine-readable status name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Findings => "findings",
            Self::Blocked => "blocked",
            Self::IntegrationFailure => "integration-failure",
            Self::ClosureRequired => "closure-required",
            Self::Admitted => "admitted",
        }
    }
}

/// Separate effect-closure parent status supplied to admission evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffectClosureParent {
    /// Effect-closure receipt identity.
    pub receipt_id: String,
    /// True only when the separate closure checker admitted its own claim.
    pub admitted: bool,
    /// Evidence tier retained from the closure checker.
    pub evidence_tier: String,
    /// Closure non-claims retained without promotion.
    pub non_claims: Vec<String>,
}

/// Complete pure architecture-gate input.
pub struct ProjectArchitectureGateInput<'a> {
    /// Activated project policy.
    pub policy: &'a ProjectArchitecturePolicy,
    /// BLAKE3 identity of the Nickel policy source bytes.
    pub policy_source_blake3: &'a str,
    /// BLAKE3 identity of the exported policy bytes.
    pub policy_blake3: &'a str,
    /// Separate lint phase status supplied by the runner.
    pub lint_phase_status: &'a str,
    /// BLAKE3 identity of the normalized command.
    pub command_blake3: &'a str,
    /// Exact resolved Cargo graph.
    pub cargo_graph: &'a ArchitectureCargoGraph,
    /// Complete normalized compiler Architecture IR.
    pub architecture_ir: &'a CompilerArchitectureIr,
    /// Compiler coverage sidecar.
    pub coverage: &'a CompilerArchitectureCoverage,
    /// Compiler collection receipt.
    pub compiler_receipt: &'a CompilerArchitectureReceipt,
    /// Optional separate effect-closure result for admission mode.
    pub effect_closure: Option<&'a EffectClosureParent>,
}

/// Rust-owned architecture evaluation receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectArchitectureReceipt {
    /// Receipt schema.
    pub schema_version: String,
    /// BLAKE3 receipt identity.
    pub receipt_id: String,
    /// Project identity.
    pub project_id: String,
    /// Selected architecture mode.
    pub mode: ArchitectureMode,
    /// Architecture phase status only.
    pub status: ArchitecturePhaseStatus,
    /// Evidence tier for the architecture evaluator.
    pub evidence_tier: String,
    /// Architecture checker identity.
    pub checker_id: String,
    /// Nickel policy source identity.
    pub policy_source_blake3: String,
    /// Exported policy identity.
    pub policy_blake3: String,
    /// Separate lint phase status.
    pub lint_phase_status: String,
    /// Normalized command identity.
    pub command_blake3: String,
    /// Resolved Cargo graph identity.
    pub cargo_graph_id: String,
    /// Compiler Architecture IR identity.
    pub architecture_ir_id: String,
    /// Compiler coverage identity.
    pub coverage_id: String,
    /// Compiler collection receipt identity.
    pub compiler_receipt_id: String,
    /// Exact compiler toolchain identities.
    pub toolchain_blake3s: Vec<String>,
    /// Exact compiler collector identities.
    pub collector_blake3s: Vec<String>,
    /// Exact compiler invocation identities.
    pub invocation_blake3s: Vec<String>,
    /// Exact target triples covered by compiler units.
    pub targets: Vec<String>,
    /// Exact selected feature sets covered by compiler units.
    pub feature_sets: Vec<Vec<String>>,
    /// Evaluated role assignments by package.
    pub package_roles: Vec<(String, ArchitectureRole)>,
    /// Evaluated application-owned port trait names.
    pub port_traits: Vec<String>,
    /// Evaluated capability families.
    pub capability_families: Vec<ArchitectureCapabilityFamily>,
    /// Explicit outbound effect authority classifications.
    pub outbound_effects: Vec<ArchitectureOutboundEffectClassification>,
    /// Compiler fact identities selected for authorized-effect evaluation.
    pub authorized_effect_fact_ids: Vec<String>,
    /// Compiler type identities selected for authorized-effect evaluation.
    pub authorized_effect_type_ids: Vec<String>,
    /// Deterministically ordered findings, including waived findings.
    pub findings: Vec<ProjectArchitectureFinding>,
    /// Compiler unknown identities retained by the gate.
    pub unknown_ids: Vec<String>,
    /// Applied waiver finding identities.
    pub applied_waiver_ids: Vec<String>,
    /// Separate effect-closure parent, when supplied.
    pub effect_closure: Option<EffectClosureParent>,
    /// Explicit architecture non-claims.
    pub non_claims: Vec<String>,
}

/// Render the Nickel contract derived from the Rust receipt DTO.
#[must_use]
pub fn render_project_architecture_receipt_contract() -> String {
    format!(
        r#"# @generated by crates/octet-standards/src/project_architecture_gate.rs; do not edit by hand.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let Blake3 = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value == 64
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Mode = OneOf ["inventory", "advisory", "gate", "admission"] in
let PhaseStatus = OneOf [
  "clean", "findings", "blocked", "integration-failure", "closure-required", "admitted",
] in
let FindingStatus = OneOf ["open", "waived"] in
let FindingSeverity = OneOf ["integration", "policy"] in
let Finding = {{
  finding_id | Blake3,
  code | NonEmptyString,
  severity | FindingSeverity,
  status | FindingStatus,
  affected_claim | NonEmptyString,
  package_name | Dyn,
  unit_id | Dyn,
  fact_id | Dyn,
  source_path | Dyn,
  source_blake3 | Blake3,
  message | NonEmptyString,
}} in
let EffectClosureParent = {{
  receipt_id | NonEmptyString,
  admitted | Bool,
  evidence_tier | NonEmptyString,
  non_claims | Array NonEmptyString,
}} in
{{
  ProjectArchitectureReceipt = {{
    schema_version | OneOf ["{PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION}"],
    receipt_id | Blake3,
    project_id | NonEmptyString,
    mode | Mode,
    status | PhaseStatus,
    evidence_tier | OneOf ["policy-check"],
    checker_id | NonEmptyString,
    policy_source_blake3 | Blake3,
    policy_blake3 | Blake3,
    lint_phase_status | OneOf ["clean", "warning-only", "lint-failure", "integration-failure"],
    command_blake3 | Blake3,
    cargo_graph_id | Blake3,
    architecture_ir_id | Blake3,
    coverage_id | Blake3,
    compiler_receipt_id | Blake3,
    toolchain_blake3s | Array Blake3,
    collector_blake3s | Array Blake3,
    invocation_blake3s | Array Blake3,
    targets | Array NonEmptyString,
    feature_sets | Array (Array NonEmptyString),
    package_roles | Array Dyn,
    port_traits | Array NonEmptyString,
    capability_families | Array NonEmptyString,
    outbound_effects | Array Dyn,
    authorized_effect_fact_ids | Array Blake3,
    authorized_effect_type_ids | Array Blake3,
    findings | Array Finding,
    unknown_ids | Array Blake3,
    applied_waiver_ids | Array Blake3,
    effect_closure | Dyn,
    non_claims | Array NonEmptyString,
  }},
  EffectClosureParent = EffectClosureParent,
}}
"#
    )
}

/// Compute the canonical identity for one Cargo graph.
///
/// # Panics
///
/// Panics only if the fixed, serializable Cargo graph projection cannot be serialized.
#[must_use]
pub fn architecture_cargo_graph_identity(graph: &ArchitectureCargoGraph) -> String {
    octet_architecture_ir::canonical_identity(
        CARGO_GRAPH_ID_DOMAIN,
        &(
            &graph.schema_version,
            &graph.cargo_lock_blake3,
            &graph.cargo_metadata_blake3,
            &graph.selected_packages,
            &graph.target_triple,
            &graph.features,
            graph.default_features,
            &graph.edges,
        ),
    )
    .expect("architecture Cargo graph identity fields must serialize")
}

/// Compute and apply project architecture policy over supplied facts.
// r[impl octet.project_architecture_gate.role_graph]
// r[impl octet.project_architecture_gate.fcis]
// r[impl octet.project_architecture_gate.modes]
// r[impl octet.project_architecture_gate.waivers]
// r[impl octet.project_architecture_gate.admission]
// r[impl octet.project_architecture_gate.evidence]
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn evaluate_project_architecture(input: &ProjectArchitectureGateInput<'_>) -> ProjectArchitectureReceipt {
    let mut findings = evaluate_findings(input);
    findings.sort();
    findings.dedup();
    apply_waivers(input, &mut findings);
    findings.sort();

    let integration_failure = findings.iter().any(|finding| {
        finding.severity == ArchitectureFindingSeverity::Integration
            && finding.status == ArchitectureFindingStatus::Open
    });
    let open_policy_findings = findings.iter().any(|finding| {
        finding.severity == ArchitectureFindingSeverity::Policy && finding.status == ArchitectureFindingStatus::Open
    });
    let status = classify_status(input.policy.mode, integration_failure, open_policy_findings, input.effect_closure);
    let targets = sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.target_triple.clone()).collect());
    let feature_sets =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| sorted_dedup(unit.features.clone())).collect());
    let package_roles = policy_package_roles(input.policy).into_iter().collect::<Vec<_>>();
    let port_traits = sorted_dedup(input.policy.ports.iter().map(|port| port.port_trait.clone()).collect());
    let capability_families = sorted_dedup(
        input.policy.capability_classifications.iter().map(|classification| classification.family).collect(),
    );
    let mut outbound_effects = input.policy.outbound_effects.clone();
    outbound_effects.sort_by_key(|classification| classification.family);
    let protected_definitions = protected_effect_definitions(input.policy);
    let authorized_effect_type_ids = sorted_dedup(
        input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id,
                    definition_path: Some(definition),
                    ..
                } if protected_definitions.types.contains(definition) => Some(type_id.clone()),
                _ => None,
            })
            .collect(),
    );
    let protected_type_ids = authorized_effect_type_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let authorized_effect_fact_ids = sorted_dedup(
        input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::TypeShape { fact_id, type_id, .. }
                | CompilerArchitectureFact::FunctionSafety {
                    fact_id,
                    output_type_id: type_id,
                    ..
                }
                | CompilerArchitectureFact::CallArgumentProvenance {
                    fact_id,
                    argument_type_id: type_id,
                    ..
                } if protected_type_ids.contains(type_id.as_str()) => Some(fact_id.clone()),
                _ => None,
            })
            .collect(),
    );
    let toolchain_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.toolchain_blake3.clone()).collect());
    let collector_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.collector_blake3.clone()).collect());
    let invocation_blake3s =
        sorted_dedup(input.architecture_ir.units.iter().map(|unit| unit.invocation_blake3.clone()).collect());
    let unknown_ids =
        sorted_dedup(input.architecture_ir.unknowns.iter().map(|unknown| unknown.unknown_id.clone()).collect());
    let applied_waiver_ids = findings
        .iter()
        .filter(|finding| finding.status == ArchitectureFindingStatus::Waived)
        .map(|finding| finding.finding_id.clone())
        .collect::<Vec<_>>();
    let mut non_claims = input.policy.non_claims.clone();
    non_claims.extend(NON_CLAIMS.iter().map(|claim| (*claim).to_owned()));
    let non_claims = sorted_dedup(non_claims);

    let mut receipt = ProjectArchitectureReceipt {
        schema_version: PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
        receipt_id: String::new(),
        project_id: input.policy.project_id.clone(),
        mode: input.policy.mode,
        status,
        evidence_tier: EVIDENCE_TIER.to_owned(),
        checker_id: "octet-standards.project-architecture-gate".to_owned(),
        policy_source_blake3: input.policy_source_blake3.to_owned(),
        policy_blake3: input.policy_blake3.to_owned(),
        lint_phase_status: input.lint_phase_status.to_owned(),
        command_blake3: input.command_blake3.to_owned(),
        cargo_graph_id: input.cargo_graph.cargo_graph_id.clone(),
        architecture_ir_id: input.architecture_ir.architecture_ir_id.clone(),
        coverage_id: input.coverage.coverage_id.clone(),
        compiler_receipt_id: input.compiler_receipt.receipt_id.clone(),
        toolchain_blake3s,
        collector_blake3s,
        invocation_blake3s,
        targets,
        feature_sets,
        package_roles,
        port_traits,
        capability_families,
        outbound_effects,
        authorized_effect_fact_ids,
        authorized_effect_type_ids,
        findings,
        unknown_ids,
        applied_waiver_ids,
        effect_closure: input.effect_closure.cloned(),
        non_claims,
    };
    receipt.receipt_id = receipt_identity(&receipt);
    receipt
}

/// Verify the stored receipt identity and required claim boundaries.
#[must_use]
pub fn verify_project_architecture_receipt(receipt: &ProjectArchitectureReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != PROJECT_ARCHITECTURE_RECEIPT_SCHEMA_VERSION {
        diagnostics.push("unsupported project architecture receipt schema".to_owned());
    }
    if receipt.receipt_id != receipt_identity(receipt) {
        diagnostics.push("project architecture receipt identity is stale or malformed".to_owned());
    }
    for required in NON_CLAIMS {
        if !receipt.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("project architecture receipt is missing non-claim `{required}`"));
        }
    }
    if !receipt.outbound_effects.is_empty() {
        for required in [
            "credential-validity",
            "revocation-freshness",
            "actor-trust",
            "business-policy-correctness",
            "external-execution-success",
            "receipt-authenticity",
        ] {
            if !receipt.non_claims.iter().any(|claim| claim == required) {
                diagnostics
                    .push(format!("project architecture receipt is missing authorized-effect non-claim `{required}`"));
            }
        }
        if receipt.authorized_effect_fact_ids.is_empty() || receipt.authorized_effect_type_ids.is_empty() {
            diagnostics.push("project architecture receipt lacks authorized-effect fact or type identities".to_owned());
        }
    }
    if receipt.evidence_tier != EVIDENCE_TIER {
        diagnostics.push("project architecture receipt evidence tier is not policy-check".to_owned());
    }
    if !matches!(
        receipt.lint_phase_status.as_str(),
        "clean" | "warning-only" | "lint-failure" | "integration-failure"
    ) {
        diagnostics.push("project architecture receipt has an unknown lint phase status".to_owned());
    }
    if let Some(closure) = &receipt.effect_closure
        && (closure.receipt_id.trim().is_empty()
            || closure.evidence_tier.trim().is_empty()
            || closure.non_claims.is_empty())
    {
        diagnostics.push("project architecture receipt has an incomplete effect-closure parent".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn evaluate_findings(input: &ProjectArchitectureGateInput<'_>) -> Vec<ProjectArchitectureFinding> {
    let mut findings = input_integrity_findings(input);
    if findings.is_empty() {
        let indexes = GateIndexes::new(input.architecture_ir);
        evaluate_role_graph(input, &indexes, &mut findings);
        evaluate_core_effects(input, &indexes, &mut findings);
        evaluate_authorized_effects(input, &indexes, &mut findings);
        evaluate_ports(input, &indexes, &mut findings);
        evaluate_construction(input, &indexes, &mut findings);
        evaluate_contract_types(input, &indexes, &mut findings);
        evaluate_no_std(input, &indexes, &mut findings);
        evaluate_unknowns(input, &indexes, &mut findings);
    }
    findings
}

fn sorted_dedup<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort();
    values.dedup();
    values
}

fn input_integrity_findings(input: &ProjectArchitectureGateInput<'_>) -> Vec<ProjectArchitectureFinding> {
    let mut findings = Vec::new();
    let source_identity =
        input.architecture_ir.units.first().map_or(input.policy_blake3, |unit| unit.source_blake3.as_str());
    validate_cargo_inputs(input, source_identity, &mut findings);
    validate_compiler_inputs(input, source_identity, &mut findings);
    validate_unit_inputs(input, source_identity, &mut findings);
    findings
}

fn validate_cargo_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    require_input(
        findings,
        source_identity,
        input.cargo_graph.schema_version == PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION,
        "cargo-graph-schema",
        "resolved Cargo graph schema is unsupported",
    );
    require_input(
        findings,
        source_identity,
        input.cargo_graph.cargo_graph_id == architecture_cargo_graph_identity(input.cargo_graph),
        "cargo-graph-identity",
        "resolved Cargo graph identity is stale or malformed",
    );
    require_input(
        findings,
        source_identity,
        !input.cargo_graph.cargo_lock_blake3.trim().is_empty()
            && !input.cargo_graph.cargo_metadata_blake3.trim().is_empty(),
        "cargo-input-identities",
        "resolved Cargo graph lacks lockfile or metadata identity",
    );
    let expected_packages = input.policy.production.packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let selected_packages = input.cargo_graph.selected_packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    require_input(
        findings,
        source_identity,
        selected_packages == expected_packages,
        "cargo-package-selection",
        "resolved Cargo graph package selection does not match production policy",
    );
    let selected_targets = input
        .policy
        .production
        .targets
        .iter()
        .filter(|target| {
            target.triple == input.cargo_graph.target_triple
                && target.features == input.cargo_graph.features
                && target.default_features == input.cargo_graph.default_features
        })
        .count();
    require_input(
        findings,
        source_identity,
        selected_targets == 1,
        "cargo-target-selection",
        "resolved Cargo graph target or features do not match exactly one policy target",
    );
}

fn validate_compiler_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let checks = [
        (
            !input.policy_source_blake3.trim().is_empty(),
            "policy-source-identity",
            "Nickel policy source identity is empty",
        ),
        (
            !input.lint_phase_status.trim().is_empty(),
            "lint-phase-status",
            "separate lint phase status is empty",
        ),
        (
            input.architecture_ir.policy_blake3 == input.policy_blake3,
            "policy-ir-mismatch",
            "Architecture IR policy identity does not match the activated policy",
        ),
        (
            input.coverage.status == CoverageStatus::Complete,
            "compiler-coverage-incomplete",
            "compiler Architecture IR coverage is incomplete",
        ),
        (
            input.compiler_receipt.status == CollectionStatus::Complete,
            "compiler-receipt-incomplete",
            "compiler Architecture IR receipt is incomplete",
        ),
        (
            input.compiler_receipt.policy_blake3 == input.policy_blake3,
            "compiler-policy-mismatch",
            "compiler receipt policy identity does not match the activated policy",
        ),
        (
            input.compiler_receipt.coverage_id == input.coverage.coverage_id,
            "compiler-coverage-mismatch",
            "compiler receipt coverage identity does not match the coverage artifact",
        ),
        (
            input.compiler_receipt.architecture_ir_id.as_deref()
                == Some(input.architecture_ir.architecture_ir_id.as_str()),
            "compiler-ir-mismatch",
            "compiler receipt Architecture IR identity does not match the IR artifact",
        ),
        (!input.command_blake3.trim().is_empty(), "command-identity", "normalized command identity is empty"),
    ];
    for (condition, code, message) in checks {
        require_input(findings, source_identity, condition, code, message);
    }
}

fn validate_unit_inputs(
    input: &ProjectArchitectureGateInput<'_>,
    source_identity: &str,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let expected_packages = input.policy.production.packages.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let unit_packages =
        input.architecture_ir.units.iter().map(|unit| unit.package_name.as_str()).collect::<BTreeSet<_>>();
    require_input(
        findings,
        source_identity,
        unit_packages == expected_packages,
        "production-unit-coverage",
        "compiler units do not exactly cover declared production packages",
    );
    let policy_roles = policy_package_roles(input.policy);
    for unit in &input.architecture_ir.units {
        let role_matches = policy_roles.get(&unit.package_name).is_some_and(|role| compiler_role(*role) == unit.role);
        require_input(
            findings,
            source_identity,
            role_matches,
            "unit-role-mismatch",
            &format!("compiler unit `{}` role does not match explicit package policy", unit.package_name),
        );
    }
}

fn require_input(
    findings: &mut Vec<ProjectArchitectureFinding>,
    source_identity: &str,
    condition: bool,
    code: &str,
    message: &str,
) {
    if !condition {
        findings.push(new_finding(
            code,
            ArchitectureFindingSeverity::Integration,
            CLAIM_INPUT_INTEGRITY,
            None,
            None,
            None,
            None,
            source_identity,
            message,
        ));
    }
}

fn evaluate_role_graph(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let roles = policy_package_roles(input.policy);
    if !roles.values().any(|role| *role == ArchitectureRole::CompositionRoot) {
        findings.push(new_finding(
            "missing-composition-root",
            ArchitectureFindingSeverity::Policy,
            CLAIM_ROLE_GRAPH,
            None,
            None,
            None,
            None,
            input.policy_blake3,
            "production architecture has no explicit composition-root package",
        ));
    }
    for edge in &input.cargo_graph.edges {
        let Some(caller_role) = roles.get(&edge.caller_package) else {
            findings.push(new_finding(
                "unknown-cargo-caller",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                Some(&edge.caller_package),
                None,
                None,
                None,
                input.policy_blake3,
                &format!("Cargo graph caller `{}` has no production role", edge.caller_package),
            ));
            continue;
        };
        let allowed = if let Some(callee_role) = roles.get(&edge.callee_package) {
            input.policy.dependency_rules.iter().any(|rule| {
                rule.caller_role == *caller_role
                    && rule.edge_kinds.iter().any(|kind| kind == &edge.edge_kind)
                    && rule.allowed_callee_roles.iter().any(|role| role == callee_role)
            })
        } else {
            input.policy.provider_classifications.iter().any(|provider| {
                provider.package == edge.callee_package
                    && provider.permitted_caller_roles.iter().any(|role| role == caller_role)
                    && provider.edge_kinds.iter().any(|kind| kind == &edge.edge_kind)
            })
        };
        if !allowed {
            let unit = indexes.unit_for_package(&edge.caller_package);
            let callee = roles.get(&edge.callee_package).map_or_else(
                || format!("unclassified external provider `{}`", edge.callee_package),
                |callee_role| format!("package `{}` with role `{:?}`", edge.callee_package, callee_role),
            );
            findings.push(finding_for_unit(
                "forbidden-role-edge",
                CLAIM_ROLE_GRAPH,
                unit,
                None,
                None,
                &format!(
                    "package `{}` with role `{:?}` has forbidden {} dependency on {callee}",
                    edge.caller_package, caller_role, edge.edge_kind
                ),
            ));
        }
    }
}

fn evaluate_core_effects(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let classifications = input
        .policy
        .capability_classifications
        .iter()
        .map(|classification| (classification.family, classification))
        .collect::<BTreeMap<_, _>>();
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Effect {
            fact_id,
            capability_family,
            primitive_definition,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        let Some(unit) = indexes.unit_for_fact(fact_id) else {
            continue;
        };
        let source = indexes.source_path(span_id);
        let policy_family = policy_capability(*capability_family);
        if unit.role == CompilerArchitectureRole::DomainCore {
            findings.push(finding_for_unit(
                "domain-core-effect",
                CLAIM_CORE_PURITY,
                Some(unit),
                Some(fact_id),
                source,
                &format!("domain-core compiler fact performs `{capability_family:?}` through `{primitive_definition}`"),
            ));
        } else if let Some(policy_family) = policy_family {
            let executor_matches = classifications
                .get(&policy_family)
                .is_some_and(|classification| compiler_role(classification.executor_role) == unit.role);
            if !executor_matches {
                findings.push(finding_for_unit(
                    "capability-executor-role",
                    CLAIM_CAPABILITY,
                    Some(unit),
                    Some(fact_id),
                    source,
                    &format!(
                        "capability `{capability_family:?}` executes in package `{}` outside its declared executor role",
                        unit.package_name
                    ),
                ));
            }
        } else {
            findings.push(finding_for_unit(
                "unclassified-capability",
                CLAIM_CAPABILITY,
                Some(unit),
                Some(fact_id),
                source,
                &format!("compiler effect capability `{capability_family:?}` has no project classification"),
            ));
        }
    }
}

#[derive(Default)]
struct ProtectedEffectDefinitions {
    types: BTreeSet<String>,
}

fn protected_effect_definitions(policy: &ProjectArchitecturePolicy) -> ProtectedEffectDefinitions {
    let mut definitions = ProtectedEffectDefinitions::default();
    for protocol in policy.outbound_effects.iter().filter_map(|classification| classification.protected.as_ref()) {
        definitions.types.extend(protocol.authority_input_types.iter().cloned());
        definitions.types.extend(protocol.authorized_output_types.iter().cloned());
        definitions.types.extend(protocol.witness_binding_types.iter().cloned());
        definitions.types.extend([
            protocol.witness_type.clone(),
            protocol.plan_type.clone(),
            protocol.attempt_type.clone(),
            protocol.observed_outcome_type.clone(),
            protocol.receipt_type.clone(),
        ]);
    }
    definitions
}

fn evaluate_authorized_effects(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let classifications = input
        .policy
        .outbound_effects
        .iter()
        .map(|classification| (classification.family, classification))
        .collect::<BTreeMap<_, _>>();
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Effect {
            fact_id,
            capability_family,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        let Some(unit) = indexes.unit_for_fact(fact_id) else {
            continue;
        };
        if unit.role != CompilerArchitectureRole::OutboundAdapter {
            continue;
        }
        let Some(family) = policy_capability(*capability_family) else {
            continue;
        };
        if !classifications.contains_key(&family) {
            findings.push(finding_for_unit(
                "unclassified-effect-authority",
                CLAIM_EFFECT_AUTHORITY_CLASSIFICATION,
                Some(unit),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("outbound effect capability `{family:?}` has no explicit authority classification"),
            ));
        }
    }
    for classification in classifications.values() {
        if classification.authority != ArchitectureEffectAuthority::Protected {
            continue;
        }
        let Some(protocol) = &classification.protected else {
            findings.push(new_finding(
                "missing-protected-effect-protocol",
                ArchitectureFindingSeverity::Policy,
                CLAIM_EFFECT_AUTHORITY_CLASSIFICATION,
                None,
                None,
                None,
                Some(classification.executor_scope.clone()),
                input.policy_blake3,
                &format!("protected effect capability `{:?}` has no protocol", classification.family),
            ));
            continue;
        };
        evaluate_protected_witness(input, indexes, classification, protocol, findings);
        evaluate_protected_calls(input, indexes, classification, protocol, findings);
    }
}

#[allow(clippy::too_many_lines)]
fn evaluate_protected_witness(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    classification: &ArchitectureOutboundEffectClassification,
    protocol: &ArchitectureProtectedEffectProtocol,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let shapes = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::TypeShape {
                fact_id,
                definition_path,
                field_visibilities,
                field_type_definitions,
                implemented_traits,
                span_id,
                ..
            } if definition_path == &protocol.witness_type => {
                Some((fact_id, field_visibilities, field_type_definitions, implemented_traits, span_id))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    if shapes.len() == 1 {
        let (fact_id, fields, field_types, traits, span_id) = shapes[0];
        let public_field = fields.iter().any(|visibility| visibility == "public");
        let owner_prefix = format!("restricted:{}", witness_owner_module(&protocol.witness_type));
        let escaped_field = fields.iter().any(|visibility| visibility != &owner_prefix);
        if public_field || escaped_field {
            findings.push(finding_for_unit(
                "witness-field-visibility",
                CLAIM_AUTHORIZATION_WITNESS,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("witness type `{}` has fields visible outside its owning module", protocol.witness_type),
            ));
        }
        for required in &protocol.witness_binding_types {
            if !field_types.iter().any(|field_type| field_type == required) {
                findings.push(finding_for_unit(
                    "witness-binding-type-missing",
                    CLAIM_AUTHORIZATION_WITNESS,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!(
                        "witness type `{}` does not carry required binding type `{required}`",
                        protocol.witness_type
                    ),
                ));
            }
        }
        for forbidden in FORBIDDEN_WITNESS_TRAITS {
            if traits.iter().any(|current| trait_matches(current, forbidden)) {
                findings.push(finding_for_unit(
                    "witness-forbidden-trait",
                    CLAIM_AUTHORIZATION_WITNESS,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!("witness type `{}` implements forbidden trait `{forbidden}`", protocol.witness_type),
                ));
            }
        }
        let broad_conversion = traits.iter().any(|current| {
            (current.contains("From<")
                || current.contains("Into<")
                || trait_matches(current, "core::convert::From")
                || trait_matches(current, "core::convert::Into"))
                && !protocol.authorized_output_types.iter().any(|allowed| current.contains(allowed))
        });
        if broad_conversion {
            findings.push(finding_for_unit(
                "witness-broad-conversion",
                CLAIM_AUTHORIZATION_WITNESS,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("witness type `{}` has an ambient conversion path", protocol.witness_type),
            ));
        }
        if protocol.witness_use == ArchitectureWitnessUse::Consume
            && traits.iter().any(|current| trait_matches(current, "core::marker::Copy"))
        {
            findings.push(finding_for_unit(
                "witness-replayable",
                CLAIM_AUTHORIZATION_WITNESS,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("consume witness `{}` is compiler-copyable", protocol.witness_type),
            ));
        }
    } else {
        findings.push(new_finding(
            "witness-shape-resolution",
            ArchitectureFindingSeverity::Policy,
            CLAIM_AUTHORIZATION_WITNESS,
            None,
            None,
            None,
            Some(protocol.witness_owner_scope.clone()),
            input.policy_blake3,
            &format!("witness type `{}` must resolve to exactly one compiler type-shape fact", protocol.witness_type),
        ));
    }
    let item_visibility = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id, visibility, ..
            } => Some((item_id.as_str(), visibility.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let safety_facts = input.architecture_ir.facts.iter().filter_map(|fact| match fact {
        CompilerArchitectureFact::FunctionSafety {
            fact_id,
            qualified_path,
            output_type_id,
            is_unsafe,
            span_id,
            ..
        } => Some((fact_id, qualified_path, output_type_id, *is_unsafe, span_id)),
        _ => None,
    });
    let type_definitions = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                definition_path: Some(definition),
                ..
            } => Some((type_id.as_str(), definition.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for (fact_id, qualified_path, output_type_id, is_unsafe, span_id) in safety_facts {
        if type_definitions.get(output_type_id.as_str()) != Some(&protocol.witness_type.as_str()) {
            continue;
        }
        let declared = protocol.witness_constructors.iter().any(|constructor| constructor == qualified_path);
        let constructor_restricted = input.architecture_ir.facts.iter().any(|fact| match fact {
            CompilerArchitectureFact::FunctionSafety {
                fact_id: current,
                item_id,
                ..
            } if current == fact_id => {
                item_visibility.get(item_id.as_str()).is_some_and(|visibility| *visibility != "public")
            }
            _ => false,
        });
        if is_unsafe || !declared || !constructor_restricted {
            findings.push(finding_for_unit(
                if is_unsafe {
                    "unsafe-witness-constructor"
                } else if !declared {
                    "undeclared-witness-constructor"
                } else {
                    "public-witness-constructor"
                },
                CLAIM_AUTHORIZATION_WITNESS,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!(
                    "function `{qualified_path}` constructs witness `{}` outside its approved safe constructor set",
                    protocol.witness_type
                ),
            ));
        }
    }
    let _ = classification;
}

#[allow(clippy::too_many_lines)]
fn evaluate_protected_calls(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    classification: &ArchitectureOutboundEffectClassification,
    protocol: &ArchitectureProtectedEffectProtocol,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let type_definitions = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                definition_path: Some(definition),
                ..
            } => Some((type_id.as_str(), definition.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let item_paths = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                ..
            } => Some((item_id.as_str(), qualified_path.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let signatures = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Signature {
                fact_id,
                owner_item_id,
                input_type_ids,
                output_type_id,
                ..
            } => Some((fact_id, owner_item_id, input_type_ids, output_type_id)),
            _ => None,
        })
        .collect::<Vec<_>>();
    for decision in &protocol.decision_functions {
        let decision_items = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    fact_id,
                    item_id,
                    qualified_path,
                    span_id,
                    ..
                } if qualified_path == decision => Some((fact_id, item_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if decision_items.len() != 1 {
            findings.push(new_finding(
                "authority-decision-resolution",
                ArchitectureFindingSeverity::Policy,
                CLAIM_AUTHORIZATION_ORIGIN,
                None,
                None,
                None,
                Some(protocol.decision_scope.clone()),
                input.policy_blake3,
                &format!("authority decision `{decision}` must resolve to exactly one compiler item"),
            ));
            continue;
        }
        let (fact_id, item_id, span_id) = decision_items[0];
        let source = indexes.source_path(span_id);
        let unit = indexes.unit_for_fact(fact_id);
        let owned = unit.is_some_and(|current| current.role == CompilerArchitectureRole::DomainCore)
            && source.is_some_and(|path| path_in_scope(path, &protocol.decision_scope));
        if !owned {
            findings.push(finding_for_unit(
                "authority-decision-scope",
                CLAIM_AUTHORIZATION_ORIGIN,
                unit,
                Some(fact_id),
                source,
                &format!("authority decision `{decision}` is outside its declared domain-core scope"),
            ));
        }
        let signature_matches = signatures.iter().find(|(_, owner_item_id, _, _)| *owner_item_id == item_id);
        let output_allowed = signature_matches.is_some_and(|(_, _, _, output_type_id)| {
            type_definitions
                .get(output_type_id.as_str())
                .is_some_and(|definition| protocol.authorized_output_types.iter().any(|allowed| allowed == definition))
        });
        let inputs_complete = signature_matches.is_some_and(|(_, _, input_type_ids, _)| {
            protocol.authority_input_types.iter().all(|required| {
                input_type_ids.iter().any(|type_id| {
                    type_definitions.get(type_id.as_str()).is_some_and(|definition| *definition == required)
                })
            })
        });
        if !output_allowed {
            findings.push(finding_for_unit(
                "authority-decision-output",
                CLAIM_AUTHORIZATION_ORIGIN,
                unit,
                Some(fact_id),
                source,
                &format!("authority decision `{decision}` does not return a declared authorized output type"),
            ));
        }
        if !inputs_complete {
            findings.push(finding_for_unit(
                "authority-decision-input",
                CLAIM_AUTHORIZATION_ORIGIN,
                unit,
                Some(fact_id),
                source,
                &format!("authority decision `{decision}` does not receive every declared authority input type"),
            ));
        }
    }
    for constructor in &protocol.witness_constructors {
        let constructor_items = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    fact_id,
                    item_id,
                    qualified_path,
                    span_id,
                    ..
                } if qualified_path == constructor => Some((fact_id, item_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if constructor_items.len() != 1 {
            findings.push(new_finding(
                "witness-constructor-resolution",
                ArchitectureFindingSeverity::Policy,
                CLAIM_AUTHORIZATION_WITNESS,
                None,
                None,
                None,
                Some(protocol.witness_owner_scope.clone()),
                input.policy_blake3,
                &format!("witness constructor `{constructor}` must resolve to exactly one compiler item"),
            ));
            continue;
        }
        let (fact_id, item_id, span_id) = constructor_items[0];
        let signature_matches = signatures.iter().find(|(_, owner_item_id, _, _)| *owner_item_id == item_id);
        let input_allowed = signature_matches.is_some_and(|(_, _, input_type_ids, _)| {
            input_type_ids.iter().any(|type_id| {
                type_definitions.get(type_id.as_str()).is_some_and(|definition| {
                    protocol.authorized_output_types.iter().any(|allowed| allowed == definition)
                })
            })
        });
        let output_matches = signature_matches.is_some_and(|(_, _, _, output_type_id)| {
            type_definitions.get(output_type_id.as_str()) == Some(&protocol.witness_type.as_str())
        });
        if !input_allowed || !output_matches {
            findings.push(finding_for_unit(
                "witness-constructor-signature",
                CLAIM_AUTHORIZATION_WITNESS,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("witness constructor `{constructor}` must transform a declared authorized output into the witness type"),
            ));
        }
    }
    let provenance = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::CallArgumentProvenance {
                fact_id,
                caller_item_id,
                callee_definition,
                argument_type_id,
                origin_definitions,
                status,
                span_id,
                ..
            } => Some((
                fact_id,
                caller_item_id,
                callee_definition,
                argument_type_id,
                origin_definitions,
                *status,
                span_id,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    for executor in &classification.executor_methods {
        let executor_items = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    fact_id,
                    item_id,
                    qualified_path,
                    span_id,
                    ..
                } if qualified_path == executor => Some((fact_id, item_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if executor_items.len() != 1 {
            findings.push(new_finding(
                "protected-executor-resolution",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PROTECTED_EXECUTOR,
                None,
                None,
                None,
                Some(classification.executor_scope.clone()),
                input.policy_blake3,
                &format!("protected executor `{executor}` must resolve to exactly one compiler item"),
            ));
        }
        for (item_fact_id, item_id, span_id) in &executor_items {
            let source = indexes.source_path(span_id);
            let unit = indexes.unit_for_fact(item_fact_id);
            let owned = unit.is_some_and(|current| current.role == CompilerArchitectureRole::OutboundAdapter)
                && source.is_some_and(|path| path_in_scope(path, &classification.executor_scope));
            if !owned {
                findings.push(finding_for_unit(
                    "protected-executor-scope",
                    CLAIM_PROTECTED_EXECUTOR,
                    unit,
                    Some(item_fact_id),
                    source,
                    &format!("protected executor `{executor}` is outside its declared outbound-adapter scope"),
                ));
            }
            let has_witness_input = signatures.iter().any(|(_, owner_item_id, input_type_ids, _)| {
                *owner_item_id == *item_id
                    && input_type_ids.iter().any(|type_id| {
                        type_definitions
                            .get(type_id.as_str())
                            .is_some_and(|definition| *definition == protocol.witness_type)
                    })
            });
            if !has_witness_input {
                findings.push(finding_for_unit(
                    "protected-executor-witness-parameter",
                    CLAIM_PROTECTED_EXECUTOR,
                    unit,
                    Some(item_fact_id),
                    source,
                    &format!("protected executor `{executor}` lacks its declared witness parameter"),
                ));
            }
        }
        let calls = provenance
            .iter()
            .filter(|(_, _, callee, type_id, _, _, _)| {
                *callee == executor
                    && type_definitions
                        .get(type_id.as_str())
                        .is_some_and(|definition| *definition == protocol.witness_type)
            })
            .collect::<Vec<_>>();
        if calls.is_empty() {
            findings.push(new_finding(
                "protected-executor-provenance-missing",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PROTECTED_EXECUTOR,
                None,
                None,
                None,
                Some(classification.executor_scope.clone()),
                input.policy_blake3,
                &format!("protected executor `{executor}` has no selected witness provenance fact"),
            ));
        }
        for (fact_id, _, _, _, origins, status, span_id) in calls {
            if protocol.witness_use == ArchitectureWitnessUse::Borrow
                && !protocol
                    .borrow_scope
                    .as_ref()
                    .is_some_and(|scope| indexes.source_path(span_id).is_some_and(|path| path_in_scope(path, scope)))
            {
                findings.push(finding_for_unit(
                    "borrowed-witness-scope",
                    CLAIM_PROTECTED_EXECUTOR,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!("borrowed witness for executor `{executor}` escapes its reusable scope"),
                ));
            }
            if *status != ValueProvenanceStatus::Exact || origins.len() != 1 {
                findings.push(finding_for_unit(
                    "unknown-witness-origin",
                    CLAIM_AUTHORIZATION_ORIGIN,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!("protected executor `{executor}` does not have one exact witness origin"),
                ));
            } else if !protocol.decision_functions.iter().any(|decision| decision == &origins[0]) {
                findings.push(finding_for_unit(
                    "unauthorized-witness-origin",
                    CLAIM_AUTHORIZATION_ORIGIN,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!(
                        "protected executor `{executor}` receives witness from unapproved origin `{}`",
                        origins[0]
                    ),
                ));
            }
        }
    }
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Effect {
            fact_id,
            capability_family,
            caller_item_id,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        if policy_capability(*capability_family) != Some(classification.family) {
            continue;
        }
        let caller = item_paths.get(caller_item_id.as_str()).copied();
        if !caller.is_some_and(|path| classification.executor_methods.iter().any(|executor| executor == path)) {
            findings.push(finding_for_unit(
                "unguarded-protected-effect",
                CLAIM_PROTECTED_EXECUTOR,
                indexes.unit_for_fact(fact_id),
                Some(fact_id),
                indexes.source_path(span_id),
                &format!(
                    "protected effect `{:?}` executes outside its declared executor methods",
                    classification.family
                ),
            ));
        }
    }
    evaluate_receipt_constructors(
        input,
        indexes,
        protocol,
        &type_definitions,
        &item_paths,
        &provenance,
        &protocol.receipt_constructors,
        &protocol.success_observation_constructors,
        true,
        findings,
    );
    evaluate_receipt_constructors(
        input,
        indexes,
        protocol,
        &type_definitions,
        &item_paths,
        &provenance,
        &protocol.failure_receipt_constructors,
        &protocol.failure_observation_constructors,
        false,
        findings,
    );
}

#[allow(clippy::too_many_arguments)]
fn evaluate_receipt_constructors(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    protocol: &ArchitectureProtectedEffectProtocol,
    type_definitions: &BTreeMap<&str, &str>,
    item_paths: &BTreeMap<&str, &str>,
    provenance: &[ProvenanceFactRef<'_>],
    receipt_constructors: &[String],
    permitted_observation_origins: &[String],
    success_receipt: bool,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    for receipt_constructor in receipt_constructors {
        let calls = provenance
            .iter()
            .filter(|(_, _, callee, _, _, _, _)| *callee == receipt_constructor)
            .collect::<Vec<_>>();
        if calls.is_empty() {
            findings.push(new_finding(
                "receipt-observation-provenance-missing",
                ArchitectureFindingSeverity::Policy,
                CLAIM_EFFECT_RECEIPT,
                None,
                None,
                None,
                Some(protocol.receipt_scope.clone()),
                input.policy_blake3,
                &format!("receipt constructor `{receipt_constructor}` has no selected observation provenance fact"),
            ));
        }
        for (fact_id, caller_item_id, _, argument_type_id, origins, status, span_id) in calls {
            let observed_argument = type_definitions
                .get(argument_type_id.as_str())
                .is_some_and(|definition| *definition == protocol.observed_outcome_type);
            let approved_observation = origins
                .iter()
                .all(|origin| permitted_observation_origins.iter().any(|constructor| constructor == origin));
            let opposite_origin = origins.iter().any(|origin| {
                if success_receipt {
                    protocol.failure_observation_constructors.iter().any(|constructor| constructor == origin)
                } else {
                    protocol.success_observation_constructors.iter().any(|constructor| constructor == origin)
                }
            });
            if !observed_argument {
                findings.push(finding_for_unit(
                    "planned-effect-promoted-to-receipt",
                    CLAIM_EFFECT_RECEIPT,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!("receipt constructor `{receipt_constructor}` receives a non-observation protocol type"),
                ));
            } else if *status != ValueProvenanceStatus::Exact || !approved_observation || opposite_origin {
                findings.push(finding_for_unit(
                    if success_receipt && opposite_origin {
                        "failure-promoted-to-success-receipt"
                    } else if !success_receipt && opposite_origin {
                        "success-promoted-to-failure-receipt"
                    } else {
                        "unknown-receipt-observation"
                    },
                    CLAIM_EFFECT_RECEIPT,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    &format!("receipt constructor `{receipt_constructor}` lacks one exact approved observation for its receipt class"),
                ));
            }
            let caller_path = item_paths.get(caller_item_id.as_str()).copied().unwrap_or("unknown-caller");
            if caller_path == "unknown-caller" {
                findings.push(finding_for_unit(
                    "receipt-caller-resolution",
                    CLAIM_EFFECT_RECEIPT,
                    indexes.unit_for_fact(fact_id),
                    Some(fact_id),
                    indexes.source_path(span_id),
                    "receipt constructor caller could not be resolved to a compiler item",
                ));
            }
        }
    }
}

type ProvenanceFactRef<'a> =
    (&'a String, &'a String, &'a String, &'a String, &'a Vec<String>, ValueProvenanceStatus, &'a String);

fn witness_owner_module(witness_type: &str) -> &str {
    witness_type.rsplit_once("::").map_or(witness_type, |(module, _)| module)
}

fn trait_matches(actual: &str, expected: &str) -> bool {
    actual == expected || actual.rsplit_once("::").is_some_and(|(_, tail)| expected.ends_with(tail))
}

fn evaluate_ports(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    for port in &input.policy.ports {
        let traits = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Trait {
                    fact_id,
                    qualified_path,
                    span_id,
                    ..
                } if qualified_path == &port.port_trait => Some((fact_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if traits.len() != 1 {
            findings.push(new_finding(
                "port-trait-resolution",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PORT_OWNERSHIP,
                None,
                None,
                None,
                None,
                input.policy_blake3,
                &format!("port trait `{}` must resolve to exactly one compiler trait fact", port.port_trait),
            ));
            continue;
        }
        let (trait_fact_id, trait_span_id) = traits[0];
        let trait_unit = indexes.unit_for_fact(trait_fact_id);
        let trait_source = indexes.source_path(trait_span_id);
        let owner_valid = trait_unit.is_some_and(|unit| unit.role == CompilerArchitectureRole::ApplicationShell)
            && trait_source.is_some_and(|path| path_in_scope(path, &port.application_owner_scope));
        if !owner_valid {
            findings.push(finding_for_unit(
                "port-owner-role",
                CLAIM_PORT_OWNERSHIP,
                trait_unit,
                Some(trait_fact_id),
                trait_source,
                &format!(
                    "port trait `{}` is not owned by its declared application-shell scope `{}`",
                    port.port_trait, port.application_owner_scope
                ),
            ));
        }

        let implementations = input
            .architecture_ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Implementation {
                    fact_id,
                    trait_definition: Some(definition),
                    span_id,
                    ..
                } if definition == &port.port_trait => Some((fact_id, span_id)),
                _ => None,
            })
            .collect::<Vec<_>>();
        if implementations.is_empty() && !port.reserved {
            findings.push(new_finding(
                "port-implementation-missing",
                ArchitectureFindingSeverity::Policy,
                CLAIM_PORT_OWNERSHIP,
                None,
                None,
                None,
                None,
                input.policy_blake3,
                &format!("non-reserved port trait `{}` has no compiler-resolved implementation", port.port_trait),
            ));
        }
        for (fact_id, span_id) in implementations {
            let unit = indexes.unit_for_fact(fact_id);
            let source = indexes.source_path(span_id);
            let role_valid = unit.is_some_and(|current| {
                matches!(
                    current.role,
                    CompilerArchitectureRole::InboundAdapter | CompilerArchitectureRole::OutboundAdapter
                )
            });
            let scope_valid =
                source.is_some_and(|path| port.permitted_adapter_scopes.iter().any(|scope| path_in_scope(path, scope)));
            if !role_valid || !scope_valid {
                findings.push(finding_for_unit(
                    "port-implementation-scope",
                    CLAIM_PORT_OWNERSHIP,
                    unit,
                    Some(fact_id),
                    source,
                    &format!(
                        "port `{}` implementation is outside its permitted adapter roles or scopes",
                        port.port_trait
                    ),
                ));
            }
        }
    }
}

fn evaluate_construction(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let type_origins = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id, origin_crate, ..
            } => Some((type_id.as_str(), origin_crate.as_str())),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let roles = policy_package_roles(input.policy);
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Constructor {
            fact_id,
            constructed_type_id,
            span_id,
            ..
        } = fact
        else {
            continue;
        };
        let Some(origin) = type_origins.get(constructed_type_id.as_str()) else {
            continue;
        };
        let is_adapter = roles
            .get(*origin)
            .is_some_and(|role| matches!(role, ArchitectureRole::InboundAdapter | ArchitectureRole::OutboundAdapter));
        if !is_adapter {
            continue;
        }
        let unit = indexes.unit_for_fact(fact_id);
        if unit.is_none_or(|current| current.role != CompilerArchitectureRole::CompositionRoot) {
            findings.push(finding_for_unit(
                "adapter-construction-outside-root",
                CLAIM_ADAPTER_CONSTRUCTION,
                unit,
                Some(fact_id),
                indexes.source_path(span_id),
                &format!("concrete adapter type from package `{origin}` is constructed outside a composition root"),
            ));
        }
    }
}

fn evaluate_contract_types(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let forbidden = input
        .policy
        .ports
        .iter()
        .flat_map(|port| port.forbidden_concrete_types.iter())
        .collect::<BTreeSet<_>>();
    if forbidden.is_empty() {
        return;
    }
    let types = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Type {
                type_id,
                display,
                definition_path,
                origin_crate,
                ..
            } => Some((type_id.as_str(), (display, definition_path.as_deref(), origin_crate))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    let items = input
        .architecture_ir
        .facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                fact_id,
                item_id,
                signature_id,
                span_id,
                ..
            } => signature_id.as_ref().map(|signature| (item_id.as_str(), (fact_id, signature, span_id))),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    for fact in &input.architecture_ir.facts {
        let CompilerArchitectureFact::Signature {
            owner_item_id,
            input_type_ids,
            output_type_id,
            is_public,
            ..
        } = fact
        else {
            continue;
        };
        if !is_public {
            continue;
        }
        let Some((item_fact_id, _, span_id)) = items.get(owner_item_id.as_str()) else {
            continue;
        };
        let unit = indexes.unit_for_fact(item_fact_id);
        if !unit.is_some_and(|current| {
            matches!(current.role, CompilerArchitectureRole::DomainCore | CompilerArchitectureRole::ApplicationShell)
        }) {
            continue;
        }
        for type_id in input_type_ids.iter().chain(std::iter::once(output_type_id)) {
            let Some((display, definition, origin)) = types.get(type_id.as_str()) else {
                continue;
            };
            let leaked = forbidden.iter().any(|pattern| {
                display.contains(pattern.as_str())
                    || definition.is_some_and(|value| value.contains(pattern.as_str()))
                    || origin.contains(pattern.as_str())
            });
            if leaked {
                findings.push(finding_for_unit(
                    "infrastructure-type-leakage",
                    CLAIM_CONTRACT_BOUNDARY,
                    unit,
                    Some(item_fact_id),
                    indexes.source_path(span_id),
                    &format!("public inward contract exposes forbidden compiler type `{display}`"),
                ));
            }
        }
    }
}

fn evaluate_no_std(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    let roles = policy_package_roles(input.policy);
    for obligation in &input.policy.core_no_std {
        let package_match = roles.get(&obligation.scope).copied() == Some(ArchitectureRole::DomainCore);
        let units = if package_match {
            input
                .architecture_ir
                .units
                .iter()
                .filter(|unit| unit.package_name == obligation.scope)
                .collect::<Vec<_>>()
        } else {
            indexes.units_for_source_scope(&obligation.scope)
        };
        let has_no_std = units.iter().all(|unit| indexes.package_has_configuration(&unit.package_name, "crate:no-std"));
        if units.is_empty() || !has_no_std {
            findings.push(new_finding(
                "no-std-obligation",
                ArchitectureFindingSeverity::Policy,
                CLAIM_NO_STD,
                None,
                None,
                None,
                Some(obligation.scope.clone()),
                input.policy_blake3,
                &format!("domain-core scope `{}` lacks compiler-confirmed crate:no-std", obligation.scope),
            ));
        }
    }
}

fn evaluate_unknowns(
    input: &ProjectArchitectureGateInput<'_>,
    indexes: &GateIndexes<'_>,
    findings: &mut Vec<ProjectArchitectureFinding>,
) {
    for unknown in &input.architecture_ir.unknowns {
        let unit = indexes.unit_for_unknown(&unknown.unknown_id);
        let source = unknown.span_id.as_deref().and_then(|span_id| indexes.source_path(span_id));
        findings.push(finding_for_unit(
            "required-compiler-unknown",
            CLAIM_COMPILER_COMPLETENESS,
            unit,
            Some(&unknown.unknown_id),
            source,
            &format!("required compiler fact remains unknown: {:?}: {}", unknown.kind, unknown.detail),
        ));
    }
}

fn apply_waivers(input: &ProjectArchitectureGateInput<'_>, findings: &mut Vec<ProjectArchitectureFinding>) {
    let finding_ids = findings.iter().map(|finding| finding.finding_id.clone()).collect::<BTreeSet<_>>();
    let mut stale = Vec::new();
    for waiver in &input.policy.waivers {
        let Some(finding) = findings.iter_mut().find(|finding| finding.finding_id == waiver.finding_id) else {
            stale.push(new_finding(
                "stale-waiver",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                None,
                None,
                None,
                None,
                &waiver.source_blake3,
                &format!("architecture waiver `{}` has no current finding", waiver.finding_id),
            ));
            continue;
        };
        let identity = ArchitectureFindingIdentity {
            finding_id: &finding.finding_id,
            policy_blake3: input.policy_blake3,
            source_blake3: &finding.source_blake3,
            affected_claim: &finding.affected_claim,
        };
        if validate_architecture_waiver_binding(waiver, &identity).is_empty() {
            finding.status = ArchitectureFindingStatus::Waived;
        } else {
            stale.push(new_finding(
                "stale-waiver",
                ArchitectureFindingSeverity::Integration,
                CLAIM_INPUT_INTEGRITY,
                finding.package_name.as_deref(),
                finding.unit_id.as_deref(),
                finding.fact_id.as_deref(),
                finding.source_path.clone(),
                &finding.source_blake3,
                &format!("architecture waiver `{}` does not match current finding bindings", waiver.finding_id),
            ));
        }
    }
    debug_assert!(findings.iter().all(|finding| finding_ids.contains(&finding.finding_id)));
    findings.extend(stale);
}

fn classify_status(
    mode: ArchitectureMode,
    integration_failure: bool,
    open_policy_findings: bool,
    closure: Option<&EffectClosureParent>,
) -> ArchitecturePhaseStatus {
    if integration_failure {
        return ArchitecturePhaseStatus::IntegrationFailure;
    }
    match mode {
        ArchitectureMode::Inventory | ArchitectureMode::Advisory => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Findings
            } else {
                ArchitecturePhaseStatus::Clean
            }
        }
        ArchitectureMode::Gate => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Blocked
            } else {
                ArchitecturePhaseStatus::Clean
            }
        }
        ArchitectureMode::Admission => {
            if open_policy_findings {
                ArchitecturePhaseStatus::Blocked
            } else {
                match closure {
                    Some(parent) if parent.admitted => ArchitecturePhaseStatus::Admitted,
                    Some(_) => ArchitecturePhaseStatus::Blocked,
                    None => ArchitecturePhaseStatus::ClosureRequired,
                }
            }
        }
    }
}

struct GateIndexes<'a> {
    ir: &'a CompilerArchitectureIr,
    units_by_package: BTreeMap<&'a str, &'a CompilationUnit>,
    fact_units: BTreeMap<&'a str, &'a CompilationUnit>,
    unknown_units: BTreeMap<&'a str, &'a CompilationUnit>,
    spans: BTreeMap<&'a str, &'a SourceSpan>,
}

impl<'a> GateIndexes<'a> {
    fn new(ir: &'a CompilerArchitectureIr) -> Self {
        let units_by_package =
            ir.units.iter().map(|unit| (unit.package_name.as_str(), unit)).collect::<BTreeMap<_, _>>();
        let item_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id, origin_crate, ..
                } => Some((item_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let type_packages = ir
            .facts
            .iter()
            .filter_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id, origin_crate, ..
                } => Some((type_id.as_str(), origin_crate.as_str())),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        let mut fact_units = BTreeMap::new();
        for fact in &ir.facts {
            let package = fact_package(fact, &item_packages, &type_packages);
            if let Some(unit) = package.and_then(|package| units_by_package.get(package).copied()) {
                fact_units.insert(fact.fact_id(), unit);
            }
        }
        let unknown_units = ir
            .unknowns
            .iter()
            .filter_map(|unknown| {
                let package =
                    unknown.owner_item_id.as_deref().and_then(|item_id| item_packages.get(item_id).copied())?;
                Some((unknown.unknown_id.as_str(), *units_by_package.get(package)?))
            })
            .collect::<BTreeMap<_, _>>();
        let spans = ir.spans.iter().map(|span| (span.span_id.as_str(), span)).collect::<BTreeMap<_, _>>();
        Self {
            ir,
            units_by_package,
            fact_units,
            unknown_units,
            spans,
        }
    }

    fn unit_for_package(&self, package: &str) -> Option<&'a CompilationUnit> {
        self.units_by_package.get(package).copied()
    }

    fn unit_for_fact(&self, fact_id: &str) -> Option<&'a CompilationUnit> {
        self.fact_units.get(fact_id).copied()
    }

    fn unit_for_unknown(&self, unknown_id: &str) -> Option<&'a CompilationUnit> {
        self.unknown_units.get(unknown_id).copied()
    }

    fn source_path(&self, span_id: &str) -> Option<&'a str> {
        self.spans.get(span_id).and_then(|span| match &span.origin {
            SourceOrigin::Workspace { source_path } => Some(source_path.as_str()),
            SourceOrigin::ExternalGenerated { .. } => None,
        })
    }

    fn units_for_source_scope(&self, scope: &str) -> Vec<&'a CompilationUnit> {
        let packages = self
            .ir
            .facts
            .iter()
            .filter_map(|fact| {
                let span_id = fact_span_id(fact)?;
                let source_path = self.source_path(span_id)?;
                let unit = path_in_scope(source_path, scope).then(|| self.unit_for_fact(fact.fact_id())).flatten()?;
                Some((unit.package_name.as_str(), unit))
            })
            .collect::<BTreeMap<_, _>>();
        packages.into_values().collect()
    }

    fn package_has_configuration(&self, package: &str, predicate: &str) -> bool {
        let expected = format!("{predicate}={package}");
        self.ir.facts.iter().any(|fact| {
            matches!(fact, CompilerArchitectureFact::Configuration { predicate: current, active: true, .. } if current == &expected)
        })
    }
}

fn fact_package<'a>(
    fact: &'a CompilerArchitectureFact,
    item_packages: &BTreeMap<&str, &'a str>,
    type_packages: &BTreeMap<&str, &'a str>,
) -> Option<&'a str> {
    match fact {
        CompilerArchitectureFact::Item { origin_crate, .. } | CompilerArchitectureFact::Trait { origin_crate, .. } => {
            Some(origin_crate)
        }
        CompilerArchitectureFact::Type { origin_crate, .. } => Some(origin_crate),
        CompilerArchitectureFact::Implementation { self_type_id, .. } => {
            type_packages.get(self_type_id.as_str()).copied()
        }
        CompilerArchitectureFact::Signature { owner_item_id, .. } => item_packages.get(owner_item_id.as_str()).copied(),
        CompilerArchitectureFact::Call { caller_item_id, .. }
        | CompilerArchitectureFact::Constructor { caller_item_id, .. }
        | CompilerArchitectureFact::Effect { caller_item_id, .. }
        | CompilerArchitectureFact::CallArgumentProvenance { caller_item_id, .. } => {
            item_packages.get(caller_item_id.as_str()).copied()
        }
        CompilerArchitectureFact::TypeShape { type_id, .. } => type_packages.get(type_id.as_str()).copied(),
        CompilerArchitectureFact::FunctionSafety { item_id, .. } => item_packages.get(item_id.as_str()).copied(),
        CompilerArchitectureFact::Module { .. }
        | CompilerArchitectureFact::Expansion { .. }
        | CompilerArchitectureFact::Configuration { .. } => None,
    }
}

fn fact_span_id(fact: &CompilerArchitectureFact) -> Option<&str> {
    match fact {
        CompilerArchitectureFact::Module { span_id, .. }
        | CompilerArchitectureFact::Item { span_id, .. }
        | CompilerArchitectureFact::Trait { span_id, .. }
        | CompilerArchitectureFact::Implementation { span_id, .. }
        | CompilerArchitectureFact::Call { span_id, .. }
        | CompilerArchitectureFact::Constructor { span_id, .. }
        | CompilerArchitectureFact::Effect { span_id, .. }
        | CompilerArchitectureFact::TypeShape { span_id, .. }
        | CompilerArchitectureFact::FunctionSafety { span_id, .. }
        | CompilerArchitectureFact::CallArgumentProvenance { span_id, .. }
        | CompilerArchitectureFact::Configuration {
            span_id: Some(span_id), ..
        } => Some(span_id),
        CompilerArchitectureFact::Type { .. }
        | CompilerArchitectureFact::Signature { .. }
        | CompilerArchitectureFact::Expansion { .. }
        | CompilerArchitectureFact::Configuration { span_id: None, .. } => None,
    }
}

fn policy_package_roles(policy: &ProjectArchitecturePolicy) -> BTreeMap<String, ArchitectureRole> {
    policy
        .roles
        .iter()
        .filter(|assignment| assignment.scope_kind == ArchitectureScopeKind::Package && assignment.role.is_production())
        .map(|assignment| (assignment.scope.clone(), assignment.role))
        .collect()
}

fn compiler_role(role: ArchitectureRole) -> CompilerArchitectureRole {
    match role {
        ArchitectureRole::DomainCore => CompilerArchitectureRole::DomainCore,
        ArchitectureRole::ApplicationShell => CompilerArchitectureRole::ApplicationShell,
        ArchitectureRole::InboundAdapter => CompilerArchitectureRole::InboundAdapter,
        ArchitectureRole::OutboundAdapter => CompilerArchitectureRole::OutboundAdapter,
        ArchitectureRole::CompositionRoot => CompilerArchitectureRole::CompositionRoot,
        ArchitectureRole::Test => CompilerArchitectureRole::Test,
        ArchitectureRole::Generated => CompilerArchitectureRole::Generated,
    }
}

fn policy_capability(family: CapabilityFamily) -> Option<ArchitectureCapabilityFamily> {
    match family {
        CapabilityFamily::Time => Some(ArchitectureCapabilityFamily::Time),
        CapabilityFamily::Randomness => Some(ArchitectureCapabilityFamily::Randomness),
        CapabilityFamily::Filesystem => Some(ArchitectureCapabilityFamily::Filesystem),
        CapabilityFamily::Network => Some(ArchitectureCapabilityFamily::Network),
        CapabilityFamily::Process => Some(ArchitectureCapabilityFamily::Process),
        CapabilityFamily::Environment => Some(ArchitectureCapabilityFamily::Environment),
        CapabilityFamily::Secrets => Some(ArchitectureCapabilityFamily::Secrets),
        CapabilityFamily::UnsafeCode => Some(ArchitectureCapabilityFamily::UnsafeCode),
        CapabilityFamily::GlobalState => Some(ArchitectureCapabilityFamily::GlobalState),
        CapabilityFamily::Concurrency => Some(ArchitectureCapabilityFamily::Concurrency),
        CapabilityFamily::Observability => Some(ArchitectureCapabilityFamily::Observability),
        CapabilityFamily::Ffi => Some(ArchitectureCapabilityFamily::Ffi),
        CapabilityFamily::Allocation => Some(ArchitectureCapabilityFamily::Allocation),
        CapabilityFamily::Unknown => None,
    }
}

fn finding_for_unit(
    code: &str,
    affected_claim: &str,
    unit: Option<&CompilationUnit>,
    fact_id: Option<&str>,
    source_path: Option<&str>,
    message: &str,
) -> ProjectArchitectureFinding {
    let source_blake3 = unit
        .map(|current| current.source_blake3.as_str())
        .or(fact_id)
        .unwrap_or("0000000000000000000000000000000000000000000000000000000000000000");
    new_finding(
        code,
        ArchitectureFindingSeverity::Policy,
        affected_claim,
        unit.map(|current| current.package_name.as_str()),
        unit.map(|current| current.unit_id.as_str()),
        fact_id,
        source_path.map(str::to_owned),
        source_blake3,
        message,
    )
}

#[allow(clippy::too_many_arguments)]
fn new_finding(
    code: &str,
    severity: ArchitectureFindingSeverity,
    affected_claim: &str,
    package_name: Option<&str>,
    unit_id: Option<&str>,
    fact_id: Option<&str>,
    source_path: Option<String>,
    source_blake3: &str,
    message: &str,
) -> ProjectArchitectureFinding {
    let finding_id = octet_architecture_ir::canonical_identity(
        FINDING_ID_DOMAIN,
        &(code, affected_claim, package_name, unit_id, fact_id, &source_path, source_blake3, message),
    )
    .expect("project architecture finding identity fields must serialize");
    ProjectArchitectureFinding {
        finding_id,
        code: code.to_owned(),
        severity,
        status: ArchitectureFindingStatus::Open,
        affected_claim: affected_claim.to_owned(),
        package_name: package_name.map(str::to_owned),
        unit_id: unit_id.map(str::to_owned),
        fact_id: fact_id.map(str::to_owned),
        source_path,
        source_blake3: source_blake3.to_owned(),
        message: message.to_owned(),
    }
}

fn receipt_identity(receipt: &ProjectArchitectureReceipt) -> String {
    let mut projection = receipt.clone();
    projection.receipt_id.clear();
    octet_architecture_ir::canonical_identity(RECEIPT_ID_DOMAIN, &projection)
        .expect("project architecture receipt identity fields must serialize")
}

fn path_in_scope(path: &str, scope: &str) -> bool {
    path == scope || path.strip_prefix(scope).is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use octet_architecture_ir::ArchitectureCollectionLimits;
    use octet_architecture_ir::ItemKind;
    use octet_architecture_ir::UnknownCompilerFact;
    use octet_architecture_ir::UnknownFactKind;

    use super::*;
    use crate::project_architecture_policy::ArchitectureCapabilityClassification;
    use crate::project_architecture_policy::ArchitectureDependencyRule;
    use crate::project_architecture_policy::ArchitectureEffectAuthority;
    use crate::project_architecture_policy::ArchitectureOutboundEffectClassification;
    use crate::project_architecture_policy::ArchitecturePort;
    use crate::project_architecture_policy::ArchitectureProductionScope;
    use crate::project_architecture_policy::ArchitectureProtectedEffectProtocol;
    use crate::project_architecture_policy::ArchitectureRoleAssignment;
    use crate::project_architecture_policy::ArchitectureTarget;
    use crate::project_architecture_policy::ArchitectureWaiver;
    use crate::project_architecture_policy::ArchitectureWitnessUse;
    use crate::project_architecture_policy::CoreCapabilityHandling;
    use crate::project_architecture_policy::PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const TARGET: &str = "x86_64-unknown-linux-gnu";

    #[test]
    fn generated_receipt_contract_is_fresh() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let path = root.join(PROJECT_ARCHITECTURE_RECEIPT_CONTRACT_PATH);
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read generated receipt contract {}: {error}", path.display()));

        assert_eq!(expected, render_project_architecture_receipt_contract());
    }

    #[test]
    fn clean_role_graph_passes_and_receipt_repeats() {
        let fixture = Fixture::new(ArchitectureMode::Gate);
        let first = fixture.evaluate(None);
        let second = fixture.evaluate(None);

        assert_eq!(first.status, ArchitecturePhaseStatus::Clean);
        assert!(first.findings.is_empty());
        assert_eq!(first, second);
        assert!(verify_project_architecture_receipt(&first).is_empty());
    }

    #[test]
    fn forbidden_role_edge_blocks_gate_but_not_inventory() {
        let mut gate = Fixture::new(ArchitectureMode::Gate);
        gate.graph.edges.push(edge("domain", "outbound"));
        gate.refresh_graph_id();
        let gate_receipt = gate.evaluate(None);
        assert_eq!(gate_receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(gate_receipt.findings.iter().any(|finding| finding.code == "forbidden-role-edge"));

        gate.policy.mode = ArchitectureMode::Inventory;
        let inventory = gate.evaluate(None);
        assert_eq!(inventory.status, ArchitecturePhaseStatus::Findings);
        assert_eq!(inventory.findings, gate_receipt.findings);
    }

    #[test]
    fn unclassified_external_provider_blocks_strict_mode() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.graph.edges.push(edge("domain", "ambient-vendor"));
        fixture.refresh_graph_id();

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| {
            finding.code == "forbidden-role-edge" && finding.message.contains("unclassified external provider")
        }));
    }

    #[test]
    fn core_effect_and_required_unknown_block_gate() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_domain_effect();
        fixture.add_domain_unknown();

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| finding.code == "domain-core-effect"));
        assert!(receipt.findings.iter().any(|finding| finding.code == "required-compiler-unknown"));
    }

    #[test]
    fn complete_port_placement_passes_and_shell_implementation_blocks() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_port_facts("outbound");
        let clean = fixture.evaluate(None);
        assert_eq!(clean.status, ArchitecturePhaseStatus::Clean);

        let mut invalid = Fixture::new(ArchitectureMode::Gate);
        invalid.add_port_facts("application");
        let blocked = invalid.evaluate(None);
        assert_eq!(blocked.status, ArchitecturePhaseStatus::Blocked);
        assert!(blocked.findings.iter().any(|finding| finding.code == "port-implementation-scope"));
    }

    #[test]
    fn incomplete_compiler_coverage_is_an_integration_failure_in_advisory_mode() {
        let mut fixture = Fixture::new(ArchitectureMode::Advisory);
        fixture.coverage.status = CoverageStatus::Incomplete;

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::IntegrationFailure);
        assert!(receipt.findings.iter().any(|finding| finding.code == "compiler-coverage-incomplete"));
    }

    #[test]
    fn exact_waiver_is_retained_while_stale_waiver_fails_closed() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.graph.edges.push(edge("domain", "outbound"));
        fixture.refresh_graph_id();
        let finding = fixture.evaluate(None).findings.remove(0);
        fixture.policy.waivers.push(ArchitectureWaiver {
            finding_id: finding.finding_id.clone(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: finding.source_blake3.clone(),
            owner: "architecture-team".to_owned(),
            reason: "bounded migration".to_owned(),
            review_condition: "remove after provider extraction".to_owned(),
            affected_claim: finding.affected_claim.clone(),
        });
        let waived = fixture.evaluate(None);
        assert_eq!(waived.status, ArchitecturePhaseStatus::Clean);
        assert_eq!(waived.findings[0].status, ArchitectureFindingStatus::Waived);

        fixture.policy.waivers[0].source_blake3 = HASH_C.to_owned();
        let stale = fixture.evaluate(None);
        assert_eq!(stale.status, ArchitecturePhaseStatus::IntegrationFailure);
        assert!(stale.findings.iter().any(|current| current.code == "stale-waiver"));
    }

    #[test]
    fn admission_keeps_effect_closure_separate() {
        let fixture = Fixture::new(ArchitectureMode::Admission);
        assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::ClosureRequired);
        let closure = EffectClosureParent {
            receipt_id: HASH_C.to_owned(),
            admitted: true,
            evidence_tier: "selected-source-token-closure".to_owned(),
            non_claims: vec!["complete-mir-closure".to_owned()],
        };
        let admitted = fixture.evaluate(Some(&closure));
        assert_eq!(admitted.status, ArchitecturePhaseStatus::Admitted);
        assert_eq!(admitted.effect_closure, Some(closure));
        assert!(!admitted.non_claims.iter().any(|claim| claim == "complete-mir-closure"));

        let denied = EffectClosureParent {
            receipt_id: HASH_B.to_owned(),
            admitted: false,
            evidence_tier: "policy-check".to_owned(),
            non_claims: vec!["complete-mir-closure".to_owned()],
        };
        let blocked = fixture.evaluate(Some(&denied));
        assert_eq!(blocked.status, ArchitecturePhaseStatus::Blocked);
        assert_eq!(blocked.effect_closure, Some(denied));
    }

    #[test]
    fn protected_effect_protocol_accepts_exact_authority_and_observation_provenance() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Clean);
        assert!(receipt.findings.is_empty());
        assert_eq!(receipt.outbound_effects.len(), 1);
        assert!(!receipt.authorized_effect_fact_ids.is_empty());
        assert!(!receipt.authorized_effect_type_ids.is_empty());
        assert!(receipt.non_claims.iter().any(|claim| claim == "credential-validity"));
    }

    #[test]
    fn protected_effect_protocol_blocks_ambiguous_origin_and_forging_constructor() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Merged, "domain::forge_publication", true);

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| finding.code == "unknown-witness-origin"));
        assert!(receipt.findings.iter().any(|finding| finding.code == "undeclared-witness-constructor"));
        assert!(receipt.findings.iter().any(|finding| finding.code == "witness-forbidden-trait"));
        mutate_witness_shape(&mut fixture.ir, |_, traits| {
            traits.push("core::convert::From<application::Request>".to_owned());
        });
        let converted = fixture.evaluate(None);
        assert!(converted.findings.iter().any(|finding| finding.code == "witness-broad-conversion"));
    }

    #[test]
    fn protected_effect_protocol_rejects_public_replayable_and_ambient_witness_shapes() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        mutate_witness_visibility(&mut fixture.ir, "public");
        mutate_witness_shape(&mut fixture.ir, |_, traits| {
            traits.extend([
                "core::marker::Copy".to_owned(),
                "core::clone::Clone".to_owned(),
                "core::default::Default".to_owned(),
                "serde::Deserialize".to_owned(),
                "core::convert::From<application::Request>".to_owned(),
            ]);
        });

        let receipt = fixture.evaluate(None);

        assert!(receipt.findings.iter().any(|finding| finding.code == "witness-field-visibility"));
        assert!(receipt.findings.iter().any(|finding| finding.code == "witness-replayable"));
        for forbidden in ["Copy", "Clone", "Default", "Deserialize"] {
            assert!(
                receipt.findings.iter().any(|finding| {
                    finding.code == "witness-forbidden-trait" && finding.message.contains(forbidden)
                })
            );
        }
        assert!(receipt.findings.iter().any(|finding| finding.code == "witness-broad-conversion"));
    }

    #[test]
    fn protected_effect_protocol_rejects_public_and_unsafe_witness_constructors() {
        let mut public = Fixture::new(ArchitectureMode::Gate);
        public.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        mutate_witness_constructor(&mut public.ir, "public", false);
        let public_receipt = public.evaluate(None);
        assert!(public_receipt.findings.iter().any(|finding| finding.code == "public-witness-constructor"));

        let mut unsafe_constructor = Fixture::new(ArchitectureMode::Gate);
        unsafe_constructor.add_protected_effect_facts(
            ValueProvenanceStatus::Exact,
            "domain::authorize_publication",
            false,
        );
        mutate_witness_constructor(&mut unsafe_constructor.ir, "restricted", true);
        let unsafe_receipt = unsafe_constructor.evaluate(None);
        assert!(unsafe_receipt.findings.iter().any(|finding| finding.code == "unsafe-witness-constructor"));
    }

    #[test]
    fn protected_effect_protocol_rejects_missing_witness_and_pre_decision_origin() {
        let mut missing = Fixture::new(ArchitectureMode::Gate);
        missing.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        remove_executor_provenance(&mut missing.ir);
        remove_executor_witness_parameter(&mut missing.ir);
        let missing_receipt = missing.evaluate(None);
        assert!(
            missing_receipt
                .findings
                .iter()
                .any(|finding| finding.code == "protected-executor-provenance-missing")
        );
        assert!(
            missing_receipt
                .findings
                .iter()
                .any(|finding| finding.code == "protected-executor-witness-parameter")
        );

        let mut pre_decision = Fixture::new(ArchitectureMode::Gate);
        pre_decision.add_protected_effect_facts(
            ValueProvenanceStatus::Exact,
            "application::prepare_publication",
            false,
        );
        let pre_decision_receipt = pre_decision.evaluate(None);
        assert!(pre_decision_receipt.findings.iter().any(|finding| finding.code == "unauthorized-witness-origin"));
    }

    #[test]
    fn protected_effect_protocol_accepts_borrow_inside_declared_scope() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        let protocol = fixture.policy.outbound_effects[0].protected.as_mut().expect("protected protocol");
        protocol.witness_use = ArchitectureWitnessUse::Borrow;
        protocol.borrow_scope = Some("crates/outbound/src".to_owned());
        protocol.borrow_rationale = Some("one bounded outbound request".to_owned());

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Clean);
        assert!(receipt.findings.is_empty());
    }

    #[test]
    fn protected_effect_protocol_rejects_missing_binding_decision_output_and_borrow_escape() {
        let mut missing_binding = Fixture::new(ArchitectureMode::Gate);
        missing_binding.add_protected_effect_facts(
            ValueProvenanceStatus::Exact,
            "domain::authorize_publication",
            false,
        );
        mutate_witness_shape(&mut missing_binding.ir, |field_types, _| field_types.clear());
        let binding_receipt = missing_binding.evaluate(None);
        assert!(binding_receipt.findings.iter().any(|finding| finding.code == "witness-binding-type-missing"));

        let mut wrong_output = Fixture::new(ArchitectureMode::Gate);
        wrong_output.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        mutate_decision_output(&mut wrong_output.ir, "domain::authorize_publication", "domain::PublicationAuthority");
        let output_receipt = wrong_output.evaluate(None);
        assert!(output_receipt.findings.iter().any(|finding| finding.code == "authority-decision-output"));

        let mut borrow_escape = Fixture::new(ArchitectureMode::Gate);
        borrow_escape.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        let protocol = borrow_escape.policy.outbound_effects[0].protected.as_mut().expect("protected protocol");
        protocol.witness_use = ArchitectureWitnessUse::Borrow;
        protocol.borrow_scope = Some("crates/application/src".to_owned());
        protocol.borrow_rationale = Some("one application request".to_owned());
        let borrow_receipt = borrow_escape.evaluate(None);
        assert!(borrow_receipt.findings.iter().any(|finding| finding.code == "borrowed-witness-scope"));
    }

    #[test]
    fn protected_effect_protocol_blocks_unguarded_effect_and_receipt_promotion() {
        let mut unguarded = Fixture::new(ArchitectureMode::Gate);
        unguarded.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        let span = span("crates/outbound/src/helper.rs");
        let helper = item(&span.span_id, "outbound::unguarded_publish");
        let mut effect = CompilerArchitectureFact::Effect {
            fact_id: String::new(),
            caller_item_id: item_id(&helper).to_owned(),
            capability_family: CapabilityFamily::Filesystem,
            primitive_definition: "std::fs::write".to_owned(),
            span_id: span.span_id.clone(),
        };
        set_fact_id(&mut effect);
        unguarded.add_unit_facts("outbound", vec![span], vec![helper, effect], Vec::new());
        let unguarded_receipt = unguarded.evaluate(None);
        assert!(unguarded_receipt.findings.iter().any(|finding| finding.code == "unguarded-protected-effect"));

        let mut promoted = Fixture::new(ArchitectureMode::Gate);
        promoted.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        let witness_type_id = promoted
            .ir
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id,
                    definition_path: Some(definition),
                    ..
                } if definition == "domain::PublicationAuthority" => Some(type_id.clone()),
                _ => None,
            })
            .expect("witness type");
        mutate_receipt_provenance(&mut promoted.ir, |argument_type_id, origins, status| {
            *argument_type_id = witness_type_id;
            *origins = vec!["domain::authorize_publication".to_owned()];
            *status = ValueProvenanceStatus::Exact;
        });
        let promoted_receipt = promoted.evaluate(None);
        assert!(promoted_receipt.findings.iter().any(|finding| finding.code == "planned-effect-promoted-to-receipt"));

        let mut false_success = Fixture::new(ArchitectureMode::Gate);
        false_success.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        mutate_receipt_provenance(&mut false_success.ir, |_, origins, status| {
            *origins = vec!["application::PublicationObservation::failure".to_owned()];
            *status = ValueProvenanceStatus::Exact;
        });
        let false_success_receipt = false_success.evaluate(None);
        assert!(
            false_success_receipt
                .findings
                .iter()
                .any(|finding| finding.code == "failure-promoted-to-success-receipt")
        );
    }

    #[test]
    fn receipt_verifier_rejects_protocol_overclaim_and_tampering() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        let mut receipt = fixture.evaluate(None);
        receipt.non_claims.retain(|claim| claim != "credential-validity");
        receipt.authorized_effect_fact_ids.clear();

        let diagnostics = verify_project_architecture_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("credential-validity")));
    }

    #[test]
    fn unclassified_outbound_effect_blocks_strict_mode() {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_outbound_effect_only();

        let receipt = fixture.evaluate(None);

        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(receipt.findings.iter().any(|finding| finding.code == "unclassified-effect-authority"));
    }

    #[test]
    fn receipt_verifier_rejects_tampering_and_missing_non_claims() {
        let fixture = Fixture::new(ArchitectureMode::Gate);
        let mut receipt = fixture.evaluate(None);
        receipt.project_id = "tampered".to_owned();
        receipt.non_claims.retain(|claim| claim != "whole-program-correctness");

        let diagnostics = verify_project_architecture_receipt(&receipt);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program-correctness")));
    }

    struct Fixture {
        policy: ProjectArchitecturePolicy,
        graph: ArchitectureCargoGraph,
        ir: CompilerArchitectureIr,
        coverage: CompilerArchitectureCoverage,
        compiler_receipt: CompilerArchitectureReceipt,
    }

    impl Fixture {
        #[allow(clippy::too_many_lines)]
        fn new(mode: ArchitectureMode) -> Self {
            let package_roles = [
                ("domain", ArchitectureRole::DomainCore),
                ("application", ArchitectureRole::ApplicationShell),
                ("inbound", ArchitectureRole::InboundAdapter),
                ("outbound", ArchitectureRole::OutboundAdapter),
                ("host", ArchitectureRole::CompositionRoot),
            ];
            let source_roles = [
                ("crates/domain/src", ArchitectureRole::DomainCore),
                ("crates/application/src", ArchitectureRole::ApplicationShell),
                ("crates/inbound/src", ArchitectureRole::InboundAdapter),
                ("crates/outbound/src", ArchitectureRole::OutboundAdapter),
                ("crates/host/src", ArchitectureRole::CompositionRoot),
            ];
            let mut roles = package_roles
                .iter()
                .map(|(scope, role)| role_assignment(ArchitectureScopeKind::Package, scope, *role))
                .collect::<Vec<_>>();
            roles.extend(
                source_roles
                    .iter()
                    .map(|(scope, role)| role_assignment(ArchitectureScopeKind::Source, scope, *role)),
            );
            let units =
                package_roles.iter().map(|(package, role)| unit(package, compiler_role(*role))).collect::<Vec<_>>();
            let policy = ProjectArchitecturePolicy {
                schema_version: PROJECT_ARCHITECTURE_POLICY_SCHEMA_VERSION.to_owned(),
                project_id: "fixture".to_owned(),
                mode,
                production: ArchitectureProductionScope {
                    packages: package_roles.iter().map(|(package, _)| (*package).to_owned()).collect(),
                    source_scopes: source_roles.iter().map(|(scope, _)| (*scope).to_owned()).collect(),
                    targets: vec![ArchitectureTarget {
                        triple: TARGET.to_owned(),
                        features: Vec::new(),
                        default_features: true,
                    }],
                },
                roles,
                dependency_rules: package_roles
                    .iter()
                    .map(|(_, role)| ArchitectureDependencyRule {
                        caller_role: *role,
                        allowed_callee_roles: allowed_roles(*role),
                        edge_kinds: vec!["normal".to_owned(), "build".to_owned()],
                    })
                    .collect(),
                provider_classifications: Vec::new(),
                ports: Vec::new(),
                capability_classifications: vec![ArchitectureCapabilityClassification {
                    family: ArchitectureCapabilityFamily::Filesystem,
                    core_handling: CoreCapabilityHandling::EffectPlan,
                    executor_role: ArchitectureRole::OutboundAdapter,
                    rationale: "outbound adapter executes filesystem plans".to_owned(),
                }],
                outbound_effects: Vec::new(),
                core_no_std: Vec::new(),
                admission: (mode == ArchitectureMode::Admission).then(|| {
                    crate::project_architecture_policy::ArchitectureAdmissionBinding {
                        strict_profile: "deterministic-core".to_owned(),
                        effect_closure_policy: "octet-effect-closure.json".to_owned(),
                    }
                }),
                waivers: Vec::new(),
                collection_limits: limits(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            let mut graph = ArchitectureCargoGraph {
                schema_version: PROJECT_ARCHITECTURE_CARGO_GRAPH_SCHEMA_VERSION.to_owned(),
                cargo_graph_id: String::new(),
                cargo_lock_blake3: HASH_A.to_owned(),
                cargo_metadata_blake3: HASH_B.to_owned(),
                selected_packages: vec![
                    "application".to_owned(),
                    "domain".to_owned(),
                    "host".to_owned(),
                    "inbound".to_owned(),
                    "outbound".to_owned(),
                ],
                target_triple: "x86_64-unknown-linux-gnu".to_owned(),
                features: Vec::new(),
                default_features: true,
                edges: vec![
                    edge("application", "domain"),
                    edge("inbound", "application"),
                    edge("outbound", "application"),
                    edge("host", "application"),
                    edge("host", "inbound"),
                    edge("host", "outbound"),
                ],
            };
            graph.edges.sort();
            graph.cargo_graph_id = architecture_cargo_graph_identity(&graph);
            let ir = CompilerArchitectureIr {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION.to_owned(),
                architecture_ir_id: HASH_B.to_owned(),
                policy_blake3: HASH_A.to_owned(),
                units,
                spans: Vec::new(),
                facts: Vec::new(),
                unknowns: Vec::new(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            let coverage = CompilerArchitectureCoverage {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION.to_owned(),
                coverage_id: HASH_C.to_owned(),
                status: CoverageStatus::Complete,
                expected_unit_ids: ir.units.iter().map(|unit| unit.unit_id.clone()).collect(),
                observed_unit_ids: ir.units.iter().map(|unit| unit.unit_id.clone()).collect(),
                missing_unit_ids: Vec::new(),
                unexpected_unit_ids: Vec::new(),
                duplicate_unit_ids: Vec::new(),
                invalid_shard_ids: Vec::new(),
            };
            let compiler_receipt = CompilerArchitectureReceipt {
                schema_version: octet_architecture_ir::COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
                receipt_id: HASH_B.to_owned(),
                status: CollectionStatus::Complete,
                policy_blake3: HASH_A.to_owned(),
                coverage_id: HASH_C.to_owned(),
                architecture_ir_id: Some(HASH_B.to_owned()),
                issue_ids: Vec::new(),
                non_claims: vec!["whole-program-correctness".to_owned()],
            };
            Self {
                policy,
                graph,
                ir,
                coverage,
                compiler_receipt,
            }
        }

        fn evaluate(&self, effect_closure: Option<&EffectClosureParent>) -> ProjectArchitectureReceipt {
            evaluate_project_architecture(&ProjectArchitectureGateInput {
                policy: &self.policy,
                policy_source_blake3: HASH_C,
                policy_blake3: HASH_A,
                lint_phase_status: "clean",
                command_blake3: HASH_B,
                cargo_graph: &self.graph,
                architecture_ir: &self.ir,
                coverage: &self.coverage,
                compiler_receipt: &self.compiler_receipt,
                effect_closure,
            })
        }

        fn refresh_graph_id(&mut self) {
            self.graph.edges.sort();
            self.graph.cargo_graph_id = architecture_cargo_graph_identity(&self.graph);
        }

        fn add_domain_effect(&mut self) {
            let span = span("crates/domain/src/lib.rs");
            let item = item(&span.span_id, "domain::decide");
            let mut effect = CompilerArchitectureFact::Effect {
                fact_id: String::new(),
                caller_item_id: item_id(&item).to_owned(),
                capability_family: CapabilityFamily::Filesystem,
                primitive_definition: "std::fs::read".to_owned(),
                span_id: span.span_id.clone(),
            };
            set_fact_id(&mut effect);
            self.add_unit_facts("domain", vec![span], vec![item, effect], Vec::new());
        }

        fn add_outbound_effect_only(&mut self) {
            let effect_span = span("crates/outbound/src/filesystem.rs");
            let executor = item(&effect_span.span_id, "outbound::FilesystemPort::publish");
            let mut effect = CompilerArchitectureFact::Effect {
                fact_id: String::new(),
                caller_item_id: item_id(&executor).to_owned(),
                capability_family: CapabilityFamily::Filesystem,
                primitive_definition: "std::fs::write".to_owned(),
                span_id: effect_span.span_id.clone(),
            };
            set_fact_id(&mut effect);
            self.add_unit_facts("outbound", vec![effect_span], vec![executor, effect], Vec::new());
        }

        #[allow(clippy::too_many_lines)]
        fn add_protected_effect_facts(
            &mut self,
            origin_status: ValueProvenanceStatus,
            witness_origin: &str,
            forge_witness: bool,
        ) {
            let protocol = ArchitectureProtectedEffectProtocol {
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
                witness_use: ArchitectureWitnessUse::Consume,
                borrow_scope: None,
                borrow_rationale: None,
                non_claims: vec![
                    "credential-validity".to_owned(),
                    "external-execution-success".to_owned(),
                ],
            };
            self.policy.outbound_effects.push(ArchitectureOutboundEffectClassification {
                family: ArchitectureCapabilityFamily::Filesystem,
                executor_scope: "crates/outbound/src".to_owned(),
                executor_methods: vec!["outbound::FilesystemPort::publish".to_owned()],
                authority: ArchitectureEffectAuthority::Protected,
                rationale: "publication requires authority".to_owned(),
                protected: Some(protocol.clone()),
            });
            let domain_span = span("crates/domain/src/authority.rs");
            let authority_input_type = type_fact("domain", &protocol.authority_input_types[0]);
            let authority_input_type_id = type_id(&authority_input_type).to_owned();
            let authorized_output_type = type_fact("domain", &protocol.authorized_output_types[0]);
            let authorized_output_type_id = type_id(&authorized_output_type).to_owned();
            let decision = item(&domain_span.span_id, &protocol.decision_functions[0]);
            let mut decision_signature = CompilerArchitectureFact::Signature {
                fact_id: String::new(),
                signature_id: octet_architecture_ir::canonical_identity(
                    "fixture-signature",
                    &protocol.decision_functions[0],
                )
                .expect("signature id"),
                owner_item_id: item_id(&decision).to_owned(),
                input_type_ids: vec![authority_input_type_id],
                output_type_id: authorized_output_type_id.clone(),
                is_public: true,
            };
            set_fact_id(&mut decision_signature);
            let witness_type = type_fact("domain", &protocol.witness_type);
            let witness_type_id = type_id(&witness_type).to_owned();
            let mut witness_shape = CompilerArchitectureFact::TypeShape {
                fact_id: String::new(),
                type_id: witness_type_id.clone(),
                definition_path: protocol.witness_type.clone(),
                field_visibilities: vec!["restricted:domain".to_owned()],
                field_type_definitions: protocol.witness_binding_types.clone(),
                implemented_traits: forge_witness.then(|| "core::clone::Clone".to_owned()).into_iter().collect(),
                span_id: domain_span.span_id.clone(),
            };
            set_fact_id(&mut witness_shape);
            let constructor_name = if forge_witness {
                "domain::PublicationAuthority::forge"
            } else {
                "domain::PublicationAuthority::from_decision"
            };
            let mut constructor = item(&domain_span.span_id, constructor_name);
            set_item_visibility(&mut constructor, "restricted");
            let mut constructor_signature = CompilerArchitectureFact::Signature {
                fact_id: String::new(),
                signature_id: octet_architecture_ir::canonical_identity("fixture-signature", &constructor_name)
                    .expect("signature id"),
                owner_item_id: item_id(&constructor).to_owned(),
                input_type_ids: vec![authorized_output_type_id.clone()],
                output_type_id: witness_type_id.clone(),
                is_public: false,
            };
            set_fact_id(&mut constructor_signature);
            let mut constructor_safety = CompilerArchitectureFact::FunctionSafety {
                fact_id: String::new(),
                item_id: item_id(&constructor).to_owned(),
                qualified_path: constructor_name.to_owned(),
                output_type_id: witness_type_id.clone(),
                is_unsafe: false,
                span_id: domain_span.span_id.clone(),
            };
            set_fact_id(&mut constructor_safety);
            self.add_unit_facts(
                "domain",
                vec![domain_span],
                vec![
                    authority_input_type,
                    authorized_output_type,
                    decision,
                    decision_signature,
                    witness_type,
                    witness_shape,
                    constructor,
                    constructor_signature,
                    constructor_safety,
                ],
                Vec::new(),
            );

            let application_span = span("crates/application/src/receipt.rs");
            let observation_type = type_fact("application", &protocol.observed_outcome_type);
            let observation_type_id = type_id(&observation_type).to_owned();
            let receipt_caller = item(&application_span.span_id, "application::record_publication");
            let mut receipt_provenance = CompilerArchitectureFact::CallArgumentProvenance {
                fact_id: String::new(),
                caller_item_id: item_id(&receipt_caller).to_owned(),
                callee_definition: protocol.receipt_constructors[0].clone(),
                argument_position: 0,
                argument_type_id: observation_type_id.clone(),
                origin_definitions: vec![protocol.success_observation_constructors[0].clone()],
                status: ValueProvenanceStatus::Exact,
                span_id: application_span.span_id.clone(),
            };
            set_fact_id(&mut receipt_provenance);
            let mut failure_receipt_provenance = CompilerArchitectureFact::CallArgumentProvenance {
                fact_id: String::new(),
                caller_item_id: item_id(&receipt_caller).to_owned(),
                callee_definition: protocol.failure_receipt_constructors[0].clone(),
                argument_position: 0,
                argument_type_id: observation_type_id,
                origin_definitions: vec![protocol.failure_observation_constructors[0].clone()],
                status: ValueProvenanceStatus::Exact,
                span_id: application_span.span_id.clone(),
            };
            set_fact_id(&mut failure_receipt_provenance);
            self.add_unit_facts(
                "application",
                vec![application_span],
                vec![
                    observation_type,
                    receipt_caller,
                    receipt_provenance,
                    failure_receipt_provenance,
                ],
                Vec::new(),
            );

            let outbound_span = span("crates/outbound/src/filesystem.rs");
            let executor = item(&outbound_span.span_id, &self.policy.outbound_effects[0].executor_methods[0]);
            let mut executor_signature = CompilerArchitectureFact::Signature {
                fact_id: String::new(),
                signature_id: octet_architecture_ir::canonical_identity(
                    "fixture-signature",
                    &self.policy.outbound_effects[0].executor_methods[0],
                )
                .expect("signature id"),
                owner_item_id: item_id(&executor).to_owned(),
                input_type_ids: vec![witness_type_id.clone()],
                output_type_id: witness_type_id.clone(),
                is_public: true,
            };
            set_fact_id(&mut executor_signature);
            let mut executor_provenance = CompilerArchitectureFact::CallArgumentProvenance {
                fact_id: String::new(),
                caller_item_id: item_id(&executor).to_owned(),
                callee_definition: self.policy.outbound_effects[0].executor_methods[0].clone(),
                argument_position: 1,
                argument_type_id: witness_type_id,
                origin_definitions: if origin_status == ValueProvenanceStatus::Merged {
                    vec![witness_origin.to_owned(), "domain::other_decision".to_owned()]
                } else {
                    vec![witness_origin.to_owned()]
                },
                status: origin_status,
                span_id: outbound_span.span_id.clone(),
            };
            set_fact_id(&mut executor_provenance);
            let mut effect = CompilerArchitectureFact::Effect {
                fact_id: String::new(),
                caller_item_id: item_id(&executor).to_owned(),
                capability_family: CapabilityFamily::Filesystem,
                primitive_definition: "std::fs::write".to_owned(),
                span_id: outbound_span.span_id.clone(),
            };
            set_fact_id(&mut effect);
            self.add_unit_facts(
                "outbound",
                vec![outbound_span],
                vec![executor, executor_signature, executor_provenance, effect],
                Vec::new(),
            );
        }

        fn add_domain_unknown(&mut self) {
            let span = span("crates/domain/src/dispatch.rs");
            let item = item(&span.span_id, "domain::dispatch");
            let mut unknown = UnknownCompilerFact {
                unknown_id: String::new(),
                kind: UnknownFactKind::DynamicDispatch,
                owner_item_id: Some(item_id(&item).to_owned()),
                target_identity: Some("dyn domain::Port".to_owned()),
                span_id: Some(span.span_id.clone()),
                detail: "dynamic target is unresolved".to_owned(),
            };
            unknown.unknown_id = octet_architecture_ir::unknown_fact_identity(&unknown);
            self.add_unit_facts("domain", vec![span], vec![item], vec![unknown]);
        }

        fn add_port_facts(&mut self, implementation_package: &str) {
            self.policy.ports.push(ArchitecturePort {
                port_trait: "application::ports::Repository".to_owned(),
                application_owner_scope: "crates/application/src".to_owned(),
                permitted_adapter_scopes: vec!["crates/outbound/src".to_owned()],
                forbidden_concrete_types: Vec::new(),
                reserved: false,
            });
            let trait_span = span("crates/application/src/ports.rs");
            let mut trait_fact = CompilerArchitectureFact::Trait {
                fact_id: String::new(),
                trait_id: HASH_C.to_owned(),
                qualified_path: "application::ports::Repository".to_owned(),
                origin_crate: "application".to_owned(),
                span_id: trait_span.span_id.clone(),
            };
            set_fact_id(&mut trait_fact);
            self.add_unit_facts("application", vec![trait_span], vec![trait_fact], Vec::new());

            let implementation_scope = if implementation_package == "outbound" {
                "crates/outbound/src/repository.rs"
            } else {
                "crates/application/src/repository.rs"
            };
            let implementation_span = span(implementation_scope);
            let self_type_id = octet_architecture_ir::canonical_identity(
                "fixture-type",
                &(implementation_package, "RepositoryAdapter"),
            )
            .expect("fixture type identity");
            let mut self_type = CompilerArchitectureFact::Type {
                fact_id: String::new(),
                type_id: self_type_id.clone(),
                display: format!("{implementation_package}::RepositoryAdapter"),
                definition_path: Some(format!("{implementation_package}::RepositoryAdapter")),
                origin_crate: implementation_package.to_owned(),
                type_kind: octet_architecture_ir::TypeKind::Adt,
            };
            set_fact_id(&mut self_type);
            let mut implementation = CompilerArchitectureFact::Implementation {
                fact_id: String::new(),
                implementation_id: HASH_C.to_owned(),
                trait_id: None,
                trait_definition: Some("application::ports::Repository".to_owned()),
                self_type_id,
                span_id: implementation_span.span_id.clone(),
            };
            set_fact_id(&mut implementation);
            self.add_unit_facts(
                implementation_package,
                vec![implementation_span],
                vec![self_type, implementation],
                Vec::new(),
            );
        }

        fn add_unit_facts(
            &mut self,
            package: &str,
            spans: Vec<SourceSpan>,
            facts: Vec<CompilerArchitectureFact>,
            unknowns: Vec<UnknownCompilerFact>,
        ) {
            assert!(self.ir.units.iter().any(|unit| unit.package_name == package), "fixture unit");
            self.ir.spans.extend(spans);
            self.ir.facts.extend(facts);
            self.ir.unknowns.extend(unknowns);
            self.ir.spans.sort();
            self.ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
            self.ir.unknowns.sort();
        }
    }

    fn role_assignment(
        scope_kind: ArchitectureScopeKind,
        scope: &str,
        role: ArchitectureRole,
    ) -> ArchitectureRoleAssignment {
        ArchitectureRoleAssignment {
            scope_kind,
            scope: scope.to_owned(),
            role,
        }
    }

    fn allowed_roles(role: ArchitectureRole) -> Vec<ArchitectureRole> {
        match role {
            ArchitectureRole::DomainCore => vec![ArchitectureRole::DomainCore],
            ArchitectureRole::ApplicationShell => {
                vec![ArchitectureRole::ApplicationShell, ArchitectureRole::DomainCore]
            }
            ArchitectureRole::InboundAdapter => {
                vec![ArchitectureRole::InboundAdapter, ArchitectureRole::ApplicationShell]
            }
            ArchitectureRole::OutboundAdapter => {
                vec![ArchitectureRole::OutboundAdapter, ArchitectureRole::ApplicationShell]
            }
            ArchitectureRole::CompositionRoot => vec![
                ArchitectureRole::CompositionRoot,
                ArchitectureRole::ApplicationShell,
                ArchitectureRole::InboundAdapter,
                ArchitectureRole::OutboundAdapter,
                ArchitectureRole::DomainCore,
            ],
            ArchitectureRole::Test | ArchitectureRole::Generated => Vec::new(),
        }
    }

    fn edge(source_package: &str, target_package: &str) -> ArchitectureCargoEdge {
        ArchitectureCargoEdge {
            caller_package: source_package.to_owned(),
            callee_package: target_package.to_owned(),
            edge_kind: "normal".to_owned(),
            target_predicate: None,
        }
    }

    fn unit(package: &str, role: CompilerArchitectureRole) -> CompilationUnit {
        CompilationUnit {
            unit_id: octet_architecture_ir::canonical_identity("fixture-unit", &(package, role)).expect("unit id"),
            package_name: package.to_owned(),
            package_version: "0.1.0".to_owned(),
            target_name: package.to_owned(),
            target_kind: "lib".to_owned(),
            crate_types: vec!["rlib".to_owned()],
            target_triple: TARGET.to_owned(),
            features: Vec::new(),
            default_features: true,
            role,
            toolchain_blake3: HASH_B.to_owned(),
            collector_blake3: HASH_C.to_owned(),
            policy_blake3: HASH_A.to_owned(),
            source_blake3: octet_architecture_ir::canonical_identity("fixture-source", &package).expect("source id"),
            invocation_blake3: HASH_B.to_owned(),
        }
    }

    fn span(path: &str) -> SourceSpan {
        let mut span = SourceSpan {
            span_id: String::new(),
            origin: SourceOrigin::Workspace {
                source_path: path.to_owned(),
            },
            byte_start: 0,
            byte_end: 1,
            line_start: 1,
            column_start: 1,
            from_expansion: false,
        };
        span.span_id = octet_architecture_ir::source_span_identity(&span);
        span
    }

    fn item(span_id: &str, qualified_path: &str) -> CompilerArchitectureFact {
        let mut fact = CompilerArchitectureFact::Item {
            fact_id: String::new(),
            item_id: octet_architecture_ir::canonical_identity("fixture-item", &qualified_path).expect("item id"),
            qualified_path: qualified_path.to_owned(),
            item_kind: ItemKind::Function,
            origin_crate: qualified_path.split("::").next().unwrap_or(qualified_path).to_owned(),
            visibility: "public".to_owned(),
            signature_id: None,
            span_id: span_id.to_owned(),
        };
        set_fact_id(&mut fact);
        fact
    }

    fn mutate_witness_visibility(ir: &mut CompilerArchitectureIr, visibility: &str) {
        let fact = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::TypeShape { definition_path, .. }
                        if definition_path == "domain::PublicationAuthority"
                )
            })
            .expect("witness shape");
        let CompilerArchitectureFact::TypeShape {
            fact_id,
            field_visibilities,
            ..
        } = fact
        else {
            panic!("witness shape expected");
        };
        field_visibilities.clear();
        field_visibilities.push(visibility.to_owned());
        fact_id.clear();
        set_fact_id(fact);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn mutate_witness_shape(ir: &mut CompilerArchitectureIr, mutate: impl FnOnce(&mut Vec<String>, &mut Vec<String>)) {
        let fact = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::TypeShape { definition_path, .. }
                        if definition_path == "domain::PublicationAuthority"
                )
            })
            .expect("witness shape");
        let CompilerArchitectureFact::TypeShape {
            fact_id,
            field_type_definitions,
            implemented_traits,
            ..
        } = fact
        else {
            panic!("witness shape expected");
        };
        mutate(field_type_definitions, implemented_traits);
        fact_id.clear();
        set_fact_id(fact);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn mutate_witness_constructor(ir: &mut CompilerArchitectureIr, visibility: &str, is_unsafe: bool) {
        let constructor_item_id = ir
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id,
                    qualified_path,
                    ..
                } if qualified_path == "domain::PublicationAuthority::from_decision" => Some(item_id.clone()),
                _ => None,
            })
            .expect("witness constructor item");
        let item = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::Item { item_id, .. } if item_id == &constructor_item_id
                )
            })
            .expect("witness constructor item");
        set_item_visibility(item, visibility);
        let safety = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::FunctionSafety { item_id, .. } if item_id == &constructor_item_id
                )
            })
            .expect("witness constructor safety");
        let CompilerArchitectureFact::FunctionSafety {
            fact_id,
            is_unsafe: current,
            ..
        } = safety
        else {
            panic!("witness constructor safety expected");
        };
        *current = is_unsafe;
        fact_id.clear();
        set_fact_id(safety);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn remove_executor_provenance(ir: &mut CompilerArchitectureIr) {
        ir.facts.retain(|fact| {
            !matches!(
                fact,
                CompilerArchitectureFact::CallArgumentProvenance { callee_definition, .. }
                    if callee_definition == "outbound::FilesystemPort::publish"
            )
        });
    }

    fn remove_executor_witness_parameter(ir: &mut CompilerArchitectureIr) {
        let executor_item_id = ir
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id,
                    qualified_path,
                    ..
                } if qualified_path == "outbound::FilesystemPort::publish" => Some(item_id.clone()),
                _ => None,
            })
            .expect("protected executor item");
        let signature = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::Signature { owner_item_id, .. }
                        if owner_item_id == &executor_item_id
                )
            })
            .expect("protected executor signature");
        let CompilerArchitectureFact::Signature {
            fact_id,
            input_type_ids,
            ..
        } = signature
        else {
            panic!("protected executor signature expected");
        };
        input_type_ids.clear();
        fact_id.clear();
        set_fact_id(signature);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn mutate_receipt_provenance(
        ir: &mut CompilerArchitectureIr,
        mutate: impl FnOnce(&mut String, &mut Vec<String>, &mut ValueProvenanceStatus),
    ) {
        let fact = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::CallArgumentProvenance { callee_definition, .. }
                        if callee_definition == "application::PublicationReceipt::from_observation"
                )
            })
            .expect("success receipt provenance");
        let CompilerArchitectureFact::CallArgumentProvenance {
            fact_id,
            argument_type_id,
            origin_definitions,
            status,
            ..
        } = fact
        else {
            panic!("receipt provenance expected");
        };
        mutate(argument_type_id, origin_definitions, status);
        fact_id.clear();
        set_fact_id(fact);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn mutate_decision_output(ir: &mut CompilerArchitectureIr, decision: &str, output: &str) {
        let decision_item_id = ir
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Item {
                    item_id,
                    qualified_path,
                    ..
                } if qualified_path == decision => Some(item_id.clone()),
                _ => None,
            })
            .expect("decision item");
        let output_type_id = ir
            .facts
            .iter()
            .find_map(|fact| match fact {
                CompilerArchitectureFact::Type {
                    type_id,
                    definition_path: Some(definition),
                    ..
                } if definition == output => Some(type_id.clone()),
                _ => None,
            })
            .expect("output type");
        let signature = ir
            .facts
            .iter_mut()
            .find(|fact| {
                matches!(
                    fact,
                    CompilerArchitectureFact::Signature { owner_item_id, .. } if owner_item_id == &decision_item_id
                )
            })
            .expect("decision signature");
        let CompilerArchitectureFact::Signature {
            fact_id,
            output_type_id: current,
            ..
        } = signature
        else {
            panic!("decision signature expected");
        };
        *current = output_type_id;
        fact_id.clear();
        set_fact_id(signature);
        ir.facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    }

    fn set_item_visibility(fact: &mut CompilerArchitectureFact, visibility: &str) {
        let CompilerArchitectureFact::Item {
            fact_id,
            visibility: current,
            ..
        } = fact
        else {
            panic!("fixture item fact expected");
        };
        *current = visibility.to_owned();
        fact_id.clear();
        set_fact_id(fact);
    }

    fn type_fact(origin_crate: &str, definition_path: &str) -> CompilerArchitectureFact {
        let type_id = octet_architecture_ir::canonical_identity("fixture-type", &(origin_crate, definition_path))
            .expect("type id");
        let mut fact = CompilerArchitectureFact::Type {
            fact_id: String::new(),
            type_id,
            display: definition_path.to_owned(),
            definition_path: Some(definition_path.to_owned()),
            origin_crate: origin_crate.to_owned(),
            type_kind: octet_architecture_ir::TypeKind::Adt,
        };
        set_fact_id(&mut fact);
        fact
    }

    fn type_id(fact: &CompilerArchitectureFact) -> &str {
        let CompilerArchitectureFact::Type { type_id, .. } = fact else {
            panic!("fixture type fact expected");
        };
        type_id
    }

    fn item_id(fact: &CompilerArchitectureFact) -> &str {
        let CompilerArchitectureFact::Item { item_id, .. } = fact else {
            panic!("fixture item fact expected");
        };
        item_id
    }

    fn set_fact_id(fact: &mut CompilerArchitectureFact) {
        let identity = octet_architecture_ir::fact_identity(fact);
        match fact {
            CompilerArchitectureFact::Module { fact_id, .. }
            | CompilerArchitectureFact::Item { fact_id, .. }
            | CompilerArchitectureFact::Type { fact_id, .. }
            | CompilerArchitectureFact::Trait { fact_id, .. }
            | CompilerArchitectureFact::Implementation { fact_id, .. }
            | CompilerArchitectureFact::Signature { fact_id, .. }
            | CompilerArchitectureFact::Call { fact_id, .. }
            | CompilerArchitectureFact::Constructor { fact_id, .. }
            | CompilerArchitectureFact::Effect { fact_id, .. }
            | CompilerArchitectureFact::Expansion { fact_id, .. }
            | CompilerArchitectureFact::Configuration { fact_id, .. }
            | CompilerArchitectureFact::TypeShape { fact_id, .. }
            | CompilerArchitectureFact::FunctionSafety { fact_id, .. }
            | CompilerArchitectureFact::CallArgumentProvenance { fact_id, .. } => *fact_id = identity,
        }
    }

    fn limits() -> ArchitectureCollectionLimits {
        const FIXTURE_LIMIT: usize = 64;
        ArchitectureCollectionLimits {
            max_units: FIXTURE_LIMIT,
            max_shards: FIXTURE_LIMIT,
            max_modules: FIXTURE_LIMIT,
            max_items: FIXTURE_LIMIT,
            max_types: FIXTURE_LIMIT,
            max_traits: FIXTURE_LIMIT,
            max_implementations: FIXTURE_LIMIT,
            max_signatures: FIXTURE_LIMIT,
            max_calls: FIXTURE_LIMIT,
            max_constructors: FIXTURE_LIMIT,
            max_effects: FIXTURE_LIMIT,
            max_expansions: FIXTURE_LIMIT,
            max_configurations: FIXTURE_LIMIT,
            max_type_shapes: FIXTURE_LIMIT,
            max_function_safety_facts: FIXTURE_LIMIT,
            max_value_provenance_facts: FIXTURE_LIMIT,
            max_value_flow_bodies: FIXTURE_LIMIT,
            max_value_flow_steps: FIXTURE_LIMIT,
            max_value_flow_rounds: FIXTURE_LIMIT,
            max_unknowns: FIXTURE_LIMIT,
            max_spans: FIXTURE_LIMIT,
            max_span_bytes: FIXTURE_LIMIT,
            max_shard_bytes: FIXTURE_LIMIT * FIXTURE_LIMIT,
            max_artifact_bytes: FIXTURE_LIMIT * FIXTURE_LIMIT,
        }
    }
}
