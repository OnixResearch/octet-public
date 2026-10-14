// r[impl octet.coverage_baseline.profile]
// r[impl octet.coverage_baseline.receipt]
// r[impl octet.coverage_baseline.native_artifacts]
// r[impl octet.coverage_baseline.comparison]
pub const PROFILE_SCHEMA: &str = "octet-coverage-profiles/v1";
pub const PROFILE_ID: &str = "cargo-llvm-cov-octet-lib";
pub const PROFILE_IDENTITY: &str = "b3:e221f5d35528e6979bb44ab58cc5e486109799cf99fb80fc4d85ab77a7082634";
pub const NATIVE_SCHEMA: &str = "octet-coverage-native-report/v1";
pub const RECEIPT_SCHEMA: &str = "octet-coverage-receipt/v1";
pub const SOURCE_MANIFEST_ARTIFACT: &str = "coverage-source-manifest.txt";
pub const CURRENT_RAW_ARTIFACT: &str = "coverage-current-native.json";
pub const CURRENT_ENVELOPE_ARTIFACT: &str = "coverage-current-envelope.json";
pub const BASELINE_ENVELOPE_ARTIFACT: &str = "coverage-baseline-envelope.json";
pub const PROJECTION_ARTIFACT: &str = "coverage-comparison.json";
pub const REQUIRED_NON_CLAIM: &str = "Coverage counts are bounded transcript evidence and do not prove behavioral correctness, path correctness, test completeness, formal proof, whole-program correctness, or release eligibility.";
pub const SOURCE_IDENTITY_PREFIX: &str = "source:b3:";

const PROFILE_JSON: &str = include_str!("../../../standards/coverage/profiles.json");

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    pub package: String,
    pub version: String,
    pub nixpkgs_revision: String,
    pub closure_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Scope {
    pub package: String,
    pub target: String,
    pub all_features: bool,
    pub test_selection: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Thresholds {
    pub lines: u64,
    pub branches: u64,
    pub functions: u64,
    pub regions: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub provider: Provider,
    pub rust_toolchain: String,
    pub llvm_tools_component: String,
    pub scope: Scope,
    pub metrics: Vec<String>,
    pub command_argv: Vec<String>,
    pub current_report_path: String,
    pub normalization_version: String,
    pub source_scope_identity: String,
    pub path_policy_identity: String,
    pub threshold_basis_points: Thresholds,
    pub regression_policy: String,
    pub denominator_policy: String,
    pub rounding_policy: String,
    pub precision_digits: u8,
    pub max_report_bytes: u64,
    pub max_files: usize,
    pub max_path_bytes: usize,
    pub max_log_bytes: usize,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
struct Registry {
    schema_version: String,
    profiles: Vec<Profile>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetricCount {
    pub covered: u64,
    pub total: u64,
    pub percentage_basis_points: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Counts {
    pub lines: MetricCount,
    pub branches: MetricCount,
    pub functions: MetricCount,
    pub regions: MetricCount,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeReport {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub provider_identity: String,
    pub rust_toolchain: String,
    pub llvm_tools_component: String,
    pub source_scope_identity: String,
    pub path_policy_identity: String,
    pub source_identity: String,
    pub raw_report_identity: String,
    pub file_count: usize,
    pub counts: Counts,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NativeEnvelope {
    pub schema_version: String,
    pub report: NativeReport,
    pub raw_report_text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetricComparison {
    pub baseline: MetricCount,
    pub current: MetricCount,
    pub covered_delta: i128,
    pub total_delta: i128,
    pub threshold_basis_points: u64,
    pub threshold_passed: bool,
    pub regression_passed: bool,
    pub denominator_passed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Comparison {
    pub regression_policy: String,
    pub denominator_policy: String,
    pub rounding_policy: String,
    pub precision_digits: u8,
    pub lines: MetricComparison,
    pub branches: MetricComparison,
    pub functions: MetricComparison,
    pub regions: MetricComparison,
    pub passed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub command_argv: Vec<String>,
    pub current: NativeReport,
    pub baseline: NativeReport,
    pub current_envelope_identity: String,
    pub baseline_envelope_identity: String,
    pub projection_identity: String,
    pub comparison: Comparison,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    let registry: Registry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded coverage profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded coverage profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown coverage profile `{profile_id}`"))
}

pub fn evaluate_profiled(
    current_text: &str,
    baseline_envelope_text: &str,
    profile_id: &str,
    source_identity: &str,
) -> Evaluation {
    let profile = match selected_profile(profile_id) {
        Ok(value) => value,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![error],
            };
        }
    };
    validation::evaluate(current_text, baseline_envelope_text, &profile, source_identity)
}

pub(crate) fn prepare_baseline_text(
    baseline_text: &str,
    profile_id: &str,
    source_identity: &str,
) -> Result<String, String> {
    if serde_json::from_str::<serde_json::Value>(baseline_text)
        .ok()
        .and_then(|value| value.get("schema_version").cloned())
        .is_some()
    {
        return Ok(baseline_text.to_owned());
    }
    baseline_envelope_text(baseline_text, profile_id, source_identity)
}

pub(crate) fn baseline_envelope_text(
    raw_report_text: &str,
    profile_id: &str,
    source_identity: &str,
) -> Result<String, String> {
    let profile = selected_profile(profile_id)?;
    let report =
        native::normalize(raw_report_text, &profile, source_identity).map_err(|diagnostics| diagnostics.join("; "))?;
    let bytes = envelope_bytes(&report, raw_report_text)?;
    String::from_utf8(bytes).map_err(|error| format!("coverage envelope is not UTF-8: {error}"))
}

pub fn envelope_bytes(report: &NativeReport, raw_report_text: &str) -> Result<Vec<u8>, String> {
    let envelope = NativeEnvelope {
        schema_version: NATIVE_SCHEMA.to_owned(),
        report: report.clone(),
        raw_report_text: raw_report_text.to_owned(),
    };
    pretty_bytes(&envelope, "coverage native envelope")
}

pub fn projection_bytes(details: &ReceiptDetails) -> Result<Vec<u8>, String> {
    pretty_bytes(&details.comparison, "coverage comparison projection")
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    validation::validate_details(details)
}

fn pretty_bytes<T: serde::Serialize>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

mod comparison;
mod native;
pub(crate) mod request;
pub mod shell;
mod validation;

#[cfg(test)]
mod tests;
