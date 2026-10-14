use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolicyRegistry {
    pub schema_version: String,
    pub profile: PilotProfile,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorpusRegistry {
    pub schema_version: String,
    pub fixtures: Vec<CorpusFixture>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PilotProfile {
    pub profile_id: String,
    pub provider: Provider,
    pub output_contract: OutputContract,
    pub corpus: Vec<CorpusFixture>,
    pub limits: Limits,
    pub comparison_outcomes: Vec<String>,
    pub comparison_parents: ComparisonParents,
    pub decision_states: Vec<String>,
    pub promotion_criteria: Vec<Criterion>,
    pub evidence_tier: String,
    pub verification_role: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Provider {
    pub repository: String,
    pub revision: String,
    pub source_hash: String,
    pub license: String,
    pub version: String,
    pub rust_toolchain: String,
    pub target: String,
    pub features: Vec<String>,
    pub derivation_attr: String,
    pub derivation_name: String,
    pub closure_components: Vec<String>,
    pub command_argv: Vec<String>,
    pub environment: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutputContract {
    pub status: String,
    pub reason_code: String,
    pub stable_machine_schema: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CorpusFixture {
    pub fixture_id: String,
    pub path: String,
    pub source_identity: String,
    pub bytes: u64,
    pub property_class: String,
    pub expected_observation: String,
    pub supported: bool,
    pub unsupported_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub max_fixtures: usize,
    pub max_source_bytes: u64,
    pub timeout_ms: u64,
    pub max_log_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComparisonParents {
    pub charon_profile: String,
    pub source_token_catalog: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Criterion {
    pub criterion_id: String,
    pub required: bool,
    pub threshold: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyzerFact {
    pub parent_identity: String,
    pub source_identity: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ComparisonRow {
    pub fixture_id: String,
    pub source_identity: String,
    pub property_class: String,
    pub outcome: String,
    pub parent_identities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    pub criterion_id: String,
    pub observed: Option<u64>,
    pub passed: bool,
    pub availability: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionReceipt {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub corpus_identity: String,
    pub provider_revision: String,
    pub provider_source_hash: String,
    pub rust_toolchain: String,
    pub target: String,
    pub derivation_attr: String,
    pub closure_components: Vec<String>,
    pub command_argv: Vec<String>,
    pub environment: Vec<String>,
    pub decision: String,
    pub blocker_code: String,
    pub failed_criteria: Vec<String>,
    pub measurements: Vec<Measurement>,
    pub comparisons: Vec<ComparisonRow>,
    pub execution_attempted: bool,
    pub reproducible_decision: bool,
    pub claims: Vec<String>,
    pub evidence_tier: String,
    pub verification_role: String,
    pub non_claims: Vec<String>,
    pub receipt_identity: String,
}
