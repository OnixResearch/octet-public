use serde::Deserialize;
use serde::Serialize;

pub(crate) const PROFILE_SCHEMA: &str = "octet-charon-semantic-profile/v1";
pub(crate) const PROFILE_ID: &str = "octet-charon-semantic-safe-add-v1";
pub(crate) const ENVELOPE_SCHEMA: &str = "octet-charon-semantic-input/v1";
pub(crate) const DETAILS_SCHEMA: &str = "octet-charon-semantic-receipt-details/v1";
pub(crate) const PROJECTION_SCHEMA: &str = "octet-charon-semantic-projection/v1";
pub(crate) const ANALYSIS_SCHEMA: &str = "octet-charon-semantic-analysis/v1";
pub(crate) const COMPARISON_SCHEMA: &str = "octet-charon-semantic-comparison/v1";
pub(crate) const COMPARISON_PARENT_SCHEMA: &str = "octet-charon-semantic-comparison-parent/v1";
pub(crate) const EXPECTED_PROFILE_IDENTITY: &str =
    "b3:9c37280796d04408389e7efc4fd8413b6c3bb3594fc66bed79ea01492b9cdf67";
pub(crate) const EXPECTED_PROVIDER_PROFILE_IDENTITY: &str =
    "b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625";
pub(crate) const EXPECTED_COMPATIBILITY_IDENTITY: &str =
    "b3:403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5";
pub(crate) const EXPECTED_CHARON_REVISION: &str = "cb50ff16b9f1066b8a97dc06da704de2da2fa41c";
pub(crate) const EXPECTED_MECHANIZED_LLBC_REVISION: &str = "8a9b8bb3823ceceb47026837d23baea0934dfb27";
pub(crate) const EXPECTED_CHARON_RUDRA_REVISION: &str = "0923bf7f21d4eb234d14bce86afc9690d0a4d00e";
pub(crate) const RECORDED_ONLY_ROLE: &str = "recorded_only";
pub(crate) const RAW_SNAPSHOT_ARTIFACT: &str = "charon-snapshot.llbc";
pub(crate) const PROJECTION_ARTIFACT: &str = "charon-semantic-projection.json";
pub(crate) const ANALYSIS_ARTIFACT: &str = "charon-semantic-analysis.json";
pub(crate) const COMPARISON_ARTIFACT: &str = "charon-semantic-comparison.json";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CharonSemanticProfile {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) provider: ProviderDependency,
    pub(crate) source: SourceSelection,
    pub(crate) extraction: ExtractionContract,
    pub(crate) fact_vocabulary: Vec<String>,
    pub(crate) effect_candidate_vocabulary: Vec<String>,
    pub(crate) limits: SemanticLimits,
    pub(crate) artifact_roles: Vec<String>,
    pub(crate) references: Vec<ReferenceMetadata>,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProviderDependency {
    pub(crate) package: String,
    pub(crate) profile_id: String,
    pub(crate) profile_blake3: String,
    pub(crate) compatibility_matrix_id: String,
    pub(crate) compatibility_matrix_blake3: String,
    pub(crate) charon_revision: String,
    pub(crate) executable_env: String,
    pub(crate) network_sandbox_env: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SourceSelection {
    pub(crate) root: String,
    pub(crate) crate_name: String,
    pub(crate) selected_items: Vec<SelectedItem>,
    pub(crate) broad_selection_allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SelectedItem {
    pub(crate) qualified_name: String,
    pub(crate) kind: String,
    pub(crate) source_file: String,
    pub(crate) source_identity: String,
    pub(crate) begin_line: u64,
    pub(crate) end_line: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExtractionContract {
    pub(crate) command_argv: Vec<String>,
    pub(crate) preset: String,
    pub(crate) has_errors: bool,
    pub(crate) skip_borrowck: bool,
    pub(crate) borrow_check_preserved: bool,
    pub(crate) target_triple: String,
    pub(crate) pointer_width_bytes: u64,
    pub(crate) endianness: String,
    pub(crate) serialization_format: String,
    pub(crate) environment_allowlist: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SemanticLimits {
    pub(crate) timeout_ms: u64,
    pub(crate) stdout_max_bytes: usize,
    pub(crate) stderr_max_bytes: usize,
    pub(crate) snapshot_max_bytes: usize,
    pub(crate) item_max_count: usize,
    pub(crate) edge_max_count: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReferenceMetadata {
    pub(crate) repository: String,
    pub(crate) revision: String,
    pub(crate) license: String,
    pub(crate) source_sentinel: String,
    pub(crate) r#use: String,
    pub(crate) recorded_only: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CharonSemanticEnvelope {
    pub(crate) schema_version: String,
    pub(crate) snapshot_text: String,
    #[serde(default)]
    pub(crate) comparison_parent_texts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ArtifactBinding {
    pub(crate) role: String,
    pub(crate) name: String,
    pub(crate) identity: String,
    pub(crate) authority: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CharonSemanticReceiptDetails {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) profile_identity: String,
    pub(crate) provider_profile_identity: String,
    pub(crate) compatibility_matrix_identity: String,
    pub(crate) charon_revision: String,
    pub(crate) artifacts: Vec<ArtifactBinding>,
    pub(crate) snapshot_identity: String,
    pub(crate) projection_identity: String,
    pub(crate) analysis_identity: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) comparison_identity: Option<String>,
    pub(crate) verification_role: String,
    pub(crate) artifact_roles: Vec<String>,
    pub(crate) limits: SemanticLimits,
    pub(crate) references: Vec<ReferenceMetadata>,
    pub(crate) projection: SemanticProjection,
    pub(crate) analysis: SemanticAnalysis,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) comparison: Option<SemanticComparison>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SemanticProjection {
    pub(crate) schema_version: String,
    pub(crate) crate_name: String,
    pub(crate) charon_version: String,
    pub(crate) target: TargetMetadata,
    pub(crate) sources: Vec<SourceIdentity>,
    pub(crate) items: Vec<SemanticItem>,
    pub(crate) edges: Vec<SemanticEdge>,
    pub(crate) trait_rows: Vec<TraitResolutionRow>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct TargetMetadata {
    pub(crate) triple: String,
    pub(crate) pointer_width_bytes: u64,
    pub(crate) endianness: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SourceIdentity {
    pub(crate) path: String,
    pub(crate) identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SemanticItem {
    pub(crate) stable_id: String,
    pub(crate) run_local_id: u64,
    pub(crate) qualified_name: String,
    pub(crate) kind: String,
    pub(crate) source_file: String,
    pub(crate) begin_line: u64,
    pub(crate) end_line: u64,
    pub(crate) opacity: String,
    pub(crate) control_flow: String,
    pub(crate) statement_count: u64,
    pub(crate) unsupported: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct SemanticEdge {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) kind: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct TraitResolutionRow {
    pub(crate) trait_name: String,
    pub(crate) implementation_name: String,
    pub(crate) method_items: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SemanticAnalysis {
    pub(crate) schema_version: String,
    pub(crate) status: String,
    pub(crate) item_count: u64,
    pub(crate) edge_count: u64,
    pub(crate) cycle_count: u64,
    pub(crate) cycles: Vec<Vec<String>>,
    pub(crate) effect_candidates: Vec<EffectCandidate>,
    pub(crate) unknowns: Vec<UnknownFact>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct EffectCandidate {
    pub(crate) item: String,
    pub(crate) candidate: String,
    pub(crate) evidence: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct UnknownFact {
    pub(crate) item: String,
    pub(crate) fact: String,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ComparisonParent {
    pub(crate) schema_version: String,
    pub(crate) authority: String,
    pub(crate) identity: String,
    pub(crate) source_identity: String,
    pub(crate) facts: Vec<ComparisonFact>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct ComparisonFact {
    pub(crate) item: String,
    pub(crate) fact: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SemanticComparison {
    pub(crate) schema_version: String,
    pub(crate) parent_identities: Vec<String>,
    pub(crate) rows: Vec<ComparisonRow>,
    pub(crate) summary: ComparisonSummary,
    pub(crate) authority_merged: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct ComparisonRow {
    pub(crate) parent_authority: String,
    pub(crate) item: String,
    pub(crate) fact: String,
    pub(crate) charon_value: Option<String>,
    pub(crate) parent_value: Option<String>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ComparisonSummary {
    pub(crate) agree: u64,
    pub(crate) disagree: u64,
    pub(crate) charon_only: u64,
    pub(crate) source_token_only: u64,
    pub(crate) unknown: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticFinding {
    pub(crate) family: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) file: Option<String>,
    pub(crate) line: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CharonSemanticEvaluation {
    pub(crate) profile_id: Option<String>,
    pub(crate) details: Option<CharonSemanticReceiptDetails>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) findings: Vec<SemanticFinding>,
}
