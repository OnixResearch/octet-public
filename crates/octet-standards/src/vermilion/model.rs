use serde::Deserialize;
use serde::Serialize;

/// Reviewed provider profile for the pinned Vermilion-to-Lean route.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProfile {
    /// Provider-profile schema version.
    pub schema_version: String,
    /// Stable reviewed profile identifier.
    pub profile_id: String,
    /// Closed provider identifier.
    pub provider_id: String,
    /// Claim scope made available by a complete provider receipt.
    pub claim_scope: String,
    /// Exact Vermilion revision.
    pub vermilion_revision: String,
    /// Vermilion repository.
    pub vermilion_repository: String,
    /// Pinned Verus front/middle-end fork.
    pub verus_fork: VerusForkProfile,
    /// Translation stages from VIR to generated Lean.
    pub translator: TranslatorProfile,
    /// Lean, Mathlib, and dependency pins.
    pub lean: LeanProfile,
    /// Checker binary and required arguments.
    pub checker: CheckerProfile,
    /// Exact environment variable allowlist.
    pub environment_allowlist: Vec<String>,
    /// Reviewer-declared process bounds.
    pub limits: ExecutionLimits,
    /// Axiom, hole, and completeness policy.
    pub assumption_policy: AssumptionPolicy,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
}

/// Pinned Verus front/middle-end fork and its vstd revision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerusForkProfile {
    /// Fork repository.
    pub repository: String,
    /// Reviewed branch.
    pub branch: String,
    /// Exact fork commit.
    pub revision: String,
    /// Exact vstd revision used by the front end.
    pub vstd_revision: String,
}

/// Vermilion direct/low-level and Lean-generation stages.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TranslatorProfile {
    /// Translator component name.
    pub name: String,
    /// Exact translator revision.
    pub revision: String,
    /// Reviewed command template; `<...>` placeholders are bound per run.
    pub argv: Vec<String>,
}

/// Lean toolchain and pinned package inventory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeanProfile {
    /// Lean toolchain identifier.
    pub toolchain: String,
    /// Mathlib revision.
    pub mathlib_revision: String,
    /// BLAKE3 of the exact `lake-manifest.json` bytes.
    pub lake_manifest_blake3: String,
    /// Every declared Lean dependency.
    pub packages: Vec<PinnedDependency>,
}

/// One pinned Lean dependency.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedDependency {
    /// Dependency name.
    pub name: String,
    /// Exact dependency revision.
    pub revision: String,
}

/// Checker identity and its mandatory arguments.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckerProfile {
    /// Checker executable name.
    pub checker_id: String,
    /// Exact checker revision.
    pub revision: String,
    /// Reviewed argument vector.
    pub argv: Vec<String>,
    /// Whether a proof-twin check is mandatory.
    pub requires_twin_check: bool,
}

/// Bounded process limits.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionLimits {
    /// Maximum stage execution time in milliseconds.
    pub timeout_ms: u64,
    /// Maximum retained stdout bytes.
    pub stdout_max_bytes: usize,
    /// Maximum retained stderr bytes.
    pub stderr_max_bytes: usize,
}

/// Reviewed axiom and proof-hole policy.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssumptionPolicy {
    /// Axioms allowed by every admitted proof.
    pub allowed_axioms: Vec<String>,
    /// Proof-hole markers that fail admission.
    pub proof_holes: Vec<String>,
    /// Whether every assumption must be declared.
    pub requires_complete_declaration: bool,
    /// Whether proof twins must be checked.
    pub require_twin_check: bool,
}

/// Common semantic contract for one bounded pilot property.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticContract {
    /// Contract schema version.
    pub schema_version: String,
    /// Stable contract identifier.
    pub contract_id: String,
    /// Reviewed contract revision.
    pub revision: String,
    /// Exact subject source path.
    pub subject_path: String,
    /// BLAKE3 identity of the subject source bytes.
    pub subject_blake3: String,
    /// Human-readable common statement.
    pub statement: String,
    /// BLAKE3 identity of the canonical statement bytes.
    pub statement_blake3: String,
    /// Arithmetic width in bits.
    pub width_bits: u16,
    /// Selected overflow behavior.
    pub overflow_behavior: String,
    /// Explicit precondition.
    pub precondition: String,
    /// Explicit postcondition.
    pub postcondition: String,
    /// Observable result type.
    pub result_type: String,
    /// Observable failure cases.
    pub observable_failures: Vec<String>,
    /// Termination scope.
    pub termination_scope: String,
    /// Per-backend obligation and bridge declarations.
    pub routes: Vec<RouteObligations>,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
}

/// One backend's declared obligations and refinement bridge.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RouteObligations {
    /// Backend identifier.
    pub backend: String,
    /// Claim scope available for this route.
    pub claim_scope: String,
    /// Obligations that must be accounted for by a receipt.
    pub required_obligations: Vec<RequiredObligation>,
    /// Stable refinement-bridge identifier.
    pub refinement_bridge_id: String,
    /// Human-readable bridge status and limitation.
    pub refinement_bridge: String,
    /// Modeled library contracts.
    pub modeled_library_contracts: Vec<String>,
}

/// One required obligation identity and kind.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredObligation {
    /// Stable obligation identifier.
    pub obligation_id: String,
    /// Obligation kind.
    pub kind: String,
}

/// Exact provider binding carried by a Vermilion receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptProvider {
    /// Provider identifier.
    pub provider_id: String,
    /// Reviewed profile identifier.
    pub profile_id: String,
    /// BLAKE3 identity of the reviewed profile bytes.
    pub profile_identity: String,
    /// BLAKE3 identity of the compatibility matrix bytes.
    pub compatibility_matrix_identity: String,
}

/// Semantic-spec binding carried by a Vermilion receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticSpecBinding {
    /// Contract identifier.
    pub contract_id: String,
    /// Contract revision.
    pub revision: String,
    /// BLAKE3 identity of the canonical statement.
    pub statement_blake3: String,
    /// Arithmetic width in bits.
    pub width_bits: u16,
    /// Selected overflow behavior.
    pub overflow_behavior: String,
    /// Explicit precondition.
    pub precondition: String,
    /// Explicit postcondition.
    pub postcondition: String,
}

/// One exact source binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBinding {
    /// Repository-relative source path.
    pub path: String,
    /// Qualified selected item.
    pub item: String,
    /// One-based inclusive start line.
    pub begin_line: u64,
    /// One-based inclusive end line.
    pub end_line: u64,
    /// BLAKE3 identity of the source bytes.
    pub blake3: String,
}

/// Toolchain identities bound by a receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainIdentity {
    /// Backend identifier.
    pub backend: String,
    /// Verus front-end revision.
    pub verus_frontend_revision: String,
    /// vstd revision.
    pub vstd_revision: String,
    /// Translator revision.
    pub translator_revision: String,
    /// Lean toolchain identifier.
    pub lean_toolchain: String,
    /// Mathlib revision.
    pub mathlib_revision: String,
    /// Solver revision (cvc5 via lean-cvc5).
    pub solver_revision: String,
    /// BLAKE3 of the exact `lake-manifest.json` bytes.
    pub lake_manifest_blake3: String,
}

/// Command, environment, and bounds recorded by a receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandBinding {
    /// Exact argument vector.
    pub argv: Vec<String>,
    /// Exact environment allowlist.
    pub environment_allowlist: Vec<String>,
    /// Enforced limits.
    pub limits: ExecutionLimits,
}

/// One generated artifact identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProofArtifact {
    /// Artifact role.
    pub role: String,
    /// Repository-relative path.
    pub path: String,
    /// BLAKE3 identity of exact bytes.
    pub blake3: String,
}

/// One obligation record and the artifacts that back it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObligationRecord {
    /// Stable obligation identifier.
    pub obligation_id: String,
    /// Obligation kind.
    pub kind: String,
    /// Generated artifact role carrying the obligation.
    pub generated_artifact_role: String,
    /// BLAKE3 identity of the generated artifact bytes.
    pub generated_artifact_blake3: String,
    /// Proof-twin artifact role.
    pub proof_twin_role: String,
    /// BLAKE3 identity of the proof-twin bytes.
    pub proof_twin_blake3: String,
    /// Lean theorem name.
    pub theorem: String,
    /// Assumptions used by this theorem.
    pub assumptions: Vec<String>,
    /// Obligation status.
    pub status: ObligationStatus,
}

/// Verified status of one obligation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObligationStatus {
    /// The checker accepted the obligation.
    Checked,
    /// The obligation remains unproved.
    Unchecked,
    /// The obligation was not emitted or was omitted.
    Missing,
    /// The checker rejected the obligation.
    Rejected,
}

/// Refinement bridge from a route representation to the common contract.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementBridge {
    /// Stable bridge identifier.
    pub bridge_id: String,
    /// Whether the bridge is checked or an open obligation.
    pub status: BridgeStatus,
    /// Exact bridge statement or recorded limitation.
    pub statement: String,
}

/// Checked status of a refinement bridge.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BridgeStatus {
    /// The bridge has its own checked theorem.
    Checked,
    /// The bridge is a visible open obligation.
    Open,
}

/// Observed checker execution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckerObservation {
    /// Checker identifier.
    pub checker_id: String,
    /// Checker status.
    pub status: CheckerStatus,
    /// Process exit code when the checker started.
    pub exit_code: Option<i32>,
    /// Whether all required obligations were judged.
    pub complete: bool,
    /// Whether the machine-readable result was empty.
    pub output_empty: bool,
    /// Required obligation count.
    pub required_obligations: u64,
    /// Checked obligation count.
    pub checked_obligations: u64,
    /// BLAKE3 identity of the bounded diagnostic output, when any.
    pub diagnostic_identity: Option<String>,
}

/// Outcome of one checker attempt.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckerStatus {
    /// The checker accepted every obligation.
    Passed,
    /// The checker ran and rejected at least one obligation.
    Failed,
    /// The checker was interrupted before completing.
    Interrupted,
    /// The checker could not start.
    Unavailable,
}

/// Theorem and axiom inventory observed for one receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProofInventory {
    /// Every checked theorem.
    pub theorems: Vec<TheoremRecord>,
    /// Every axiom used by those theorems.
    pub axioms: Vec<String>,
}

/// One theorem inventory row.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TheoremRecord {
    /// Obligation identifier.
    pub obligation_id: String,
    /// Lean theorem name.
    pub theorem: String,
    /// Assumptions used by the theorem.
    pub assumptions: Vec<String>,
}

/// Receipt provenance class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptOrigin {
    /// The receipt observes an actual provider execution.
    ProviderRun,
    /// The receipt is a synthetic contract-conformance fixture.
    SyntheticContractFixture,
}

/// Admissible evidence role after evaluation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceDisposition {
    /// The provider completed and the receipt may carry property evidence.
    Property,
    /// The receipt is a bounded observation only.
    RecordedOnly,
}

/// Final Vermilion outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VermilionOutcome {
    /// Every required obligation was checked under the reviewed policy.
    Supported,
    /// A reviewed tool or language boundary prevented execution.
    Blocked,
    /// Execution or contract validation failed.
    Failed,
}

impl VermilionOutcome {
    /// Return the stable serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

/// Typed comparison carried by a receipt when more than one backend is present.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonMetadata {
    /// Comparison category spelling.
    pub category: String,
    /// Participating providers.
    pub participants: Vec<String>,
    /// Whether any provider evidence authority was merged.
    pub evidence_merge: bool,
    /// Open obligations that prevent agreement from implying equivalence.
    pub open_obligations: Vec<String>,
}

/// One Vermilion receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VermilionReceipt {
    /// Receipt schema version.
    pub schema_version: String,
    /// BLAKE3 identity of this receipt with the field cleared.
    pub receipt_identity: String,
    /// Receipt provenance class.
    pub origin: ReceiptOrigin,
    /// Exact provider binding.
    pub provider: ReceiptProvider,
    /// Exact semantic-spec binding.
    pub semantic_spec: SemanticSpecBinding,
    /// Exact selected source.
    pub source_tree: Vec<SourceBinding>,
    /// Toolchain identities.
    pub toolchain: ToolchainIdentity,
    /// Command, environment, and bounds.
    pub command: CommandBinding,
    /// Generated artifacts and proof twins.
    pub generated_artifacts: Vec<ProofArtifact>,
    /// Required obligations.
    pub obligations: Vec<ObligationRecord>,
    /// Refinement bridges, including open obligations.
    pub refinement_bridges: Vec<RefinementBridge>,
    /// Checker observation.
    pub checker: CheckerObservation,
    /// Theorem and axiom inventory.
    pub proof_inventory: ProofInventory,
    /// Receipt-declared outcome.
    pub outcome: VermilionOutcome,
    /// Evidence role requested by the receipt.
    pub evidence_disposition: EvidenceDisposition,
    /// Optional cross-provider comparison metadata.
    pub comparison: Option<ComparisonMetadata>,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
}
