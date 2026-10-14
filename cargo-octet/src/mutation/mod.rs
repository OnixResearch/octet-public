// r[impl octet.mutation_testing.profile]
// r[impl octet.mutation_testing.receipt]
// r[impl octet.mutation_testing.native_artifacts]
// r[impl octet.mutation_testing.baseline]
// r[impl octet.mutation_testing.boundary]
pub const PROFILE_SCHEMA: &str = "octet-mutation-profiles/v1";
pub const PROFILE_ID: &str = "cargo-mutants-octet-metadata";
pub const PROFILE_IDENTITY: &str = "b3:a40fc0726aea38932f42d6096a84846778e832472767b3c3129de4f7e69d9176";
pub const RECEIPT_SCHEMA: &str = "octet-cargo-mutants-receipt/v1";
pub const SOURCE_MANIFEST_ARTIFACT: &str = "mutation-source-manifest.json";
pub const OUTCOMES_ARTIFACT: &str = "mutation-outcomes.json";
pub const INVENTORY_ARTIFACT: &str = "mutation-inventory.json";
pub const PROJECTION_ARTIFACT: &str = "mutation-projection.json";
pub const REQUIRED_NON_CLAIM: &str = "Mutation outcomes are bounded transcript evidence and do not prove test completeness, source correctness, formal proof, whole-program correctness, or release eligibility.";

const PROFILE_JSON: &str = include_str!("../../../standards/mutation/profiles.json");

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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OutcomePolicy {
    pub caught: String,
    pub missed: String,
    pub timeout: String,
    pub unviable: String,
    pub not_run: String,
    pub tool_failure: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub provider: Provider,
    pub rust_toolchain: String,
    pub scope: Scope,
    pub command_argv: Vec<String>,
    pub output_root: String,
    pub native_outcomes_path: String,
    pub native_inventory_path: String,
    pub normalization_version: String,
    pub source_manifest_schema: String,
    pub outcome_policy: OutcomePolicy,
    pub retain_logs: bool,
    pub retain_diffs: bool,
    pub max_mutants: usize,
    pub max_report_bytes: u64,
    pub max_artifact_count: usize,
    pub max_artifact_bytes: u64,
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RetainedArtifact {
    pub source_locator: String,
    pub locator: String,
    pub identity: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MutantRecord {
    pub stable_id: String,
    pub name: String,
    pub package: String,
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub replacement: String,
    pub genre: String,
    pub source_identity: String,
    pub outcome: String,
    pub log: RetainedArtifact,
    pub diff: Option<RetainedArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BaselineRecord {
    pub outcome: String,
    pub log: RetainedArtifact,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub provider_identity: String,
    pub rust_toolchain: String,
    pub command_argv: Vec<String>,
    pub source_manifest_identity: String,
    pub native_outcomes_identity: String,
    pub native_inventory_identity: String,
    pub projection_identity: String,
    pub baseline: BaselineRecord,
    pub mutants: Vec<MutantRecord>,
    pub outcome_counts: std::collections::BTreeMap<String, u64>,
    pub gate_status: String,
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
        .map_err(|error| format!("parse embedded mutation profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded mutation profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown mutation profile `{profile_id}`"))
}

pub fn projection_bytes(details: &ReceiptDetails) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&details.mutants)
        .map_err(|error| format!("serialize mutation projection: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    validation::validate_details(details)
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

mod records;
pub(crate) mod request;
pub mod shell;
mod validation;

#[cfg(test)]
mod tests;
