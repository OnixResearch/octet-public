//! Check report and stable renderers.

use serde::Deserialize;
use serde::Serialize;

use crate::capability_manifest::CapabilityReport;
use crate::diagnostic::Diagnostic;
use crate::model::EnforcementMode;
use crate::model::Severity;
use crate::suppression::Suppression;

const EMPTY_REPORT_MARKDOWN: &str = "# Tiger Style Standards Report\n\nNo findings.\n";
const REPORT_HEADING: &str = "# Tiger Style Standards Report";
const FINDINGS_HEADING: &str = "## Findings";
const POLICY_EXECUTIONS_HEADING: &str = "## Policy Execution Coverage";
const SUPPRESSIONS_HEADING: &str = "## Active Suppressions";
const SUPPRESSED_FINDINGS_HEADING: &str = "## Suppressed Findings";
const CAPABILITY_REPORTS_HEADING: &str = "## Capability Manifest Reports";
const NO_SUPPRESSIONS_LINE: &str = "No active suppressions.";

/// Actual execution outcome for one policy checker.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyExecutionStatus {
    /// The checker ran on an applicable scope and emitted no diagnostic for the policy.
    Passed,
    /// The checker ran on an applicable scope and emitted at least one diagnostic for the policy.
    Failed,
    /// The checker was not executed for this policy.
    NotRun,
    /// No implemented checker evaluates this policy.
    Unsupported,
    /// The policy family was not selected or no applicable source scope exists.
    NotApplicable,
    /// The checker failed to execute, for example after a timeout or unparseable input.
    ToolError,
}

/// Ownership of the checker responsible for a policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyExecutionOwnership {
    /// Implemented by a local pure checker in this crate.
    Local,
    /// Owned by an external tool that this run does not execute.
    External,
    /// Requires a human review process instead of a machine check.
    HumanProcess,
}

/// Evidence that one selected policy was actually executed by a named checker.
///
/// Unlike [`CheckReport::evaluated_policy_ids`], this record is derived from checker execution
/// rather than registry selection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyExecution {
    /// Stable policy ID.
    pub policy_id: String,
    /// Registry revision (`schema_version`) that supplied the policy definition.
    ///
    /// The registry schema version is the revision available on the policy entry; it changes when
    /// the exported policy definitions change.
    pub policy_revision: String,
    /// Checker that evaluated or owns the policy.
    pub checker: String,
    /// Declared checker owner for the policy.
    pub checker_owner: String,
    /// Deterministic description of the selected family and source scope.
    pub selected_scope: String,
    /// Actual execution outcome.
    pub status: PolicyExecutionStatus,
    /// Why the status was assigned.
    pub coverage_explanation: String,
    /// Identities of diagnostics the checker emitted for this policy.
    pub linked_diagnostics: Vec<String>,
    /// Mechanism or checker identity that produced the coverage claim.
    pub evidence_identity: String,
    /// Whether the checker is local, external, or human-process owned.
    pub ownership: PolicyExecutionOwnership,
    /// True when the policy was selected and carries a hard enforcement level.
    pub required: bool,
}

/// Stable report from a Tiger Style standards checker run.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckReport {
    /// Diagnostics emitted by the run.
    pub diagnostics: Vec<Diagnostic>,
    /// Active suppressions accepted by the run.
    pub active_suppressions: Vec<Suppression>,
    /// Checker diagnostics an active suppression covers; they do not block.
    #[serde(default)]
    pub suppressed_diagnostics: Vec<Diagnostic>,
    /// Selected policy IDs from registry metadata.
    ///
    /// This list is registry selection metadata, not execution proof. Read
    /// [`Self::policy_executions`] for what a checker actually executed.
    pub evaluated_policy_ids: Vec<String>,
    /// Enforcement-mode metadata for selected policies.
    ///
    /// This list is migration metadata preserved for compatibility; it does not prove that a
    /// checker ran. Read [`Self::policy_executions`] for actual coverage.
    #[serde(default)]
    pub policy_evaluations: Vec<PolicyEvaluation>,
    /// Capability manifest reports emitted by architecture checks.
    #[serde(default)]
    pub capability_reports: Vec<CapabilityReport>,
    /// Actual execution coverage for every policy in the registry.
    #[serde(default)]
    pub policy_executions: Vec<PolicyExecution>,
    /// Selected hard policy IDs whose execution coverage this report must account for.
    #[serde(default)]
    pub required_policy_ids: Vec<String>,
}

/// Enforcement metadata emitted in machine-readable reports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyEvaluation {
    /// Policy ID.
    pub policy_id: String,
    /// Current enforcement mode.
    pub current_enforcement_mode: EnforcementMode,
    /// Target enforcement mode.
    pub target_enforcement_mode: EnforcementMode,
    /// Mechanism or fallback classification for failures.
    pub failure_type: String,
}

impl CheckReport {
    /// Returns true when this report should fail default CI mode.
    #[must_use]
    pub fn has_blocking_diagnostics(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_blocking)
    }

    /// Returns true when this report only has advisory or informational findings.
    #[must_use]
    pub fn is_advisory_only(&self) -> bool {
        !self.diagnostics.is_empty() && !self.has_blocking_diagnostics()
    }

    /// Returns true when every selected hard policy has an accounted-for execution outcome.
    ///
    /// Local checkers must report `passed`, `failed`, or `not_applicable`. External and
    /// human-process checkers must report `not_run`, because this run cannot complete them. A
    /// local `not_run`, `unsupported`, or `tool_error` outcome makes coverage incomplete.
    #[must_use]
    pub fn coverage_complete(&self) -> bool {
        self.required_policy_ids.iter().all(|policy_id| {
            self.policy_executions
                .iter()
                .filter(|execution| &execution.policy_id == policy_id)
                .any(PolicyExecution::is_accounted_for)
        }) && self
            .policy_executions
            .iter()
            .filter(|execution| execution.required)
            .all(PolicyExecution::is_accounted_for)
    }

    /// Returns true when strict policy coverage passes this report.
    ///
    /// Strict mode requires no blocking diagnostics, complete execution coverage for selected hard
    /// policies, and no selected hard policy that is left to an external or human-process checker
    /// with only a `not_run` outcome. Local `not_applicable` outcomes remain eligible because they
    /// carry an explicit reason.
    #[must_use]
    pub fn strict_passing(&self) -> bool {
        !self.has_blocking_diagnostics() && self.coverage_complete() && !self.required_checker_not_run()
    }

    fn required_checker_not_run(&self) -> bool {
        self.policy_executions.iter().any(|execution| {
            execution.required
                && execution.ownership != PolicyExecutionOwnership::Local
                && execution.status == PolicyExecutionStatus::NotRun
        })
    }
}

impl PolicyExecution {
    /// Returns true when this record proves a completed local checker outcome.
    #[must_use]
    pub fn is_completed_local_check(&self) -> bool {
        self.ownership == PolicyExecutionOwnership::Local
            && matches!(self.status, PolicyExecutionStatus::Passed | PolicyExecutionStatus::Failed)
    }

    /// Returns true when this execution record accounts for a selected hard policy.
    #[must_use]
    pub fn is_accounted_for(&self) -> bool {
        match self.ownership {
            PolicyExecutionOwnership::Local => matches!(
                self.status,
                PolicyExecutionStatus::Passed | PolicyExecutionStatus::Failed | PolicyExecutionStatus::NotApplicable
            ),
            PolicyExecutionOwnership::External | PolicyExecutionOwnership::HumanProcess => {
                self.status == PolicyExecutionStatus::NotRun
            }
        }
    }
}

/// Serializes a report as deterministic pretty JSON.
///
/// # Errors
///
/// Returns a serde error if the report cannot be serialized.
pub fn render_json(report: &CheckReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

/// Renders a report as deterministic markdown.
#[must_use]
pub fn render_markdown(report: &CheckReport) -> String {
    if report.diagnostics.is_empty()
        && report.active_suppressions.is_empty()
        && report.capability_reports.is_empty()
        && report.policy_executions.is_empty()
    {
        return EMPTY_REPORT_MARKDOWN.to_owned();
    }

    let mut markdown = String::new();
    append_heading(&mut markdown);
    append_diagnostics(&mut markdown, &report.diagnostics);
    append_policy_executions(&mut markdown, report);
    append_suppressions(&mut markdown, &report.active_suppressions);
    append_suppressed_diagnostics(&mut markdown, &report.suppressed_diagnostics);
    append_capability_reports(&mut markdown, &report.capability_reports);
    markdown
}

fn append_heading(markdown: &mut String) {
    markdown.push_str(REPORT_HEADING);
    markdown.push_str("\n\n");
}

fn append_diagnostics(markdown: &mut String, diagnostics: &[Diagnostic]) {
    markdown.push_str(FINDINGS_HEADING);
    markdown.push_str("\n\n");
    if diagnostics.is_empty() {
        markdown.push_str("No findings.\n\n");
        return;
    }

    for diagnostic in diagnostics {
        append_diagnostic(markdown, diagnostic);
    }
    markdown.push('\n');
}

fn append_diagnostic(markdown: &mut String, diagnostic: &Diagnostic) {
    let line = diagnostic.line.map_or_else(|| String::from("unknown"), |line| line.to_string());
    markdown.push_str("- [");
    markdown.push_str(severity_label(diagnostic.severity));
    markdown.push_str("] `");
    markdown.push_str(&diagnostic.policy_id);
    markdown.push_str("` at `");
    markdown.push_str(&diagnostic.path);
    markdown.push(':');
    markdown.push_str(&line);
    markdown.push_str("` — ");
    markdown.push_str(&diagnostic.message);
    markdown.push('\n');
    markdown.push_str("  - Source: ");
    markdown.push_str(&diagnostic.source_section);
    markdown.push('\n');
    markdown.push_str("  - Remediation: ");
    markdown.push_str(&diagnostic.remediation);
    markdown.push('\n');
}

fn append_policy_executions(markdown: &mut String, report: &CheckReport) {
    if report.policy_executions.is_empty() {
        return;
    }
    markdown.push_str(POLICY_EXECUTIONS_HEADING);
    markdown.push_str("\n\nCoverage complete: ");
    markdown.push_str(if report.coverage_complete() { "yes" } else { "no" });
    markdown.push_str(".\n\n");
    for execution in &report.policy_executions {
        markdown.push_str("- `");
        markdown.push_str(&execution.policy_id);
        markdown.push_str("` status=");
        markdown.push_str(execution_status_label(execution.status));
        markdown.push_str(" checker=");
        markdown.push_str(&execution.checker);
        markdown.push_str(" owner=");
        markdown.push_str(&execution.checker_owner);
        markdown.push_str(" ownership=");
        markdown.push_str(execution_ownership_label(execution.ownership));
        markdown.push_str(" required=");
        markdown.push_str(if execution.required { "yes" } else { "no" });
        markdown.push('\n');
        markdown.push_str("  - Scope: ");
        markdown.push_str(&execution.selected_scope);
        markdown.push('\n');
        markdown.push_str("  - Explanation: ");
        markdown.push_str(&execution.coverage_explanation);
        markdown.push('\n');
        markdown.push_str("  - Evidence: ");
        markdown.push_str(&execution.evidence_identity);
        markdown.push('\n');
        markdown.push_str("  - Linked diagnostics: ");
        markdown.push_str(&execution.linked_diagnostics.len().to_string());
        if !execution.linked_diagnostics.is_empty() {
            markdown.push_str(" (");
            markdown.push_str(&execution.linked_diagnostics.join(", "));
            markdown.push(')');
        }
        markdown.push('\n');
    }
}

fn execution_status_label(status: PolicyExecutionStatus) -> &'static str {
    match status {
        PolicyExecutionStatus::Passed => "passed",
        PolicyExecutionStatus::Failed => "failed",
        PolicyExecutionStatus::NotRun => "not_run",
        PolicyExecutionStatus::Unsupported => "unsupported",
        PolicyExecutionStatus::NotApplicable => "not_applicable",
        PolicyExecutionStatus::ToolError => "tool_error",
    }
}

fn execution_ownership_label(ownership: PolicyExecutionOwnership) -> &'static str {
    match ownership {
        PolicyExecutionOwnership::Local => "local",
        PolicyExecutionOwnership::External => "external",
        PolicyExecutionOwnership::HumanProcess => "human_process",
    }
}

fn append_suppressions(markdown: &mut String, suppressions: &[Suppression]) {
    markdown.push_str(SUPPRESSIONS_HEADING);
    markdown.push_str("\n\n");
    if suppressions.is_empty() {
        markdown.push_str(NO_SUPPRESSIONS_LINE);
        markdown.push('\n');
        return;
    }

    for suppression in suppressions {
        markdown.push_str("- `");
        markdown.push_str(&suppression.policy_id);
        markdown.push_str("` at `");
        markdown.push_str(&suppression.path);
        markdown.push(':');
        markdown.push_str(&suppression.line.to_string());
        markdown.push_str("` scope=");
        markdown.push_str(&suppression.scope);
        markdown.push('\n');
    }
}

fn append_suppressed_diagnostics(markdown: &mut String, diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        return;
    }
    markdown.push('\n');
    markdown.push_str(SUPPRESSED_FINDINGS_HEADING);
    markdown.push_str("\n\n");
    for diagnostic in diagnostics {
        append_diagnostic(markdown, diagnostic);
    }
}

fn append_capability_reports(markdown: &mut String, reports: &[CapabilityReport]) {
    if reports.is_empty() {
        return;
    }
    markdown.push('\n');
    markdown.push_str(CAPABILITY_REPORTS_HEADING);
    markdown.push_str("\n\n");
    for report in reports {
        append_capability_report(markdown, report);
    }
}

fn append_capability_report(markdown: &mut String, report: &CapabilityReport) {
    markdown.push_str("- `");
    markdown.push_str(&report.policy_id);
    markdown.push_str("` mode=");
    markdown.push_str(&report.enforcement_mode);
    markdown.push('\n');
    markdown.push_str("  - Scope classes: ");
    markdown.push_str(&report.declared_scope_classes.join(", "));
    markdown.push('\n');
    markdown.push_str("  - Capability families: ");
    markdown.push_str(&report.declared_capability_families.join(", "));
    markdown.push('\n');
    markdown.push_str("  - Observations: ");
    markdown.push_str(&report.observations.len().to_string());
    markdown.push('\n');
    for observation in &report.observations {
        append_capability_observation(markdown, observation);
    }
    markdown.push_str("  - Handler evidence: ");
    markdown.push_str(&report.handler_evidence.len().to_string());
    markdown.push('\n');
    for evidence in &report.handler_evidence {
        markdown.push_str("    - `");
        markdown.push_str(&evidence.handler);
        markdown.push_str("` artifact=`");
        markdown.push_str(&evidence.artifact);
        markdown.push_str("` replay=`");
        markdown.push_str(&evidence.replay_command);
        markdown.push_str("` blake3=");
        markdown.push_str(&evidence.blake3);
        markdown.push_str(" success=");
        markdown.push_str(&evidence.success_path_coverage.to_string());
        markdown.push_str(" failure=");
        markdown.push_str(&evidence.failure_path_coverage.to_string());
        markdown.push('\n');
    }
    markdown.push_str("  - Non-claims:\n");
    for non_claim in &report.non_claims {
        markdown.push_str("    - ");
        markdown.push_str(non_claim);
        markdown.push('\n');
    }
}

fn append_capability_observation(
    markdown: &mut String,
    observation: &crate::capability_manifest::CapabilityObservation,
) {
    let line = observation.line.map_or_else(|| String::from("unknown"), |line| line.to_string());
    markdown.push_str("    - ");
    markdown.push_str(&observation.status);
    markdown.push_str(" `");
    markdown.push_str(&observation.observation_kind);
    markdown.push_str("` ");
    markdown.push_str(&observation.capability_family);
    markdown.push_str(" at `");
    markdown.push_str(&observation.path);
    markdown.push(':');
    markdown.push_str(&line);
    markdown.push_str("` matched `");
    markdown.push_str(&observation.matched);
    markdown.push_str("` lint_ids=[");
    markdown.push_str(&observation.linked_lint_ids.join(", "));
    markdown.push_str("]\n");
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
        Severity::Note => "note",
    }
}
