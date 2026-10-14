use serde::Deserialize;
use serde::Serialize;

/// Closed downstream profile for the pinned Aeneas provider.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerProfile {
    /// Consumer-profile schema version.
    pub schema_version: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Stable owner-selected consumer identifier.
    pub consumer_id: String,
    /// Exact provider and compatibility identities.
    pub provider: ProviderBinding,
    /// Exact source and selected-item scope.
    pub source: SourceBinding,
    /// Maintained theorem mappings.
    pub theorems: Vec<TheoremBinding>,
    /// Declared external semantic models.
    pub external_models: Vec<TrustInput>,
    /// Other declared trust inputs.
    pub trust_inputs: Vec<TrustInput>,
    /// Consumer resource bounds within provider limits.
    pub resource_policy: ResourcePolicy,
    /// Closed supported, blocked, and failed outcome vocabulary.
    pub outcomes: Vec<String>,
    /// Whether the profile requires complete trust disclosure.
    pub trust_closure_complete: bool,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
}

/// Exact pinned provider identity referenced by a consumer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderBinding {
    /// Selected proof backend.
    pub backend: String,
    /// Reviewed provider profile identifier.
    pub profile_id: String,
    /// BLAKE3 identity of the reviewed provider profile.
    pub profile_identity: String,
    /// Reviewed compatibility-matrix identifier.
    pub compatibility_matrix_id: String,
    /// BLAKE3 identity of the compatibility matrix.
    pub compatibility_matrix_identity: String,
    /// Whether the consumer requests network authority.
    pub network_allowed: bool,
}

/// Exact source root, files, and selected items.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceBinding {
    /// Root relative to the explicit source capability.
    pub root: String,
    /// Selected crate name.
    pub crate_name: String,
    /// Exact source file identities.
    pub paths: Vec<SourceFileBinding>,
    /// Exact selected items and spans.
    pub selected_items: Vec<SelectedItem>,
    /// Whether broad crate selection is allowed.
    pub broad_selection_allowed: bool,
}

/// One source path and its exact content identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFileBinding {
    /// Workspace-relative source path.
    pub path: String,
    /// BLAKE3 identity of source bytes.
    pub identity: String,
}

/// One selected source item joined across provider artifacts.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SelectedItem {
    /// Qualified Rust item name.
    pub qualified_name: String,
    /// Supported item kind.
    pub kind: String,
    /// Workspace-relative source file.
    pub source_file: String,
    /// BLAKE3 identity of the selected source file.
    pub source_identity: String,
    /// One-based inclusive starting line.
    pub begin_line: u64,
    /// One-based inclusive ending line.
    pub end_line: u64,
    /// Expected generated Lean declaration name.
    pub generated_name: String,
}

/// One maintained theorem and its consumer-owned claim mapping.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TheoremBinding {
    /// Source-capability-relative theorem path.
    pub path: String,
    /// Stable theorem identifier.
    pub theorem_id: String,
    /// Lean declaration name expected in theorem source.
    pub declaration_name: String,
    /// Consumer requirement authorizing the claim.
    pub requirement_id: String,
    /// Reviewed claim class.
    pub claim_class: String,
    /// Approved BLAKE3 identity of maintained theorem bytes.
    pub approved_statement_blake3: String,
    /// Maintained or generated source classification.
    pub source_disposition: String,
}

/// One explicit model or trust input.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrustInput {
    /// Stable trust-input identifier.
    pub input_id: String,
    /// BLAKE3 identity of the trust input.
    pub identity: String,
}

/// Consumer limits that cannot exceed provider authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePolicy {
    /// Maximum stage execution time in milliseconds.
    pub timeout_ms: u64,
    /// Maximum retained stdout bytes.
    pub stdout_max_bytes: usize,
    /// Maximum retained stderr bytes.
    pub stderr_max_bytes: usize,
    /// Exact environment variable allowlist.
    pub environment_allowlist: Vec<String>,
}

/// Final consumer execution classification.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumerOutcome {
    /// The complete selected proof path passed.
    Supported,
    /// A reviewed language boundary rejected the exact source.
    Blocked,
    /// Policy, process, resource, or artifact validation failed.
    Failed,
}

/// Typed status for one execution stage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageStatus {
    /// The stage completed within policy.
    Succeeded,
    /// A reviewed source construct is unsupported.
    Unsupported,
    /// The provider process failed.
    ProcessFailed,
    /// Provider or consumer policy failed.
    PolicyFailed,
    /// A produced artifact failed validation.
    ArtifactFailed,
    /// The bounded stage timed out.
    TimedOut,
    /// A retained-output or other budget was exhausted.
    BudgetExceeded,
}

/// Loaded facts for deterministic outcome classification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageObservation {
    /// Stable stage name.
    pub stage: String,
    /// Typed stage status.
    pub status: StageStatus,
    /// Reviewed unsupported construct, when applicable.
    pub construct: Option<String>,
    /// BLAKE3 identity of the bounded diagnostic.
    pub diagnostic_identity: Option<String>,
    /// Next discriminating action for a blocker.
    pub next_action: Option<String>,
}

/// Deterministic classification and bounded blocker facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutcomeDecision {
    /// Final consumer outcome.
    pub outcome: ConsumerOutcome,
    /// Stage that blocked or failed.
    pub stage: Option<String>,
    /// Reviewed unsupported construct.
    pub construct: Option<String>,
    /// BLAKE3 identity of the bounded diagnostic.
    pub diagnostic_identity: Option<String>,
    /// Next discriminating action.
    pub next_action: Option<String>,
}

/// One exact artifact identity retained by the consumer binding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerArtifactIdentity {
    /// Artifact role in the source-to-theorem path.
    pub role: String,
    /// BLAKE3 identity of exact artifact bytes.
    pub blake3: String,
}

/// Consumer-specific binding added to a verified formal-proof envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConsumerReceiptBinding {
    /// Consumer receipt-binding schema version.
    pub schema_version: String,
    /// Selected consumer profile identifier.
    pub profile_id: String,
    /// BLAKE3 identity of the consumer profile.
    pub profile_identity: String,
    /// Stable consumer identifier.
    pub consumer_id: String,
    /// Exact provider profile identity.
    pub provider_profile_identity: String,
    /// Exact compatibility-matrix identity.
    pub compatibility_matrix_identity: String,
    /// Explicit capability-relative source root.
    pub source_root: String,
    /// Selected source path.
    pub source_path: String,
    /// BLAKE3 identity of selected source bytes.
    pub source_identity: String,
    /// Exact selected item and span.
    pub selected_item: SelectedItem,
    /// Maintained theorem mapping.
    pub theorem: TheoremBinding,
    /// Declared external models.
    pub external_models: Vec<TrustInput>,
    /// Declared trust inputs.
    pub trust_inputs: Vec<TrustInput>,
    /// Exact bounded resource and environment policy.
    pub resource_policy: ResourcePolicy,
    /// Final consumer outcome.
    pub outcome: ConsumerOutcome,
    /// Exact source, translation, generated, and theorem artifacts.
    pub stage_artifacts: Vec<ConsumerArtifactIdentity>,
    /// Evidence role of the verified theorem binding.
    pub verification_role: String,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
    /// BLAKE3 identity of the receipt projection.
    pub receipt_identity: String,
}
