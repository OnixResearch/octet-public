//! Bounded Vermilion provider evaluation and cross-provider comparison.
//!
//! Vermilion keeps the pinned Verus Rust front/middle end and replaces the
//! SMT backend with explicit Lean theorem obligations. This module owns the
//! reviewed provider profile, the pilot semantic contract, the typed receipt
//! contract, and a pure comparison vocabulary. It does not execute Vermilion,
//! Verus, Lean, or any solver.
// r[impl octet.vermilion_proof_rail.provider_profile]
// r[impl octet.vermilion_proof_rail.semantic_contract]
// r[impl octet.vermilion_proof_rail.receipt]
// r[impl octet.vermilion_proof_rail.obligations]
// r[impl octet.vermilion_proof_rail.comparison]
pub use comparison::ComparisonCategory;
pub use comparison::ComparisonReport;
pub use comparison::ComparisonRow;
pub use comparison::RouteClaim;
pub use comparison::RouteEvidence;
pub use comparison::RouteOutcome;
pub use comparison::compare;
pub use model::AssumptionPolicy;
pub use model::BridgeStatus;
pub use model::CheckerObservation;
pub use model::CheckerProfile;
pub use model::CheckerStatus;
pub use model::CommandBinding;
pub use model::ComparisonMetadata;
pub use model::EvidenceDisposition;
pub use model::ExecutionLimits;
pub use model::LeanProfile;
pub use model::ObligationRecord;
pub use model::ObligationStatus;
pub use model::PinnedDependency;
pub use model::ProofArtifact;
pub use model::ProofInventory;
pub use model::ProviderProfile;
pub use model::ReceiptOrigin;
pub use model::ReceiptProvider;
pub use model::RefinementBridge;
pub use model::RequiredObligation;
pub use model::RouteObligations;
pub use model::SemanticContract;
pub use model::SemanticSpecBinding;
pub use model::SourceBinding;
pub use model::TheoremRecord;
pub use model::ToolchainIdentity;
pub use model::TranslatorProfile;
pub use model::VermilionOutcome;
pub use model::VermilionReceipt;
pub use model::VerusForkProfile;
use serde::Deserialize;
use serde::Serialize;

/// Provider-profile schema version.
pub const PROFILE_SCHEMA: &str = "octet-vermilion-lean-provider-profile/v1";
/// Receipt schema version.
pub const RECEIPT_SCHEMA: &str = "octet-vermilion-proof-receipt/v1";
/// Cross-provider comparison schema version.
pub const COMPARISON_SCHEMA: &str = "octet-provider-comparison/v1";
/// Closed provider identifier.
pub const PROVIDER_ID: &str = "vermilion-lean";
/// Reviewed provider profile identifier.
pub const EXPECTED_PROFILE_ID: &str = "octet-vermilion-lean-7c83bbc";
/// BLAKE3 identity of the reviewed provider profile bytes.
pub const PROFILE_IDENTITY: &str = "b3:a95f705436b12445d3d3cb6b7f794cf37f1e610349c1cb24725e5fd0abf65fab";
/// BLAKE3 identity of the reviewed compatibility-matrix bytes.
pub const COMPATIBILITY_MATRIX_IDENTITY: &str = "b3:285ca956a570428c2b748003c145868a20ccf33df645c66ab02200ccd942a9b7";
/// Pilot semantic-contract schema version.
pub const CONTRACT_SCHEMA: &str = "octet-proof-semantic-contract/v1";
/// Pilot semantic-contract identifier.
pub const CONTRACT_ID: &str = "octet.bounded_increment.checked/v1";
/// BLAKE3 identity of the reviewed pilot contract bytes.
pub const CONTRACT_IDENTITY: &str = "b3:47abb1f9e604b1ef7259290eda7429417bacfdf4ae6860ad382c656058eb8fae";
/// Exact pilot semantic statement bytes.
pub const PILOT_SPEC_TEXT: &str =
    include_str!("../../../../standards/proof-rails/vermilion-lean/pilot/semantic-spec.txt");
/// Exact pilot source bytes.
pub const PILOT_SOURCE: &str = include_str!("../../../../standards/proof-rails/vermilion-lean/pilot/src/lib.rs");
/// Required non-claims on every contract and receipt.
pub const REQUIRED_NON_CLAIMS: [&str; 5] = [
    "does not prove Vermilion, Verus, or Lean soundness",
    "does not prove that any route refines the executable Rust function",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not establish release eligibility",
];

const PROFILE_JSON: &str = include_str!("../../../../standards/proof-rails/vermilion-lean/generated/profile.json");
const CONTRACT_JSON: &str =
    include_str!("../../../../standards/proof-rails/vermilion-lean/generated/pilot-contract.json");
const BLAKE3_HEX_LENGTH: usize = 64;

/// Typed validation diagnostic class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticClass {
    /// No diagnostic.
    None,
    /// Receipt schema or provider envelope drifted.
    SchemaDrift,
    /// Source bytes, path, item, or span were substituted.
    SourceSubstitution,
    /// Semantic-spec identity or revision was substituted.
    SpecSubstitution,
    /// A precondition or postcondition was weakened.
    WeakenedPrecondition,
    /// Overflow behavior changed.
    ChangedOverflowSemantics,
    /// A `nat` claim was substituted for a `u64` claim.
    NatU64Confusion,
    /// The receipt names another provider.
    WrongProvider,
    /// Toolchain pins drifted.
    ToolchainDrift,
    /// The dependency audit or `lake-manifest.json` binding is absent.
    MissingDependencyAudit,
    /// Generated output or a proof twin is stale.
    StaleGeneratedOutput,
    /// A proof-twin identity is stale.
    StaleProofTwin,
    /// A required obligation is missing or has the wrong kind.
    MissingObligation,
    /// An obligation outside the contract is present.
    UnexpectedObligation,
    /// An assumption outside the reviewed allowance is present.
    UnexpectedAxiom,
    /// A `sorry`, `admit`, or other proof hole remains.
    UnresolvedProofHole,
    /// The checker rejected an obligation.
    ProofRejected,
    /// The checker was interrupted.
    InterruptedChecking,
    /// The checker emitted no machine-readable result.
    EmptyOutput,
    /// The checker could not start.
    MissingChecker,
    /// The command, environment, or resource policy drifted.
    CommandDrift,
    /// The receipt requests more evidence authority than its origin allows.
    EvidenceOverclaim,
    /// Claim boundaries were weakened or removed.
    NonClaimDrift,
    /// The receipt projection was tampered with.
    ReceiptTamper,
    /// The exact source or language surface is unsupported.
    UnsupportedSurface,
}

impl DiagnosticClass {
    /// Return the stable serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SchemaDrift => "schema_drift",
            Self::SourceSubstitution => "source_substitution",
            Self::SpecSubstitution => "spec_substitution",
            Self::WeakenedPrecondition => "weakened_precondition",
            Self::ChangedOverflowSemantics => "changed_overflow_semantics",
            Self::NatU64Confusion => "nat_u64_confusion",
            Self::WrongProvider => "wrong_provider",
            Self::ToolchainDrift => "toolchain_drift",
            Self::MissingDependencyAudit => "missing_dependency_audit",
            Self::StaleGeneratedOutput => "stale_generated_output",
            Self::StaleProofTwin => "stale_proof_twin",
            Self::MissingObligation => "missing_obligation",
            Self::UnexpectedObligation => "unexpected_obligation",
            Self::UnexpectedAxiom => "unexpected_axiom",
            Self::UnresolvedProofHole => "unresolved_proof_hole",
            Self::ProofRejected => "proof_rejected",
            Self::InterruptedChecking => "interrupted_checking",
            Self::EmptyOutput => "empty_output",
            Self::MissingChecker => "missing_checker",
            Self::CommandDrift => "command_drift",
            Self::EvidenceOverclaim => "evidence_overclaim",
            Self::NonClaimDrift => "non_claim_drift",
            Self::ReceiptTamper => "receipt_tamper",
            Self::UnsupportedSurface => "unsupported_surface",
        }
    }
}

/// One typed validation diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Diagnostic {
    /// Diagnostic class.
    pub class: DiagnosticClass,
    /// Field or binding that failed.
    pub field: String,
    /// Bounded explanation.
    pub message: String,
}

/// Toolchain presence observed by the caller before evaluation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolchainAvailability {
    /// The reviewed closure is available.
    Available,
    /// The reviewed closure is absent or unverified.
    Unavailable,
}

/// Pure evaluation result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    /// Derived outcome.
    pub outcome: VermilionOutcome,
    /// Primary diagnostic class.
    pub diagnostic: DiagnosticClass,
    /// Every validation diagnostic.
    pub diagnostics: Vec<Diagnostic>,
    /// Refinement bridges that remain open obligations.
    pub open_obligations: Vec<String>,
    /// Admissible evidence role.
    pub evidence_disposition: EvidenceDisposition,
    /// Whether property evidence may be admitted.
    pub property_evidence_admitted: bool,
}

/// Load and validate the checked Vermilion provider profile.
///
/// # Errors
///
/// Returns an error when the embedded profile is malformed, violates its closed
/// contract, or does not match the reviewed identity.
pub fn selected_profile() -> Result<ProviderProfile, String> {
    let (profile, identity) = load_profile(PROFILE_JSON.as_bytes())?;
    if identity == PROFILE_IDENTITY {
        Ok(profile)
    } else {
        Err("embedded Vermilion provider profile identity drifted".to_owned())
    }
}

/// Parse, measure, and validate an exact provider profile.
///
/// # Errors
///
/// Returns an error when the supplied bytes are malformed or violate the closed
/// contract.
pub fn load_profile(bytes: &[u8]) -> Result<(ProviderProfile, String), String> {
    let profile: ProviderProfile =
        serde_json::from_slice(bytes).map_err(|error| format!("parse Vermilion provider profile: {error}"))?;
    let diagnostics = validate_profile(&profile);
    if diagnostics.is_empty() {
        Ok((profile, prefixed_identity(bytes)))
    } else {
        Err(format!(
            "invalid Vermilion provider profile: {}",
            diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect::<Vec<_>>().join("; ")
        ))
    }
}

/// Validate a loaded provider profile without external effects.
#[must_use]
pub fn validate_profile(profile: &ProviderProfile) -> Vec<Diagnostic> {
    validation::profile(profile)
}

/// Load and validate the checked pilot semantic contract.
///
/// # Errors
///
/// Returns an error when the embedded contract is malformed, violates its closed
/// contract, or does not match the reviewed identity.
pub fn selected_contract() -> Result<SemanticContract, String> {
    let (contract, identity) = load_contract(CONTRACT_JSON.as_bytes())?;
    if identity == CONTRACT_IDENTITY {
        Ok(contract)
    } else {
        Err("embedded Vermilion pilot contract identity drifted".to_owned())
    }
}

/// Parse, measure, and validate an exact pilot semantic contract.
///
/// # Errors
///
/// Returns an error when the supplied bytes are malformed or violate the closed
/// contract.
pub fn load_contract(bytes: &[u8]) -> Result<(SemanticContract, String), String> {
    let contract: SemanticContract =
        serde_json::from_slice(bytes).map_err(|error| format!("parse Vermilion pilot contract: {error}"))?;
    let diagnostics = validate_contract(&contract);
    if diagnostics.is_empty() {
        Ok((contract, prefixed_identity(bytes)))
    } else {
        Err(format!(
            "invalid Vermilion pilot contract: {}",
            diagnostics.iter().map(|diagnostic| diagnostic.message.as_str()).collect::<Vec<_>>().join("; ")
        ))
    }
}

/// Validate a loaded pilot contract without external effects.
#[must_use]
pub fn validate_contract(contract: &SemanticContract) -> Vec<Diagnostic> {
    validation::contract(contract)
}

/// Validate a receipt against the selected profile and contract.
#[must_use]
pub fn validate_receipt(receipt: &VermilionReceipt) -> Vec<Diagnostic> {
    match (selected_profile(), selected_contract()) {
        (Ok(profile), Ok(contract)) => {
            validation::receipt(receipt, &profile, PROFILE_IDENTITY, &contract, CONTRACT_IDENTITY)
        }
        (Err(error), _) | (_, Err(error)) => {
            vec![Diagnostic {
                class: DiagnosticClass::SchemaDrift,
                field: "embedded".to_owned(),
                message: error,
            }]
        }
    }
}

/// Evaluate a receipt against the selected profile, contract, and toolchain probe.
#[must_use]
pub fn evaluate(receipt: &VermilionReceipt, availability: ToolchainAvailability) -> Evaluation {
    match (selected_profile(), selected_contract()) {
        (Ok(profile), Ok(contract)) => {
            evaluate_against(receipt, &profile, PROFILE_IDENTITY, &contract, CONTRACT_IDENTITY, availability)
        }
        (Err(error), _) | (_, Err(error)) => Evaluation {
            outcome: VermilionOutcome::Failed,
            diagnostic: DiagnosticClass::SchemaDrift,
            diagnostics: vec![Diagnostic {
                class: DiagnosticClass::SchemaDrift,
                field: "embedded".to_owned(),
                message: error,
            }],
            open_obligations: Vec::new(),
            evidence_disposition: EvidenceDisposition::RecordedOnly,
            property_evidence_admitted: false,
        },
    }
}

/// Evaluate a receipt against explicit profile and contract identities.
#[must_use]
pub fn evaluate_against(
    receipt: &VermilionReceipt,
    profile: &ProviderProfile,
    profile_identity: &str,
    contract: &SemanticContract,
    contract_identity: &str,
    availability: ToolchainAvailability,
) -> Evaluation {
    let mut diagnostics = validation::profile(profile);
    diagnostics.extend(validation::contract(contract));
    if !is_b3(profile_identity) {
        diagnostics.push(diagnostic(DiagnosticClass::SchemaDrift, "profile_identity", "malformed profile identity"));
    }
    if !is_b3(contract_identity) {
        diagnostics.push(diagnostic(DiagnosticClass::SchemaDrift, "contract_identity", "malformed contract identity"));
    }
    diagnostics.extend(validation::receipt(receipt, profile, profile_identity, contract, contract_identity));
    let mut open_obligations = validation::open_obligations(receipt, contract);
    open_obligations.sort();
    open_obligations.dedup();
    if availability == ToolchainAvailability::Unavailable {
        return Evaluation {
            outcome: VermilionOutcome::Blocked,
            diagnostic: DiagnosticClass::MissingChecker,
            diagnostics,
            open_obligations,
            evidence_disposition: EvidenceDisposition::RecordedOnly,
            property_evidence_admitted: false,
        };
    }
    diagnostics.sort_by(|left, right| left.class.as_str().cmp(right.class.as_str()));
    diagnostics.dedup();
    let primary = primary_class(&diagnostics);
    let outcome = if diagnostics.is_empty() {
        VermilionOutcome::Supported
    } else {
        VermilionOutcome::Failed
    };
    let evidence_disposition = if outcome == VermilionOutcome::Supported
        && receipt.origin == ReceiptOrigin::ProviderRun
        && primary == DiagnosticClass::None
    {
        EvidenceDisposition::Property
    } else {
        EvidenceDisposition::RecordedOnly
    };
    Evaluation {
        outcome,
        diagnostic: primary,
        diagnostics,
        open_obligations,
        evidence_disposition,
        property_evidence_admitted: evidence_disposition == EvidenceDisposition::Property,
    }
}

fn primary_class(diagnostics: &[Diagnostic]) -> DiagnosticClass {
    const PRECEDENCE: [DiagnosticClass; 24] = [
        DiagnosticClass::MissingChecker,
        DiagnosticClass::InterruptedChecking,
        DiagnosticClass::EmptyOutput,
        DiagnosticClass::ReceiptTamper,
        DiagnosticClass::EvidenceOverclaim,
        DiagnosticClass::WrongProvider,
        DiagnosticClass::SchemaDrift,
        DiagnosticClass::SpecSubstitution,
        DiagnosticClass::SourceSubstitution,
        DiagnosticClass::NatU64Confusion,
        DiagnosticClass::ChangedOverflowSemantics,
        DiagnosticClass::WeakenedPrecondition,
        DiagnosticClass::MissingDependencyAudit,
        DiagnosticClass::ToolchainDrift,
        DiagnosticClass::CommandDrift,
        DiagnosticClass::StaleGeneratedOutput,
        DiagnosticClass::StaleProofTwin,
        DiagnosticClass::UnresolvedProofHole,
        DiagnosticClass::UnexpectedAxiom,
        DiagnosticClass::MissingObligation,
        DiagnosticClass::UnexpectedObligation,
        DiagnosticClass::ProofRejected,
        DiagnosticClass::UnsupportedSurface,
        DiagnosticClass::NonClaimDrift,
    ];
    PRECEDENCE
        .iter()
        .copied()
        .find(|class| diagnostics.iter().any(|diagnostic| diagnostic.class == *class))
        .unwrap_or(DiagnosticClass::None)
}

/// Compute the BLAKE3 identity of a receipt with its identity field cleared.
///
/// # Errors
///
/// Returns an error when the typed receipt cannot be serialized.
pub fn receipt_identity(receipt: &VermilionReceipt) -> Result<String, String> {
    let mut projection = receipt.clone();
    projection.receipt_identity.clear();
    let mut bytes =
        serde_json::to_vec_pretty(&projection).map_err(|error| format!("serialize Vermilion receipt: {error}"))?;
    bytes.push(b'\n');
    Ok(prefixed_identity(&bytes))
}

/// Return a canonical `b3:` identity for exact bytes.
#[must_use]
pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Return whether a string is one canonical lowercase `b3:` identity.
#[must_use]
pub fn is_b3(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub(crate) fn diagnostic(class: DiagnosticClass, field: &str, message: &str) -> Diagnostic {
    Diagnostic {
        class,
        field: field.to_owned(),
        message: message.to_owned(),
    }
}

mod comparison;
mod model;
mod validation;

#[cfg(test)]
mod tests;
