//! Fixture manifest validation.

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::EnforcementLevel;
use crate::model::PolicyRegistry;
use crate::model::Severity;

const FIXTURE_POLICY_ID: &str = "tigerstyle.check.fixture-coverage";
const FIXTURE_SOURCE_SECTION: &str =
    "openspec/changes/codify-octet-standards/specs/standards-codification/spec.md#hard-rule-families";
const FIXTURE_PATH: &str = "fixture-manifest";
const MISSING_POLICY_REMEDIATION: &str = "add manifest coverage for the enabled hard policy";
const MISSING_COMPLIANT_REMEDIATION: &str = "add at least one compliant fixture reference";
const MISSING_NONCOMPLIANT_REMEDIATION: &str = "add at least one noncompliant fixture reference";

/// Fixture coverage manifest.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FixtureManifest {
    /// Per-policy fixture coverage entries.
    pub policy_coverage: Vec<PolicyFixtureCoverage>,
}

/// Fixture coverage for one policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyFixtureCoverage {
    /// Stable policy ID.
    pub policy_id: String,
    /// Passing fixture names or paths.
    pub compliant: Vec<String>,
    /// Failing fixture names or paths.
    pub noncompliant: Vec<String>,
}

/// Validates that every hard policy has compliant and noncompliant fixtures.
#[must_use]
pub fn validate_hard_policy_fixture_coverage(registry: &PolicyRegistry, manifest: &FixtureManifest) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for policy in &registry.policies {
        if policy.enforcement_level != EnforcementLevel::Hard {
            continue;
        }
        match coverage_for_policy(manifest, &policy.policy_id) {
            Some(coverage) => append_coverage_diagnostics(coverage, &mut diagnostics),
            None => diagnostics.push(missing_policy_diagnostic(&policy.policy_id)),
        }
    }
    diagnostics
}

fn coverage_for_policy<'a>(manifest: &'a FixtureManifest, policy_id: &str) -> Option<&'a PolicyFixtureCoverage> {
    manifest.policy_coverage.iter().find(|coverage| coverage.policy_id == policy_id)
}

fn append_coverage_diagnostics(coverage: &PolicyFixtureCoverage, diagnostics: &mut Vec<Diagnostic>) {
    if coverage.compliant.is_empty() {
        diagnostics.push(missing_compliant_diagnostic(&coverage.policy_id));
    }
    if coverage.noncompliant.is_empty() {
        diagnostics.push(missing_noncompliant_diagnostic(&coverage.policy_id));
    }
}

fn missing_policy_diagnostic(policy_id: &str) -> Diagnostic {
    fixture_diagnostic(policy_id, "hard policy is missing fixture coverage", MISSING_POLICY_REMEDIATION)
}

fn missing_compliant_diagnostic(policy_id: &str) -> Diagnostic {
    fixture_diagnostic(policy_id, "hard policy has no compliant fixture", MISSING_COMPLIANT_REMEDIATION)
}

fn missing_noncompliant_diagnostic(policy_id: &str) -> Diagnostic {
    fixture_diagnostic(policy_id, "hard policy has no noncompliant fixture", MISSING_NONCOMPLIANT_REMEDIATION)
}

fn fixture_diagnostic(policy_id: &str, message: &str, remediation: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: format!("{FIXTURE_POLICY_ID}:{policy_id}"),
        source_section: FIXTURE_SOURCE_SECTION.to_owned(),
        path: FIXTURE_PATH.to_owned(),
        line: None,
        severity: Severity::Error,
        message: message.to_owned(),
        remediation: remediation.to_owned(),
    })
}
