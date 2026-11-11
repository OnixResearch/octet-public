//! Cross-tool architecture, assurance, and adoption experiment contract.
//!
//! The existing `benchmark` module keeps the strict same-command performance
//! comparator. This module is a separate competitive experiment: it consumes a
//! predeclared Nickel-authored profile, retained observations, and an explicit
//! applicability matrix. It never turns unsupported scope into clean evidence.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const PROFILES_SCHEMA: &str = "octet-benchmark-experiment-profiles/v1";
pub const OBSERVATIONS_SCHEMA: &str = "octet-benchmark-experiment-observations/v1";
pub const ARTIFACT_SCHEMA: &str = "octet-benchmark-experiment/v1";
pub const PROFILE_ID: &str = "octet-cross-tool-architecture-v1";
pub const ARTIFACT_NAME: &str = "benchmark-experiment.json";
pub const OBSERVATIONS_NAME: &str = "benchmark-experiment-observations.json";
pub const REPORT_NAME: &str = "benchmark-experiment-report.md";
pub const REQUIRED_NON_CLAIM: &str = "The experiment is bounded fixture evidence; it does not prove correctness, universal detection, complete Rust coverage, or release eligibility.";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentRegistry {
    pub schema_version: String,
    pub profiles: Vec<ExperimentProfile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Architecture,
    Assurance,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Variant {
    Compliant,
    Violating,
    Control,
    EmptyScope,
    Disputed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedOutcome {
    Finding,
    Clean,
    EmptyScope,
    Disputed,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityStatus {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSemantics {
    Recorded,
    Probe,
    Invoke,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParserKind {
    None,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationStatus {
    CheckedIn,
    UpstreamArgs,
    NotInstalled,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseScope {
    pub selected_sources: Vec<String>,
    pub expected_analyzed_sources: usize,
    pub empty: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentCase {
    pub case_id: String,
    pub task_id: String,
    pub category: Category,
    pub variant: Variant,
    pub source_path: String,
    pub source_identity: String,
    pub scope: CaseScope,
    pub expected: ExpectedOutcome,
    pub rationale: String,
    pub reviewer_role: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentTask {
    pub task_id: String,
    pub category: Category,
    pub description: String,
    pub comparable_subset: String,
    pub required_scope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPin {
    pub tool_id: String,
    pub kind: String,
    pub executable: String,
    pub version_pin: String,
    pub source_pin: String,
    pub config_path: Vec<String>,
    pub config_identity: Vec<String>,
    pub configuration_status: ConfigurationStatus,
    pub rules_pin: String,
    pub probe_argv: Vec<String>,
    pub run_argv: Vec<String>,
    pub execution: ExecutionSemantics,
    pub parser: ParserKind,
    pub setup_source: String,
    pub setup_steps: Vec<String>,
    pub required_config_files: Vec<String>,
    pub claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Applicability {
    pub task_id: String,
    pub tool_id: String,
    pub status: ApplicabilityStatus,
    pub case_ids: Vec<String>,
    pub comparable_subset: String,
    pub scope: String,
    pub rationale: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionTarget {
    pub selection_id: String,
    pub source_repository: String,
    pub source_revision: String,
    pub workspace_path: String,
    pub immutable: bool,
    pub notes: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionToolPlan {
    pub tool_id: String,
    pub setup_steps: Vec<String>,
    pub required_config_files: Vec<String>,
    pub initial_command: Vec<String>,
    pub incremental_command: Vec<String>,
    pub unavailable_metrics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionPlan {
    pub clean_consumer: AdoptionTarget,
    pub real_consumer: AdoptionTarget,
    pub tools: Vec<AdoptionToolPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SamplePlan {
    pub warm_state: WarmState,
    pub cache_state: CacheState,
    pub target: String,
    pub sample_count: usize,
    pub timeout_ms: u64,
    pub environment_facts: Vec<String>,
    pub dependency_differences: Vec<String>,
    pub unavailable_metrics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeControlDeclaration {
    pub control_id: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageBoundary {
    pub corpus_id: String,
    pub bounded: bool,
    pub complete_rust_coverage: bool,
    pub statement: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentProfile {
    pub profile_id: String,
    pub source_revision: String,
    pub rust_toolchain: String,
    pub evidence_tier: String,
    pub declared_before_measurement: bool,
    pub corpus: Vec<ExperimentCase>,
    pub tasks: Vec<ExperimentTask>,
    pub tools: Vec<ToolPin>,
    pub applicability: Vec<Applicability>,
    pub adoption: AdoptionPlan,
    pub sample_plan: SamplePlan,
    pub negative_controls: Vec<NegativeControlDeclaration>,
    pub coverage_boundary: CoverageBoundary,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarmState {
    Cold,
    Warm,
    NoChange,
    Incremental,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheState {
    Cold,
    Warm,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationStatus {
    Completed,
    TimedOut,
    Failed,
    Unavailable,
    Unsupported,
    NotRun,
    Unparsed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompileStatus {
    Compiled,
    NotCompiled,
    NotApplicable,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementOrigin {
    Local,
    RecordedFixture,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Local,
    Recorded,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationEnvironment {
    pub target: String,
    pub host_os: String,
    pub host_arch: String,
    pub rust_toolchain: String,
    pub cache_state: CacheState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationScope {
    pub selected_sources: Vec<String>,
    pub analyzed_sources: Vec<String>,
    pub scope_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedFinding {
    pub case_id: String,
    pub rule_id: String,
    pub path: String,
    pub line: u32,
    pub message: String,
    pub suppressed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedSuppression {
    pub case_id: String,
    pub rule_id: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolObservation {
    pub observation_id: String,
    pub tool_id: String,
    pub task_id: String,
    pub case_id: Option<String>,
    pub source_identity: String,
    pub tool_version: String,
    pub origin: MeasurementOrigin,
    pub environment: ObservationEnvironment,
    pub scope: ObservationScope,
    pub status: ObservationStatus,
    pub compile_status: CompileStatus,
    pub findings: Vec<ObservedFinding>,
    pub suppressions: Vec<ObservedSuppression>,
    pub stdout: String,
    pub stderr: String,
    pub stdout_identity: String,
    pub stderr_identity: String,
    pub exit_code: Option<i32>,
    pub wall_duration_ns: Option<u64>,
    pub warm_state: WarmState,
    pub sample_index: u32,
    pub failure_reason: Option<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAvailability {
    pub tool_id: String,
    pub status: Availability,
    pub version_output: String,
    pub version_identity: String,
    pub reason: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdoptionPhase {
    Initial,
    Incremental,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionObservation {
    pub observation_id: String,
    pub tool_id: String,
    pub target_selection: String,
    pub phase: AdoptionPhase,
    pub origin: MeasurementOrigin,
    pub measured: bool,
    pub wall_duration_ns: Option<u64>,
    pub warm_state: WarmState,
    pub environment: ObservationEnvironment,
    pub stdout: String,
    pub stderr: String,
    pub stdout_identity: String,
    pub stderr_identity: String,
    pub exit_code: Option<i32>,
    pub unavailable_reason: Option<String>,
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentObservations {
    pub schema_version: String,
    pub profile_id: String,
    pub tool_availability: Vec<ToolAvailability>,
    pub observations: Vec<ToolObservation>,
    pub adoption: Vec<AdoptionObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolTaskMetrics {
    pub category: Category,
    pub task_id: String,
    pub tool_id: String,
    pub applicability_status: ApplicabilityStatus,
    pub comparable_subset: String,
    pub availability: Availability,
    pub comparable_cases: u64,
    pub true_positive: u64,
    pub false_positive: u64,
    pub false_negative: u64,
    pub true_negative: u64,
    pub unsupported_cases: u64,
    pub inconclusive_cases: u64,
    pub incomparable_cases: u64,
    pub suppressed_cases: u64,
    pub empty_scope_cases: u64,
    pub disputed_cases: u64,
    pub missing_execution_cases: u64,
    pub precision_basis_points: Option<u64>,
    pub recall_basis_points: Option<u64>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdoptionSummary {
    pub tool_id: String,
    pub availability: Availability,
    pub setup_steps: u64,
    pub config_files: u64,
    pub clean_initial_ns: Option<u64>,
    pub clean_incremental_ns: Option<u64>,
    pub real_initial_ns: Option<u64>,
    pub real_incremental_ns: Option<u64>,
    pub incremental_over_initial_basis_points: Option<u64>,
    pub measured_targets: Vec<String>,
    pub unavailable_metrics: Vec<String>,
    pub unavailable_reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvenanceSummary {
    pub executed_locally: Vec<String>,
    pub recorded_fixtures: Vec<String>,
    pub unavailable: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedProjection {
    pub architecture: Vec<ToolTaskMetrics>,
    pub assurance: Vec<ToolTaskMetrics>,
    pub adoption: Vec<AdoptionSummary>,
    pub provenance: ProvenanceSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ControlOutcome {
    Contained,
    Escaped,
    NotRun,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NegativeControlRecord {
    pub control_id: String,
    pub outcome: ControlOutcome,
    pub evidence: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentArtifact {
    pub schema_version: String,
    pub profile_identity: String,
    pub profile: ExperimentProfile,
    pub tool_availability: Vec<ToolAvailability>,
    pub observations_identity: String,
    pub observations: Vec<ToolObservation>,
    pub adoption_identity: String,
    pub adoption: Vec<AdoptionObservation>,
    pub derived: DerivedProjection,
    pub derived_identity: String,
    pub controls: Vec<NegativeControlRecord>,
    pub controls_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperimentEvaluation {
    pub artifact: Option<ExperimentArtifact>,
    pub diagnostics: Vec<String>,
}

/// BTreeMap of case id to case for deterministic iteration.
pub(crate) fn case_map(profile: &ExperimentProfile) -> BTreeMap<&str, &ExperimentCase> {
    profile.corpus.iter().map(|case| (case.case_id.as_str(), case)).collect()
}

pub(crate) fn task_map(profile: &ExperimentProfile) -> BTreeMap<&str, &ExperimentTask> {
    profile.tasks.iter().map(|task| (task.task_id.as_str(), task)).collect()
}

pub(crate) fn tool_map(profile: &ExperimentProfile) -> BTreeMap<&str, &ToolPin> {
    profile.tools.iter().map(|tool| (tool.tool_id.as_str(), tool)).collect()
}

pub(crate) fn applicability_map(profile: &ExperimentProfile) -> BTreeMap<(&str, &str), &Applicability> {
    profile
        .applicability
        .iter()
        .map(|entry| ((entry.tool_id.as_str(), entry.task_id.as_str()), entry))
        .collect()
}
