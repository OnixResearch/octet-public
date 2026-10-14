//! Pure checker runner.

use crate::architecture::check_architecture;
use crate::capability_manifest::check_capability_manifest;
use crate::model::FamilySelection;
use crate::model::PolicyRegistry;
use crate::proof::check_verified_proof_rails;
// r[impl octet.runner_self_check.warning_budget.reduction.standards_resource]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_report]
// r[impl octet.runner_self_check.warning_budget.reduction.standards_policy_evaluation]
use crate::source::SourceFile;
use crate::suppression::validate_suppressions;

/// Inputs for a pure checker run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckInput {
    /// Policy registry.
    pub registry: PolicyRegistry,
    /// Source files to inspect.
    pub sources: Vec<SourceFile>,
    /// Selected rule families. Empty means every family.
    pub selection: FamilySelection,
}

/// Runs pure checker logic.
#[must_use]
pub fn run_checks(input: &CheckInput) -> crate::report::CheckReport {
    let suppression_report = validate_suppressions(&input.registry, &input.sources, &input.selection);
    let mut diagnostics = suppression_report.diagnostics;
    diagnostics.extend(check_architecture(&input.registry, &input.sources, &input.selection));
    let capability_manifest = check_capability_manifest(&input.registry, &input.sources, &input.selection);
    diagnostics.extend(capability_manifest.diagnostics);
    diagnostics.extend(crate::resource::check_resources(&input.registry, &input.sources, &input.selection));
    diagnostics.extend(check_verified_proof_rails(&input.registry, &input.sources, &input.selection));
    let evaluated_policy_ids = evaluated_policy_ids(&input.registry, &input.selection);
    let policy_evaluations = policy_evaluations(&input.registry, &input.selection);

    crate::report::CheckReport {
        diagnostics,
        active_suppressions: suppression_report.active_suppressions,
        evaluated_policy_ids,
        policy_evaluations,
        capability_reports: capability_manifest.reports,
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
