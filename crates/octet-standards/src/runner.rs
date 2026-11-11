//! Pure checker runner.

use serde::Deserialize;
use serde::Serialize;

use crate::architecture::ARCHITECTURE_CHECKED_POLICY_IDS;
use crate::architecture::architecture_applicable_scope;
use crate::architecture::check_architecture;
use crate::capability_manifest::capability_manifest_checker_applies;
use crate::capability_manifest::check_capability_manifest;
use crate::diagnostic::Diagnostic;
use crate::model::EnforcementLevel;
use crate::model::FamilySelection;
use crate::model::MechanismKind;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::proof::check_verified_proof_rails;
use crate::proof::policy_is_verified;
use crate::proof::proof_rail_checker_applies;
// r[impl octet.runner_self_check.warning_budget.reduction.standards_resource]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_report]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation]
use crate::report::PolicyExecution;
use crate::report::PolicyExecutionOwnership;
use crate::report::PolicyExecutionStatus;
use crate::source::SourceFile;
use crate::suppression::apply_suppressions;
use crate::suppression::validate_suppressions;

const LOCAL_CHECKER_ID: &str = "octet-standards";
const HUMAN_CHECKER_ID: &str = "human-process";
const SUPPRESSION_CHECKER: &str = "suppression";
const ARCHITECTURE_CHECKER: &str = "architecture";
const CAPABILITY_MANIFEST_CHECKER: &str = "capability-manifest";
const RESOURCE_CHECKER: &str = "resource";
const PROOF_RAIL_CHECKER: &str = "proof-rails";
const SUPPRESSION_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
const BOUNDED_QUEUE_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
const SUPPRESSION_SCOPE: &str = "suppression-markers";
const RESOURCE_SCOPE: &str = "embedded-bounded-primitives";
const CAPABILITY_MANIFEST_SCOPE: &str = "capability-manifest";
const PROOF_RAIL_SCOPE: &str = "proof-rails";
const UNSELECTED_SCOPE: &str = "unselected";
const UNSUPPORTED_SCOPE: &str = "not-implemented";
const EXTERNAL_SCOPE: &str = "external-checker";
const HUMAN_SCOPE: &str = "human-review";
const NO_SCOPE: &str = "none";

/// Classified failure from a policy checker execution.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerFailureKind {
    /// The checker process returned an execution error.
    ExecutionError,
    /// The checker exceeded its execution deadline.
    Timeout,
    /// The checker could not parse its input or output.
    ParseFailure,
}

/// A checker execution failure supplied to a run.
///
/// Failures are recorded against local checkers (`suppression`, `architecture`,
/// `capability-manifest`, `resource`, `proof-rails`). Externally owned checkers are never executed
/// by this crate, so they remain `not_run` even when a failure record is supplied.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckerFailure {
    /// Local checker name the failure belongs to.
    pub checker: String,
    /// Failure classification.
    pub kind: CheckerFailureKind,
    /// Policy IDs this failure applies to. Empty applies to every policy the checker evaluates.
    #[serde(default)]
    pub policy_ids: Vec<String>,
    /// Human-readable failure detail.
    pub message: String,
}

/// Inputs for a pure checker run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckInput {
    /// Policy registry.
    pub registry: PolicyRegistry,
    /// Source files to inspect.
    pub sources: Vec<SourceFile>,
    /// Selected rule families. Empty means every family.
    pub selection: FamilySelection,
    /// Checker execution failures supplied by the caller. Empty by default.
    pub checker_failures: Vec<CheckerFailure>,
}

/// Runs pure checker logic.
///
/// Active suppressions remove the checker diagnostics they cover into
/// `suppressed_diagnostics`; suppression-validation diagnostics are never suppressed.
#[must_use]
pub fn run_checks(input: &CheckInput) -> crate::report::CheckReport {
    let suppression_report = validate_suppressions(&input.registry, &input.sources, &input.selection);
    let mut checker_diagnostics = check_architecture(&input.registry, &input.sources, &input.selection);
    let capability_manifest = check_capability_manifest(&input.registry, &input.sources, &input.selection);
    checker_diagnostics.extend(capability_manifest.diagnostics);
    checker_diagnostics.extend(crate::resource::check_resources(&input.registry, &input.sources, &input.selection));
    checker_diagnostics.extend(check_verified_proof_rails(&input.registry, &input.sources, &input.selection));
    let checker_diagnostics = apply_suppressions(checker_diagnostics, &suppression_report.active_suppressions);
    let policy_executions = build_policy_executions(
        input,
        &suppression_report.diagnostics,
        &checker_diagnostics.remaining,
        &checker_diagnostics.suppressed,
    );
    let required_policy_ids = required_policy_ids(&input.registry, &input.selection);
    let mut diagnostics = suppression_report.diagnostics;
    diagnostics.extend(checker_diagnostics.remaining);
    let evaluated_policy_ids = evaluated_policy_ids(&input.registry, &input.selection);
    let policy_evaluations = policy_evaluations(&input.registry, &input.selection);

    crate::report::CheckReport {
        diagnostics,
        active_suppressions: suppression_report.active_suppressions,
        suppressed_diagnostics: checker_diagnostics.suppressed,
        evaluated_policy_ids,
        policy_evaluations,
        capability_reports: capability_manifest.reports,
        policy_executions,
        required_policy_ids,
    }
}

/// Coverage plan for one selected policy.
enum CoveragePlan {
    /// A local checker evaluates the policy.
    Local(LocalPlan),
    /// The registry assigns the local checker ID, but no local checker implements the policy.
    Unsupported,
    /// An external checker owns the policy and this run does not execute it.
    External,
    /// A human review process owns the policy.
    Human,
}

/// Local checker execution metadata for one policy.
struct LocalPlan {
    /// Local checker name.
    checker: &'static str,
    /// Applicable scope label, or `None` when no source scope applies.
    scope: Option<&'static str>,
    /// Explanation used when `scope` is `None`.
    unavailable_reason: String,
    /// Checker or mechanism identity for the coverage claim.
    evidence_identity: String,
}

fn required_policy_ids(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<String> {
    let mut policy_ids = registry
        .policies
        .iter()
        .filter(|policy| policy_required(policy, selection))
        .map(|policy| policy.policy_id.clone())
        .collect::<Vec<_>>();
    policy_ids.sort_unstable();
    policy_ids.dedup();
    policy_ids
}

fn policy_required(policy: &PolicyEntry, selection: &FamilySelection) -> bool {
    selection.includes(&policy.pillar) && policy.enforcement_level == EnforcementLevel::Hard
}

fn build_policy_executions(
    input: &CheckInput,
    suppression_diagnostics: &[Diagnostic],
    checker_diagnostics: &[Diagnostic],
    suppressed_diagnostics: &[Diagnostic],
) -> Vec<PolicyExecution> {
    input
        .registry
        .policies
        .iter()
        .map(|policy| {
            if !input.selection.includes(&policy.pillar) {
                return unselected_execution(input, policy);
            }
            selected_execution(input, policy, suppression_diagnostics, checker_diagnostics, suppressed_diagnostics)
        })
        .collect()
}

fn unselected_execution(input: &CheckInput, policy: &PolicyEntry) -> PolicyExecution {
    let checker = declared_checker(policy);
    PolicyExecution {
        policy_id: policy.policy_id.clone(),
        policy_revision: input.registry.schema_version.clone(),
        checker: checker.clone(),
        checker_owner: policy.checker_owner.clone(),
        selected_scope: selected_scope(input, policy, UNSELECTED_SCOPE),
        status: PolicyExecutionStatus::NotApplicable,
        coverage_explanation: format!("policy family `{}` was not selected for this run", policy.pillar),
        linked_diagnostics: Vec::new(),
        evidence_identity: format!("checker={checker}"),
        ownership: policy_ownership(policy),
        required: false,
    }
}

fn selected_execution(
    input: &CheckInput,
    policy: &PolicyEntry,
    suppression_diagnostics: &[Diagnostic],
    checker_diagnostics: &[Diagnostic],
    suppressed_diagnostics: &[Diagnostic],
) -> PolicyExecution {
    let plan = coverage_plan(&input.registry, &input.sources, policy);
    let ownership = plan_ownership(&plan);
    let linked_diagnostics =
        linked_diagnostic_identities(policy, suppression_diagnostics, checker_diagnostics, suppressed_diagnostics);
    let suppressed_count =
        suppressed_diagnostics.iter().filter(|diagnostic| diagnostic.policy_id == policy.policy_id).count();
    let (checker, status, coverage_explanation, evidence_identity, scope) = match plan {
        CoveragePlan::Local(local) => local_execution(input, policy, &local, &linked_diagnostics, suppressed_count),
        CoveragePlan::Unsupported => (
            policy.checker_id.clone(),
            PolicyExecutionStatus::Unsupported,
            format!(
                "registry assigns checker `{}`, but no implemented local checker evaluates this policy",
                policy.checker_id
            ),
            format!("checker={}; implementation=none", policy.checker_id),
            UNSUPPORTED_SCOPE,
        ),
        CoveragePlan::External => (
            policy.checker_id.clone(),
            PolicyExecutionStatus::NotRun,
            format!(
                "externally owned checker `{}` is not executed by octet-standards; run it separately",
                policy.checker_id
            ),
            format!("checker={}", policy.checker_id),
            EXTERNAL_SCOPE,
        ),
        CoveragePlan::Human => (
            HUMAN_CHECKER_ID.to_owned(),
            PolicyExecutionStatus::NotRun,
            "human review process required; this is not a completed local check".to_owned(),
            format!("checker={HUMAN_CHECKER_ID}"),
            HUMAN_SCOPE,
        ),
    };
    PolicyExecution {
        policy_id: policy.policy_id.clone(),
        policy_revision: input.registry.schema_version.clone(),
        checker,
        checker_owner: policy.checker_owner.clone(),
        selected_scope: selected_scope(input, policy, scope),
        status,
        coverage_explanation,
        linked_diagnostics,
        evidence_identity,
        ownership,
        required: policy_required(policy, &input.selection),
    }
}

fn local_execution(
    input: &CheckInput,
    policy: &PolicyEntry,
    local: &LocalPlan,
    linked_diagnostics: &[String],
    suppressed_count: usize,
) -> (String, PolicyExecutionStatus, String, String, &'static str) {
    let checker = local.checker.to_owned();
    let evidence_identity = local.evidence_identity.clone();
    let Some(scope) = local.scope else {
        return (
            checker,
            PolicyExecutionStatus::NotApplicable,
            local.unavailable_reason.clone(),
            evidence_identity,
            NO_SCOPE,
        );
    };
    if let Some(failure) = checker_failure(input, local.checker, &policy.policy_id) {
        return (
            checker,
            PolicyExecutionStatus::ToolError,
            format!(
                "{} checker did not complete: {} ({})",
                local.checker,
                failure_kind_label(failure.kind),
                failure.message
            ),
            evidence_identity,
            scope,
        );
    }
    if linked_diagnostics.is_empty() {
        (
            checker,
            PolicyExecutionStatus::Passed,
            format!(
                "{} checker ran on {scope} scope over {} supplied source file(s) and emitted no diagnostic",
                local.checker,
                input.sources.len()
            ),
            evidence_identity,
            scope,
        )
    } else {
        (
            checker,
            PolicyExecutionStatus::Failed,
            format!(
                "{} checker ran on {scope} scope and emitted {} diagnostic(s) ({} suppressed by active suppressions)",
                local.checker,
                linked_diagnostics.len(),
                suppressed_count
            ),
            evidence_identity,
            scope,
        )
    }
}

fn coverage_plan(registry: &PolicyRegistry, sources: &[SourceFile], policy: &PolicyEntry) -> CoveragePlan {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return CoveragePlan::Human;
    }
    if let Some(local) = local_plan(registry, sources, policy) {
        return CoveragePlan::Local(local);
    }
    if policy.checker_id == LOCAL_CHECKER_ID {
        return CoveragePlan::Unsupported;
    }
    CoveragePlan::External
}

fn local_plan(registry: &PolicyRegistry, sources: &[SourceFile], policy: &PolicyEntry) -> Option<LocalPlan> {
    if policy.policy_id == SUPPRESSION_POLICY_ID {
        return Some(LocalPlan {
            checker: SUPPRESSION_CHECKER,
            scope: (!sources.is_empty()).then_some(SUPPRESSION_SCOPE),
            unavailable_reason: no_source_scope_reason(SUPPRESSION_CHECKER, sources.len()),
            evidence_identity: format!("checker={SUPPRESSION_CHECKER}"),
        });
    }
    if policy.policy_id == BOUNDED_QUEUE_POLICY_ID {
        return Some(LocalPlan {
            checker: RESOURCE_CHECKER,
            scope: (!sources.is_empty()).then_some(RESOURCE_SCOPE),
            unavailable_reason: no_source_scope_reason(RESOURCE_CHECKER, sources.len()),
            evidence_identity: format!("checker={RESOURCE_CHECKER}; mechanism=bounded-primitive"),
        });
    }
    if ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str()) {
        if capability_manifest_checker_applies(policy) {
            return Some(LocalPlan {
                checker: CAPABILITY_MANIFEST_CHECKER,
                scope: (!sources.is_empty()).then_some(CAPABILITY_MANIFEST_SCOPE),
                unavailable_reason: no_source_scope_reason(CAPABILITY_MANIFEST_CHECKER, sources.len()),
                evidence_identity: format!("checker={CAPABILITY_MANIFEST_CHECKER}; mechanism=capability-api"),
            });
        }
        return Some(LocalPlan {
            checker: ARCHITECTURE_CHECKER,
            scope: architecture_applicable_scope(registry, sources, policy),
            unavailable_reason: format!(
                "no applicable architecture source scope among {} supplied source file(s)",
                sources.len()
            ),
            evidence_identity: architecture_evidence_identity(policy),
        });
    }
    if policy_is_verified(policy) {
        return Some(LocalPlan {
            checker: PROOF_RAIL_CHECKER,
            scope: proof_rail_checker_applies(policy, sources).then_some(PROOF_RAIL_SCOPE),
            unavailable_reason: format!("no declared proof rail matches the {} supplied source file(s)", sources.len()),
            evidence_identity: format!("checker={PROOF_RAIL_CHECKER}; rails={}", policy.proof_rails.len()),
        });
    }
    None
}

fn no_source_scope_reason(checker: &str, source_count: usize) -> String {
    format!("no applicable {checker} source scope among {source_count} supplied source file(s)")
}

fn architecture_evidence_identity(policy: &PolicyEntry) -> String {
    if policy.mechanisms.is_empty() {
        return format!("checker={ARCHITECTURE_CHECKER}");
    }
    let mechanisms = policy.mechanisms.iter().map(|mechanism| mechanism_kind_label(mechanism.kind)).collect::<Vec<_>>();
    format!("checker={ARCHITECTURE_CHECKER}; mechanism={}", mechanisms.join("+"))
}

fn mechanism_kind_label(kind: MechanismKind) -> &'static str {
    match kind {
        MechanismKind::CrateBoundary => "crate-boundary",
        MechanismKind::CapabilityApi => "capability-api",
        MechanismKind::BoundedPrimitive => "bounded-primitive",
        MechanismKind::TypeState => "type-state",
        MechanismKind::Port => "port",
    }
}

fn checker_failure<'a>(input: &'a CheckInput, checker: &str, policy_id: &str) -> Option<&'a CheckerFailure> {
    input.checker_failures.iter().find(|failure| {
        failure.checker == checker
            && (failure.policy_ids.is_empty() || failure.policy_ids.iter().any(|id| id == policy_id))
    })
}

fn failure_kind_label(kind: CheckerFailureKind) -> &'static str {
    match kind {
        CheckerFailureKind::ExecutionError => "execution-error",
        CheckerFailureKind::Timeout => "timeout",
        CheckerFailureKind::ParseFailure => "parse-failure",
    }
}

fn linked_diagnostic_identities(
    policy: &PolicyEntry,
    suppression_diagnostics: &[Diagnostic],
    checker_diagnostics: &[Diagnostic],
    suppressed_diagnostics: &[Diagnostic],
) -> Vec<String> {
    suppression_diagnostics
        .iter()
        .chain(checker_diagnostics)
        .chain(suppressed_diagnostics)
        .filter(|diagnostic| diagnostic.policy_id == policy.policy_id)
        .map(diagnostic_identity)
        .collect()
}

fn diagnostic_identity(diagnostic: &Diagnostic) -> String {
    diagnostic
        .line
        .map_or_else(|| format!("{}:?", diagnostic.path), |line| format!("{}:{}", diagnostic.path, line))
}

fn selected_scope(input: &CheckInput, policy: &PolicyEntry, scope: &str) -> String {
    let family = if input.selection.includes_all() {
        "all"
    } else {
        policy.pillar.as_str()
    };
    format!("family={family}; sources={}; scope={scope}", input.sources.len())
}

fn declared_checker(policy: &PolicyEntry) -> String {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return HUMAN_CHECKER_ID.to_owned();
    }
    if policy.policy_id == BOUNDED_QUEUE_POLICY_ID {
        return RESOURCE_CHECKER.to_owned();
    }
    if ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str())
        && capability_manifest_checker_applies(policy)
    {
        return CAPABILITY_MANIFEST_CHECKER.to_owned();
    }
    if ARCHITECTURE_CHECKED_POLICY_IDS.contains(&policy.policy_id.as_str()) {
        return ARCHITECTURE_CHECKER.to_owned();
    }
    if policy_is_verified(policy) {
        return PROOF_RAIL_CHECKER.to_owned();
    }
    policy.checker_id.clone()
}

fn policy_ownership(policy: &PolicyEntry) -> PolicyExecutionOwnership {
    if policy.checker_id == HUMAN_CHECKER_ID {
        return PolicyExecutionOwnership::HumanProcess;
    }
    // A registry checker ID can lag the local runner: the resource checker evaluates
    // `bounded-queues` and the proof-rail checker evaluates verified policies with rails even when
    // the registry still names an external owner. Actual local execution wins over the declaration.
    if policy.checker_id == LOCAL_CHECKER_ID
        || policy.policy_id == BOUNDED_QUEUE_POLICY_ID
        || policy_is_verified(policy)
    {
        return PolicyExecutionOwnership::Local;
    }
    PolicyExecutionOwnership::External
}

fn plan_ownership(plan: &CoveragePlan) -> PolicyExecutionOwnership {
    match plan {
        CoveragePlan::Local(_) | CoveragePlan::Unsupported => PolicyExecutionOwnership::Local,
        CoveragePlan::External => PolicyExecutionOwnership::External,
        CoveragePlan::Human => PolicyExecutionOwnership::HumanProcess,
    }
}

fn evaluated_policy_ids(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<String> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .map(|policy| policy.policy_id.clone())
        .collect()
}

fn policy_evaluations(registry: &PolicyRegistry, selection: &FamilySelection) -> Vec<crate::report::PolicyEvaluation> {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .map(|policy| crate::report::PolicyEvaluation {
            policy_id: policy.policy_id.clone(),
            current_enforcement_mode: policy.current_enforcement_mode,
            target_enforcement_mode: policy.target_enforcement_mode,
            failure_type: policy_failure_type(policy),
        })
        .collect()
}

fn policy_failure_type(policy: &crate::model::PolicyEntry) -> String {
    policy.mechanisms.first().map_or_else(
        || match policy.target_enforcement_mode {
            crate::model::EnforcementMode::Lint => "lint".to_owned(),
            crate::model::EnforcementMode::Gate => "gate".to_owned(),
            crate::model::EnforcementMode::Structural => "structural-missing-mechanism".to_owned(),
            crate::model::EnforcementMode::Verified => "verified-proof-rail".to_owned(),
            crate::model::EnforcementMode::HumanOracle => "human-oracle".to_owned(),
        },
        |mechanism| match mechanism.kind {
            crate::model::MechanismKind::CrateBoundary => "crate-boundary".to_owned(),
            crate::model::MechanismKind::CapabilityApi => "capability-api".to_owned(),
            crate::model::MechanismKind::BoundedPrimitive => "bounded-primitive".to_owned(),
            crate::model::MechanismKind::TypeState => "type-state".to_owned(),
            crate::model::MechanismKind::Port => "port".to_owned(),
        },
    )
}
