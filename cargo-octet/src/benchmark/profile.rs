use super::PROFILE_SCHEMA;

const PROFILE_JSON: &str = include_str!("../../../standards/benchmark/profiles.json");

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    pub package: String,
    pub version: String,
    pub nixpkgs_revision: String,
    pub closure_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkloadProfile {
    pub workload_id: String,
    pub workload_identity: String,
    pub command_id: String,
    pub command_argv: Vec<String>,
    pub mode: String,
    pub source_state_identity: String,
    pub environment_identity: String,
    pub required_environment_facts: Vec<String>,
    pub sample_count: usize,
    pub required_metrics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CorpusProfile {
    pub corpus_id: String,
    pub source_kind: String,
    pub source_repository: String,
    pub source_revision: String,
    pub representativeness_boundary: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResourceProfile {
    pub allocation_input_path: String,
    pub llvm_input_path: String,
    pub compiler_identity: String,
    pub target: String,
    pub optimization_profile: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComparisonPolicy {
    pub practical_threshold_basis_points: u64,
    pub uncertainty_threshold_basis_points: u64,
    pub minimum_samples: usize,
    pub summary: SummaryKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryKind {
    Median,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub max_samples: usize,
    pub max_log_bytes: usize,
    pub max_artifact_bytes: u64,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub provider: Provider,
    pub rust_toolchain: String,
    pub workloads: Vec<WorkloadProfile>,
    pub signal_corpus: Vec<CorpusProfile>,
    pub resource_observation: ResourceProfile,
    pub comparison: ComparisonPolicy,
    pub limits: Limits,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Registry {
    schema_version: String,
    profiles: Vec<Profile>,
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    let registry: Registry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded benchmark profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded benchmark profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown benchmark profile `{profile_id}`"))
}
