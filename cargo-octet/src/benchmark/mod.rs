// r[impl octet.benchmark_evidence.profile]
// r[impl octet.benchmark_evidence.receipt]
// r[impl octet.benchmark_evidence.boundary]
pub const PROFILE_SCHEMA: &str = "octet-benchmark-profiles/v1";
pub const PROFILE_ID: &str = "octet-metadata-benchmark";
pub const PROFILE_IDENTITY: &str = "b3:64e4b8163b3c89d8143d4f6ff07230d33b2b2a9b7bcab35bf45325027a20cd47";
pub const INPUT_SCHEMA: &str = "octet-benchmark-input/v1";
pub const RECEIPT_SCHEMA: &str = "octet-benchmark-receipt/v1";
pub const BASELINE_ARTIFACT: &str = "benchmark-baseline-samples.json";
pub const CANDIDATE_ARTIFACT: &str = "benchmark-candidate-samples.json";
pub const ALLOCATION_ARTIFACT: &str = "benchmark-allocations.json";
pub const COPY_ARTIFACT: &str = "benchmark-copy-sites.json";
pub const ORACLE_ARTIFACT: &str = "benchmark-signal-oracle.json";
pub const FINDINGS_ARTIFACT: &str = "benchmark-signal-findings.json";
pub const PROJECTION_ARTIFACT: &str = "benchmark-projection.json";
pub const REQUIRED_NON_CLAIM: &str = "Benchmark results are bounded transcript evidence and do not prove correctness, universal performance, corpus representativeness, formal proof, whole-program behavior, or release eligibility.";

pub use profile::ComparisonPolicy;
pub use profile::Profile;
pub use profile::WorkloadProfile;
pub use profile::selected_profile;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentObservation {
    pub environment_identity: String,
    pub target: String,
    pub host_os: String,
    pub host_arch: String,
    pub kernel_identity: String,
    pub cpu_identity: String,
    pub governor: String,
    pub load_class: String,
    pub rust_toolchain: String,
    pub nix_closure_identity: String,
    pub isolation: String,
    pub cpu_time_supported: bool,
    pub peak_memory_supported: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Sample {
    pub sample_index: usize,
    pub outcome: String,
    pub wall_duration_ns: u64,
    pub cpu_duration_ns: Option<u64>,
    pub peak_memory_bytes: Option<u64>,
    pub output_bytes: u64,
    pub log_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunSet {
    pub workload_id: String,
    pub workload_identity: String,
    pub command_id: String,
    pub command_argv: Vec<String>,
    pub mode: String,
    pub source_state_identity: String,
    pub environment: EnvironmentObservation,
    pub samples: Vec<Sample>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AllocationObservation {
    pub workload_id: String,
    pub instrumentation_identity: String,
    pub interval_id: String,
    pub scope: String,
    pub allocation_events: u64,
    pub allocated_bytes: u64,
    pub zero_allocation_claim: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CopyObservation {
    pub workload_id: String,
    pub llvm_artifact_identity: String,
    pub compiler_identity: String,
    pub target: String,
    pub optimization_profile: String,
    pub site_id: String,
    pub context: String,
    pub constant_length_bytes: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OracleRecord {
    pub case_id: String,
    pub source_identity: String,
    pub lint_id: String,
    pub location_class: String,
    pub expected: String,
    pub rationale: String,
    pub reviewer_role: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalFinding {
    pub case_id: String,
    pub source_identity: String,
    pub lint_id: String,
    pub location_class: String,
    pub suppressed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonResult {
    Improved,
    Stable,
    Regressed,
    Inconclusive,
    Incomparable,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PerformanceComparison {
    pub result: ComparisonResult,
    pub baseline_median_ns: Option<u64>,
    pub candidate_median_ns: Option<u64>,
    pub practical_threshold_basis_points: u64,
    pub uncertainty_threshold_basis_points: u64,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResourceSummary {
    pub allocation_events: u64,
    pub allocated_bytes: u64,
    pub copy_site_count: usize,
    pub known_copy_bytes: u64,
    pub unknown_copy_length_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SignalSummary {
    pub true_positive: u64,
    pub false_positive: u64,
    pub false_negative: u64,
    pub expected_suppression: u64,
    pub unexpected_suppression: u64,
    pub inconclusive: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PublicClaim {
    pub metric: String,
    pub value: String,
    pub projection_identity: String,
    pub corpus: String,
    pub environment: String,
    pub sample_range: String,
    pub interpretation_boundary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BenchmarkProjection {
    pub performance: PerformanceComparison,
    pub resources: ResourceSummary,
    pub signals: SignalSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub provider_identity: String,
    pub command_argv: Vec<String>,
    pub baseline_identity: String,
    pub candidate_identity: String,
    pub allocation_identity: String,
    pub copy_identity: String,
    pub oracle_identity: String,
    pub findings_identity: String,
    pub projection_identity: String,
    pub bounded_log_identity: String,
    pub projection: BenchmarkProjection,
    pub public_claims: Vec<PublicClaim>,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    validation::validate_details(details)
}

pub fn projection_bytes(projection: &BenchmarkProjection) -> Result<Vec<u8>, String> {
    pretty_bytes(projection, "benchmark projection")
}

pub fn pretty_bytes<T: serde::Serialize>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

mod comparison;
mod profile;
mod profile_validation;
pub(crate) mod request;
mod resource_shell;
mod resources;
mod runtime;
pub mod shell;
mod signal;
mod validation;

#[cfg(test)]
mod mode_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
