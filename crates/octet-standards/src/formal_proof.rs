//! Common formal-proof envelopes and provider-specific proof contracts.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::proof::PROOF_STATUS_FAILED;
use crate::proof::PROOF_STATUS_PASSED;
use crate::proof::VerusProofReceipt;

/// Schema version for the provider-neutral formal-proof envelope.
pub const FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION: &str = "octet-formal-proof-envelope/v1";
/// Provider identifier for the Aeneas-to-Lean proof path.
pub const AENEAS_LEAN_PROVIDER_ID: &str = "aeneas-lean";
const AENEAS_CONSUMER_POLICY_ID: &str = "octet.aeneas_consumer_contract.external-fixture";
/// Provider identifier for the explicit Verus-v1 compatibility adapter.
pub const VERUS_V1_PROVIDER_ID: &str = "verus-v1-compatibility";
/// Evidence tier emitted only for a passed, fully validated formal proof.
pub const FORMAL_PROOF_EVIDENCE_TIER: &str = "formal_proof";
/// Evidence tier used for failed or recorded-only provider observations.
pub const RECORDED_ONLY_EVIDENCE_TIER: &str = "recorded_only";
/// Claim class used by the Verus compatibility adapter.
pub const VERUS_COMPATIBILITY_CLAIM_CLASS: &str = "verus-v1-compatibility";
/// Claim class required for the first named Aeneas theorem fixture.
pub const AENEAS_NAMED_THEOREM_CLAIM_CLASS: &str = "named-theorem-over-generated-model";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS: &str = "does not prove Aeneas or Charon translation soundness";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS: &str = "does not prove Lean or verifier soundness";
/// Aeneas non-claim required on every common envelope.
pub const AENEAS_NON_CLAIM_WHOLE_PROGRAM: &str = "does not prove whole-program correctness";

const LITTLE_ENDIAN: &str = "little";
const BIG_ENDIAN: &str = "big";
const THEOREM_DECLARATION_PREFIX: &str = "theorem ";
const THEOREM_BODY_SEPARATOR: &str = ":=";
const THEOREM_TYPE_SEPARATOR: &str = ":";
const VACUOUS_THEOREM_IDENTIFIER: &str = "True";
const FORBIDDEN_THEOREM_DECLARATIONS: &[&str] = &["admit", "sorry"];
const AENEAS_PROVIDER_PROFILE_JSON: &str =
    include_str!("../../../standards/proof-rails/aeneas-lean/generated/profile.json");
const REQUIRED_AENEAS_FIELDS: &[&str] = &[
    "aeneas_release",
    "charon_revision",
    "ocaml_toolchain",
    "lean_toolchain",
    "aeneas_lean_library",
];

/// Provider-neutral status for one formal-proof execution.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalProofStatus {
    /// The provider completed successfully and passed the configured checker.
    Passed,
    /// The provider produced a failure or a non-property observation.
    Failed,
}

impl FormalProofStatus {
    /// Return the stable serialized status spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => PROOF_STATUS_PASSED,
            Self::Failed => PROOF_STATUS_FAILED,
        }
    }
}

/// Provider-neutral evidence tier for one formal-proof envelope.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FormalProofEvidenceTier {
    /// A passed theorem receipt admitted as formal-proof evidence.
    FormalProof,
    /// A bounded observation that is not admitted as a formal proof.
    RecordedOnly,
}

impl FormalProofEvidenceTier {
    /// Return the stable serialized evidence-tier spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FormalProof => FORMAL_PROOF_EVIDENCE_TIER,
            Self::RecordedOnly => RECORDED_ONLY_EVIDENCE_TIER,
        }
    }
}

/// BLAKE3 identity for one locally reviewable proof artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofArtifactIdentity {
    /// Repo-relative artifact locator.
    pub locator: String,
    /// BLAKE3 digest of the exact artifact bytes.
    pub blake3: String,
}

/// The requirement and claim classification carried by a formal-proof envelope.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofClaim {
    /// Stable theorem or property identifier.
    pub theorem_or_property_id: String,
    /// Requirement or policy identifier authorizing the claim.
    pub requirement_id: String,
    /// Reviewed claim classification.
    pub claim_class: String,
}

/// The common formal-proof envelope shared by closed provider payloads.
// r[impl octet.aeneas_lean_proof_rail.provider_contract]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FormalProofEnvelope {
    /// Envelope schema version.
    pub schema_version: String,
    /// Provider identifier duplicated for explicit cross-checking.
    pub provider_id: String,
    /// Owning policy identifier.
    pub policy_id: String,
    /// Stable rail identifier.
    pub rail_id: String,
    /// Scoped checked-source label.
    pub checked_scope: String,
    /// Provider execution status.
    pub status: FormalProofStatus,
    /// Evidence tier assigned after validation.
    pub evidence_tier: FormalProofEvidenceTier,
    /// Requirement and theorem/property identity.
    pub claim: FormalProofClaim,
    /// BLAKE3 identity of the provider package lock.
    pub package_lock_identity: String,
    /// Source artifacts bound by the proof.
    pub source_bindings: Vec<FormalProofArtifactIdentity>,
    /// Other proof-stage artifacts bound by the proof.
    pub artifact_identities: Vec<FormalProofArtifactIdentity>,
    /// Explicit trust-boundary statements.
    pub trust_boundaries: Vec<String>,
    /// Explicit non-claim statements.
    pub non_claims: Vec<String>,
    /// Optional exact-source Aeneas consumer binding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumer: Option<crate::aeneas_consumer::ConsumerReceiptBinding>,
    /// Closed provider-specific payload.
    pub provider: FormalProofProvider,
}

/// Closed provider payload carried by the common envelope.
#[expect(
    clippy::large_enum_variant,
    reason = "The closed provider enum retains typed payloads for explicit serialization and validation."
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum FormalProofProvider {
    /// An explicitly adapted, unchanged Verus-v1 receipt.
    #[serde(rename = "verus-v1")]
    VerusV1(VerusCompatibilityPayload),
    /// The typed Aeneas-to-Lean provider contract.
    #[serde(rename = "aeneas-lean")]
    AeneasLean(AeneasLeanProofPayload),
}

impl FormalProofProvider {
    /// Return the provider identifier represented by this payload.
    #[must_use]
    pub const fn provider_id(&self) -> &'static str {
        match self {
            Self::VerusV1(_) => VERUS_V1_PROVIDER_ID,
            Self::AeneasLean(_) => AENEAS_LEAN_PROVIDER_ID,
        }
    }
}

/// Compatibility payload that preserves the original Verus-v1 receipt structure.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerusCompatibilityPayload {
    /// The unchanged Verus-v1 receipt being adapted.
    pub receipt: VerusProofReceipt,
}

/// Bounded process limits for Aeneas, Charon, and Lean execution.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasExecutionLimits {
    /// Maximum stage execution time in milliseconds.
    pub timeout_ms: u64,
    /// Maximum captured stdout bytes per stage.
    pub stdout_max_bytes: usize,
    /// Maximum captured stderr bytes per stage.
    pub stderr_max_bytes: usize,
}

/// Effective Charon extraction policy bound by an Aeneas proof payload.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CharonExtractionPolicy {
    /// Whether Charon reported extraction errors.
    pub has_errors: bool,
    /// Whether Charon skipped Rust borrow checking.
    pub skip_borrowck: bool,
    /// Target triple used for extraction.
    pub target_triple: String,
    /// Target pointer width in bits.
    pub pointer_width_bits: u16,
    /// Target byte order.
    pub endianness: String,
    /// Reviewed MIR extraction start stage.
    pub mir_start_stage: String,
    /// Reviewed drop-handling policy.
    pub drop_policy: String,
    /// Reviewed Charon `start_from` setting.
    pub start_from: String,
    /// Exact include rules.
    pub include_rules: Vec<String>,
    /// Exact exclude rules.
    pub exclude_rules: Vec<String>,
    /// Exact opaque rules.
    pub opaque_rules: Vec<String>,
    /// Ordered transformation settings.
    pub transformation_settings: Vec<String>,
    /// Serialized LLBC format identifier.
    pub serialization_format: String,
}

/// Validate an observed Charon extraction policy against the reviewed policy.
// r[impl octet.aeneas_lean_proof_rail.charon_contract]
#[must_use]
pub fn validate_charon_extraction_contract(
    observed: &CharonExtractionPolicy,
    reviewed: &CharonExtractionPolicy,
) -> Vec<String> {
    let mut diagnostics = validate_charon_policy_shape("extraction_policy", observed);
    diagnostics.extend(validate_charon_policy_shape("reviewed_extraction_policy", reviewed));
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.target_triple",
        &observed.target_triple,
        &reviewed.target_triple,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.pointer_width_bits",
        &observed.pointer_width_bits,
        &reviewed.pointer_width_bits,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.endianness",
        &observed.endianness,
        &reviewed.endianness,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.mir_start_stage",
        &observed.mir_start_stage,
        &reviewed.mir_start_stage,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.drop_policy",
        &observed.drop_policy,
        &reviewed.drop_policy,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.start_from",
        &observed.start_from,
        &reviewed.start_from,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.include_rules",
        &observed.include_rules,
        &reviewed.include_rules,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.exclude_rules",
        &observed.exclude_rules,
        &reviewed.exclude_rules,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.opaque_rules",
        &observed.opaque_rules,
        &reviewed.opaque_rules,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.transformation_settings",
        &observed.transformation_settings,
        &reviewed.transformation_settings,
    );
    push_contract_mismatch(
        &mut diagnostics,
        "extraction_policy.serialization_format",
        &observed.serialization_format,
        &reviewed.serialization_format,
    );
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_charon_policy_shape(field: &str, policy: &CharonExtractionPolicy) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if policy.has_errors {
        diagnostics.push(format!("{field}.has_errors must be false"));
    }
    if policy.skip_borrowck {
        diagnostics.push(format!("{field}.skip_borrowck must be false"));
    }
    push_non_empty(&mut diagnostics, &format!("{field}.target_triple"), &policy.target_triple);
    if policy.pointer_width_bits == 0 {
        diagnostics.push(format!("{field}.pointer_width_bits must be greater than zero"));
    }
    if !matches!(policy.endianness.as_str(), LITTLE_ENDIAN | BIG_ENDIAN) {
        diagnostics.push(format!("{field}.endianness must be `{LITTLE_ENDIAN}` or `{BIG_ENDIAN}`"));
    }
    push_non_empty(&mut diagnostics, &format!("{field}.mir_start_stage"), &policy.mir_start_stage);
    push_non_empty(&mut diagnostics, &format!("{field}.drop_policy"), &policy.drop_policy);
    push_non_empty(&mut diagnostics, &format!("{field}.start_from"), &policy.start_from);
    push_no_blank_strings(&mut diagnostics, &format!("{field}.include_rules"), &policy.include_rules);
    push_no_blank_strings(&mut diagnostics, &format!("{field}.exclude_rules"), &policy.exclude_rules);
    push_no_blank_strings(&mut diagnostics, &format!("{field}.opaque_rules"), &policy.opaque_rules);
    push_non_empty_strings(
        &mut diagnostics,
        &format!("{field}.transformation_settings"),
        &policy.transformation_settings,
    );
    push_non_empty(&mut diagnostics, &format!("{field}.serialization_format"), &policy.serialization_format);
    diagnostics
}

fn push_contract_mismatch<T: PartialEq>(diagnostics: &mut Vec<String>, field: &str, observed: &T, reviewed: &T) {
    if observed != reviewed {
        diagnostics.push(format!("{field} drifted from reviewed extraction contract"));
    }
}

#[derive(Deserialize)]
struct AeneasProviderProfile {
    aeneas_release: String,
    charon_revision: String,
    ocaml_toolchain: String,
    lean_toolchain: String,
    aeneas_lean_library: String,
    extraction_policy: CharonExtractionPolicy,
}

fn reviewed_aeneas_provider_profile() -> Result<AeneasProviderProfile, String> {
    serde_json::from_str::<AeneasProviderProfile>(AENEAS_PROVIDER_PROFILE_JSON)
        .map_err(|error| format!("embedded Aeneas provider profile is invalid: {error}"))
}

fn validate_aeneas_provider_pins(payload: &AeneasLeanProofPayload, reviewed: &AeneasProviderProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_contract_mismatch(&mut diagnostics, "aeneas_release", &payload.aeneas_release, &reviewed.aeneas_release);
    push_contract_mismatch(&mut diagnostics, "charon_revision", &payload.charon_revision, &reviewed.charon_revision);
    push_contract_mismatch(&mut diagnostics, "ocaml_toolchain", &payload.ocaml_toolchain, &reviewed.ocaml_toolchain);
    push_contract_mismatch(&mut diagnostics, "lean_toolchain", &payload.lean_toolchain, &reviewed.lean_toolchain);
    push_contract_mismatch(
        &mut diagnostics,
        "aeneas_lean_library",
        &payload.aeneas_lean_library,
        &reviewed.aeneas_lean_library,
    );
    diagnostics
}

/// Declared trust-input policy for Aeneas theorem admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasAssumptionPolicy {
    /// External model identifiers allowed by the reviewed profile.
    pub external_models: Vec<String>,
    /// Axiom or assumption identifiers allowed by the reviewed profile.
    pub axioms: Vec<String>,
    /// Whether every trust input must be declared before admission.
    pub requires_complete_declaration: bool,
}

/// Distinguishes generated definitions from hand-maintained theorem source.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasTheoremSourceDisposition {
    /// The theorem source was emitted by the translation pipeline.
    Generated,
    /// The theorem source was reviewed and maintained by Octet.
    Maintained,
}

/// The deterministic theorem-adequacy outcome recorded before Lean checking.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasTheoremAdequacyDecision {
    /// The statement is non-vacuous for its approved claim class.
    NonVacuous,
    /// The statement does not establish the selected claim class.
    Vacuous,
    /// The statement was rejected by the adequacy review.
    Rejected,
}

/// The typed result of the named Lean theorem check.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasLeanCheckStatus {
    /// Lean checked the selected theorem successfully.
    Passed,
    /// Lean rejected the selected theorem or its generated model.
    Failed,
}

/// Complete trust inputs declared for one theorem admission.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasTrustClosure {
    /// Whether the provider observed and declared the complete trust closure.
    pub complete: bool,
    /// External model identifiers included in the closure.
    pub declared_external_models: Vec<String>,
    /// Axiom or assumption identifiers included in the closure.
    pub declared_axioms: Vec<String>,
    /// Opaque generated definitions included in the closure.
    pub declared_opaque_definitions: Vec<String>,
    /// Admitted declarations are retained for rejection diagnostics.
    pub declared_admitted_declarations: Vec<String>,
    /// Trust inputs found by the checker but absent from the declaration.
    pub undeclared_inputs: Vec<String>,
}

/// The deterministic adequacy observations for one named theorem.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasTheoremAdequacy {
    /// Reviewed adequacy decision for the theorem statement.
    pub decision: AeneasTheoremAdequacyDecision,
    /// Forbidden declarations such as `sorry` or `admit` found in the source.
    pub forbidden_declarations: Vec<String>,
}

/// Assess a bounded, already-loaded theorem source before provider admission.
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
#[must_use]
pub fn assess_aeneas_theorem_source(source: &str) -> AeneasTheoremAdequacy {
    let statement = source.lines().find_map(|line| {
        let declaration = line.trim_start().strip_prefix(THEOREM_DECLARATION_PREFIX)?;
        let header = declaration.split_once(THEOREM_BODY_SEPARATOR)?.0;
        header.rsplit_once(THEOREM_TYPE_SEPARATOR).map(|(_, value)| value.trim())
    });
    let decision = match statement {
        Some(value) if true_style_statement(value) => AeneasTheoremAdequacyDecision::Vacuous,
        Some(value) if !value.is_empty() => AeneasTheoremAdequacyDecision::NonVacuous,
        _ => AeneasTheoremAdequacyDecision::Rejected,
    };
    let forbidden_declarations = FORBIDDEN_THEOREM_DECLARATIONS
        .iter()
        .filter(|declaration| {
            source
                .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .any(|token| token == **declaration)
        })
        .map(|declaration| (*declaration).to_owned())
        .collect();
    AeneasTheoremAdequacy {
        decision,
        forbidden_declarations,
    }
}

fn true_style_statement(statement: &str) -> bool {
    let mut identifiers = statement
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|identifier| !identifier.is_empty());
    let Some(first) = identifiers.next() else {
        return false;
    };
    first == VACUOUS_THEOREM_IDENTIFIER && identifiers.all(|identifier| identifier == VACUOUS_THEOREM_IDENTIFIER)
}

/// Bind observed theorem bytes to an independently reviewed digest.
///
/// # Errors
///
/// Returns an error when the approved digest is absent or differs from the observed digest.
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
pub fn validate_aeneas_theorem_statement_approval(
    expected_blake3: &str,
    observed_blake3: &str,
) -> Result<String, String> {
    let expected_blake3 = expected_blake3.trim();
    if expected_blake3.is_empty() {
        return Err("Aeneas rail must declare approved_theorem_statement_blake3".to_owned());
    }
    if expected_blake3 != observed_blake3 {
        return Err(format!(
            "Aeneas theorem statement BLAKE3 mismatch: expected `{expected_blake3}`, observed `{observed_blake3}`"
        ));
    }
    Ok(expected_blake3.to_owned())
}

/// The exact theorem result reported by the Lean checker.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasLeanCheckResult {
    /// The named theorem selected by the checker.
    pub theorem_id: String,
    /// The checker result for the selected theorem.
    pub status: AeneasLeanCheckStatus,
}

/// Admission facts required before an Aeneas result can become formal proof evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasTheoremAdmission {
    /// Stable name of the theorem checked over the generated model.
    pub theorem_id: String,
    /// Requirement authorizing this theorem claim.
    pub requirement_id: String,
    /// Explicit claim class for the theorem.
    pub claim_class: String,
    /// Whether the theorem source is generated or maintained.
    pub source_disposition: AeneasTheoremSourceDisposition,
    /// BLAKE3 of the approved theorem-statement artifact.
    pub approved_statement_blake3: String,
    /// Whether the named theorem was present in the selected source.
    pub named_theorem_present: bool,
    /// Whether review approved the exact statement identity.
    pub statement_approved: bool,
    /// Deterministic adequacy observations.
    pub adequacy: AeneasTheoremAdequacy,
    /// Complete declared trust closure.
    pub trust_closure: AeneasTrustClosure,
    /// Result of checking this exact theorem with Lean.
    pub lean_check: AeneasLeanCheckResult,
}

/// Recorded-only semantic-foundation reference metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticReference {
    /// Exact semantic-reference repository revision.
    pub revision: String,
    /// License identity for the referenced material.
    pub license: String,
    /// Source sentinel used to identify the reviewed revision.
    pub source_sentinel: String,
    /// Must remain true because the reference is not proof authority.
    pub recorded_only: bool,
}

/// Outcome of Aeneas source preflight before translation or theorem checking.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasPreflightDisposition {
    /// The selected source is within the reviewed provider surface.
    Ready,
    /// The selected source contains a construct the provider does not admit.
    Blocked,
    /// The provider could not produce a usable translation observation.
    Failed,
}

impl AeneasPreflightDisposition {
    /// Return the stable serialized disposition spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Blocked => "blocked",
            Self::Failed => "failed",
        }
    }
}

/// Explicit source or translation input that prevents proof promotion.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
// r[impl octet.aeneas_lean_proof_rail.unsupported_surface]
pub enum AeneasUnsupportedInput {
    /// Unsafe Rust is outside the reviewed provider surface.
    UnsafeRust,
    /// Concurrent Rust is outside the reviewed provider surface.
    Concurrency,
    /// Unsupported loop-control transfer is outside the reviewed provider surface.
    UnsupportedLoopControl,
    /// An unresolved external definition prevents a closed translation.
    UnresolvedExternal,
    /// A translation failure prevents a usable provider observation.
    TranslationFailure,
}

impl AeneasUnsupportedInput {
    /// Return the stable serialized input spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsafeRust => "unsafe_rust",
            Self::Concurrency => "concurrency",
            Self::UnsupportedLoopControl => "unsupported_loop_control",
            Self::UnresolvedExternal => "unresolved_external",
            Self::TranslationFailure => "translation_failure",
        }
    }
}

/// Pure preflight observations passed from the provider shell to validation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasPreflight {
    /// Explicit unsupported or failed inputs observed for the selected source.
    pub unsupported_inputs: Vec<AeneasUnsupportedInput>,
    /// The disposition selected by the provider shell.
    pub disposition: AeneasPreflightDisposition,
}

/// Classify preflight inputs without process, filesystem, or environment access.
#[must_use]
pub const fn classify_aeneas_preflight(inputs: &[AeneasUnsupportedInput]) -> AeneasPreflightDisposition {
    let mut index = 0;
    while index < inputs.len() {
        if matches!(inputs[index], AeneasUnsupportedInput::TranslationFailure) {
            return AeneasPreflightDisposition::Failed;
        }
        index += 1;
    }
    if inputs.is_empty() {
        AeneasPreflightDisposition::Ready
    } else {
        AeneasPreflightDisposition::Blocked
    }
}

/// Validate that a recorded preflight disposition matches its explicit inputs.
// r[impl octet.aeneas_lean_proof_rail.unsupported_surface]
#[must_use]
pub fn validate_aeneas_preflight(preflight: &AeneasPreflight) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let expected = classify_aeneas_preflight(&preflight.unsupported_inputs);
    if preflight.disposition != expected {
        diagnostics.push(format!(
            "preflight.disposition expected `{}` for recorded inputs but got `{}`",
            expected.as_str(),
            preflight.disposition.as_str()
        ));
    }
    diagnostics
}

/// Closed Aeneas-to-Lean provider payload.
// r[impl octet.aeneas_lean_proof_rail.provider_contract]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasLeanProofPayload {
    /// Exact Aeneas release identity.
    pub aeneas_release: String,
    /// Exact Charon revision required by the Aeneas release.
    pub charon_revision: String,
    /// Exact OCaml toolchain identity.
    pub ocaml_toolchain: String,
    /// Exact Lean toolchain identity.
    pub lean_toolchain: String,
    /// Exact Aeneas Lean library identity.
    pub aeneas_lean_library: String,
    /// Reviewed safe-Rust source surface admitted by the provider.
    pub checked_source_surface: Vec<String>,
    /// Pure source and translation preflight outcome.
    pub preflight: AeneasPreflight,
    /// Shell-free provider command argv.
    pub command_argv: Vec<String>,
    /// Bounded execution limits.
    pub limits: AeneasExecutionLimits,
    /// Explicit environment variable allowlist.
    pub environment_allowlist: Vec<String>,
    /// Effective Charon extraction contract.
    pub extraction_policy: CharonExtractionPolicy,
    /// Declared model and assumption policy.
    pub assumption_policy: AeneasAssumptionPolicy,
    /// Optional theorem admission; absent means translation-only recorded evidence.
    pub theorem_admission: Option<AeneasTheoremAdmission>,
    /// Ordered, hash-linked artifacts for every provider stage.
    // r[impl octet.aeneas_lean_proof_rail.stage_chain]
    pub stage_chain: AeneasStageChain,
    /// Recorded-only semantic foundation reference.
    pub semantic_reference: SemanticReference,
}

/// Schema version for the ordered Aeneas stage-chain contract.
pub const AENEAS_STAGE_CHAIN_SCHEMA_VERSION: &str = "octet-aeneas-lean-stage-chain/v1";

const AENEAS_STAGE_COUNT: usize = 6;
const BLAKE3_HEX_LENGTH: usize = 64;

/// Ordered Aeneas pipeline stages.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasStageKind {
    /// Checked Rust source and selected item scope.
    RustSource,
    /// Charon identity, effective options, target metadata, and LLBC output.
    CharonExtraction,
    /// Aeneas identity, command configuration, and translation manifest.
    AeneasTranslation,
    /// Generated Lean definitions and imported external model inventory.
    GeneratedLean,
    /// Hand-maintained theorem source and approved statement identity.
    Theorem,
    /// Lean package/toolchain identity and final checker result.
    LeanCheck,
}

impl AeneasStageKind {
    /// Return the stable serialized stage name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RustSource => "rust_source",
            Self::CharonExtraction => "charon_extraction",
            Self::AeneasTranslation => "aeneas_translation",
            Self::GeneratedLean => "generated_lean",
            Self::Theorem => "theorem",
            Self::LeanCheck => "lean_check",
        }
    }
}

const EXPECTED_STAGE_KINDS: [AeneasStageKind; AENEAS_STAGE_COUNT] = [
    AeneasStageKind::RustSource,
    AeneasStageKind::CharonExtraction,
    AeneasStageKind::AeneasTranslation,
    AeneasStageKind::GeneratedLean,
    AeneasStageKind::Theorem,
    AeneasStageKind::LeanCheck,
];

/// Typed role for a stage-chain artifact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AeneasStageArtifactRole {
    /// Provider profile identity.
    ProviderProfile,
    /// Checked Rust source bytes or source manifest.
    RustSource,
    /// Effective serialized Charon extraction contract.
    CharonOptions,
    /// Charon argv identity.
    CharonCommand,
    /// Emitted LLBC bytes.
    Llbc,
    /// Aeneas argv/backend identity.
    AeneasCommand,
    /// Aeneas `translation.json` manifest.
    TranslationManifest,
    /// Generated Lean definitions and file inventory.
    GeneratedLean,
    /// Explicit inventory of imported external models.
    ExternalModelInventory,
    /// Hand-maintained theorem source.
    TheoremSource,
    /// Approved theorem statement identity.
    TheoremStatement,
    /// Deterministic theorem-adequacy decision.
    TheoremAdequacy,
    /// Complete theorem trust-closure identity.
    TrustClosure,
    /// Lean package/toolchain identity.
    LeanToolchain,
    /// Aeneas Lean library identity.
    AeneasLeanLibrary,
    /// Lean checker argv identity.
    LeanCommand,
    /// Lean checker/package-lock result identity.
    CheckerResult,
    /// Aeneas package-lock identity.
    PackageLock,
}

impl AeneasStageArtifactRole {
    /// Return the stable serialized role name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderProfile => "provider_profile",
            Self::RustSource => "rust_source",
            Self::CharonOptions => "charon_options",
            Self::CharonCommand => "charon_command",
            Self::Llbc => "llbc",
            Self::AeneasCommand => "aeneas_command",
            Self::TranslationManifest => "translation_manifest",
            Self::GeneratedLean => "generated_lean",
            Self::ExternalModelInventory => "external_model_inventory",
            Self::TheoremSource => "theorem_source",
            Self::TheoremStatement => "theorem_statement",
            Self::TheoremAdequacy => "theorem_adequacy",
            Self::TrustClosure => "trust_closure",
            Self::LeanToolchain => "lean_toolchain",
            Self::AeneasLeanLibrary => "aeneas_lean_library",
            Self::LeanCommand => "lean_command",
            Self::CheckerResult => "checker_result",
            Self::PackageLock => "package_lock",
        }
    }
}

/// BLAKE3-bound typed identity for one Aeneas stage artifact.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStageArtifactIdentity {
    /// Semantic role of the artifact in the stage chain.
    pub role: AeneasStageArtifactRole,
    /// Repo-relative artifact locator.
    pub locator: String,
    /// Lowercase BLAKE3 digest of exact artifact bytes.
    pub blake3: String,
}

/// One ordered stage with exact input/output artifact links.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStage {
    /// Ordered stage role.
    pub kind: AeneasStageKind,
    /// Exact artifacts consumed from the previous stage.
    pub input_artifacts: Vec<AeneasStageArtifactIdentity>,
    /// Exact artifacts emitted for the next stage.
    pub output_artifacts: Vec<AeneasStageArtifactIdentity>,
}

/// Complete ordered, hash-linked Aeneas stage chain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasStageChain {
    /// Stage-chain schema version.
    pub schema_version: String,
    /// BLAKE3-bound provider profile identity.
    pub provider_profile: AeneasStageArtifactIdentity,
    /// BLAKE3-bound package-lock identity.
    pub package_lock: AeneasStageArtifactIdentity,
    /// Exactly six ordered stages from Rust source to Lean result.
    pub stages: Vec<AeneasStage>,
}

/// Schema version for the Aeneas translation-manifest contract.
// r[impl octet.aeneas_lean_proof_rail.translation_manifest]
pub const AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION: &str = "octet-aeneas-lean-translation-manifest/v1";

/// A source span preserved from the current Charon extraction run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasSourceSpan {
    /// Repo-relative source locator.
    pub locator: String,
    /// Inclusive start byte in the source envelope.
    pub start_byte: usize,
    /// Exclusive end byte in the source envelope.
    pub end_byte: usize,
}

/// A selected LLBC declaration used as the current-run join target.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasLlbcDeclaration {
    /// Charon-local declaration identifier; this is not a durable identity.
    pub def_id: String,
    /// Rust declaration name preserved by Charon.
    pub rust_name: String,
    /// Rust source span preserved by Charon.
    pub rust_span: AeneasSourceSpan,
}

/// A generated Lean file measured by the imperative shell.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasGeneratedLeanFile {
    /// Repo-relative generated Lean file locator.
    pub locator: String,
    /// Lowercase BLAKE3 digest of the measured file bytes.
    pub blake3: String,
}

/// One declaration link recorded by Aeneas `translation.json`.
#[expect(
    clippy::struct_excessive_bools,
    reason = "translation.json preserves independent boolean declaration metadata"
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasTranslationDeclaration {
    /// Charon-local declaration identifier used only for this run's join.
    pub def_id: String,
    /// Rust declaration name copied from the selected LLBC declaration.
    pub rust_name: String,
    /// Rust source span copied from the selected LLBC declaration.
    pub rust_span: AeneasSourceSpan,
    /// Generated Lean declaration name.
    pub lean_name: String,
    /// Generated Lean file containing the declaration.
    pub lean_file: String,
    /// Whether the generated declaration is opaque.
    pub is_opaque: bool,
    /// Whether the generated declaration may fail.
    pub can_fail: bool,
    /// Whether the generated declaration may diverge.
    pub can_diverge: bool,
    /// Whether the generated declaration is recursive.
    pub is_recursive: bool,
    /// Current-run parent declaration for loop metadata, when present.
    pub loop_parent_def_id: Option<String>,
    /// Durable identity must be derived from bound content, never from `def_id`.
    pub durable_identity: Option<String>,
}

/// Typed Aeneas translation manifest and its measured generated-file inventory.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AeneasTranslationManifest {
    /// Translation-manifest schema version.
    pub schema_version: String,
    /// Selected LLBC-to-Lean declaration links.
    pub declarations: Vec<AeneasTranslationDeclaration>,
    /// Every generated Lean file named by the translation output.
    pub generated_files: Vec<AeneasGeneratedLeanFile>,
}

/// Validate translation declaration links and generated-file identities.
// r[impl octet.aeneas_lean_proof_rail.translation_manifest]
#[must_use]
pub fn validate_aeneas_translation_manifest(
    manifest: &AeneasTranslationManifest,
    llbc_declarations: &[AeneasLlbcDeclaration],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_exact(
        &mut diagnostics,
        "translation_manifest.schema_version",
        AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION,
        &manifest.schema_version,
    );
    if manifest.declarations.is_empty() {
        diagnostics.push("translation_manifest.declarations must not be empty".to_owned());
    }
    if llbc_declarations.is_empty() {
        diagnostics.push("translation_manifest.llbc_declarations must not be empty".to_owned());
    }
    if manifest.generated_files.is_empty() {
        diagnostics.push("translation_manifest.generated_files must not be empty".to_owned());
    }

    let llbc_ids = validate_llbc_declarations(llbc_declarations, &mut diagnostics);
    let generated_files = validate_generated_lean_files(&manifest.generated_files, &mut diagnostics);
    let mut manifest_ids = BTreeSet::new();
    let mut lean_declarations = BTreeSet::new();
    for (index, declaration) in manifest.declarations.iter().enumerate() {
        let field = format!("translation_manifest.declarations[{index}]");
        push_non_empty(&mut diagnostics, &format!("{field}.def_id"), &declaration.def_id);
        push_non_empty(&mut diagnostics, &format!("{field}.rust_name"), &declaration.rust_name);
        validate_source_span(&format!("{field}.rust_span"), &declaration.rust_span, &mut diagnostics);
        push_non_empty(&mut diagnostics, &format!("{field}.lean_name"), &declaration.lean_name);
        push_non_empty(&mut diagnostics, &format!("{field}.lean_file"), &declaration.lean_file);
        if !manifest_ids.insert(declaration.def_id.clone()) {
            diagnostics.push(format!("{field}.def_id duplicates a current-run declaration"));
        }
        if !lean_declarations.insert((declaration.lean_file.clone(), declaration.lean_name.clone())) {
            diagnostics.push(format!("{field} duplicates a generated Lean declaration"));
        }
        if declaration.durable_identity.is_some() {
            diagnostics
                .push(format!("{field}.durable_identity must be absent because def_id is a within-run join key only"));
        }
        match llbc_declarations.iter().find(|candidate| candidate.def_id == declaration.def_id) {
            Some(llbc) => {
                if declaration.rust_name != llbc.rust_name {
                    diagnostics
                        .push(format!("{field}.rust_name does not match LLBC declaration `{}`", declaration.def_id));
                }
                if declaration.rust_span != llbc.rust_span {
                    diagnostics
                        .push(format!("{field}.rust_span does not match LLBC declaration `{}`", declaration.def_id));
                }
            }
            None => diagnostics.push(format!("{field}.def_id is missing from the current LLBC run")),
        }
        if !generated_files.contains(&declaration.lean_file) {
            diagnostics.push(format!("{field}.lean_file is missing from generated_files inventory"));
        }
        if let Some(loop_parent_def_id) = &declaration.loop_parent_def_id {
            push_non_empty(&mut diagnostics, &format!("{field}.loop_parent_def_id"), loop_parent_def_id);
            if !llbc_ids.contains(loop_parent_def_id) {
                diagnostics.push(format!("{field}.loop_parent_def_id is missing from the current LLBC run"));
            }
        }
    }
    for llbc in llbc_declarations {
        if !manifest_ids.contains(&llbc.def_id) {
            diagnostics.push(format!("translation_manifest is missing selected LLBC declaration `{}`", llbc.def_id));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_llbc_declarations(
    declarations: &[AeneasLlbcDeclaration],
    diagnostics: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for (index, declaration) in declarations.iter().enumerate() {
        let field = format!("translation_manifest.llbc_declarations[{index}]");
        push_non_empty(diagnostics, &format!("{field}.def_id"), &declaration.def_id);
        push_non_empty(diagnostics, &format!("{field}.rust_name"), &declaration.rust_name);
        validate_source_span(&format!("{field}.rust_span"), &declaration.rust_span, diagnostics);
        if !ids.insert(declaration.def_id.clone()) {
            diagnostics.push(format!("{field}.def_id duplicates a current-run declaration"));
        }
    }
    ids
}

fn validate_generated_lean_files(files: &[AeneasGeneratedLeanFile], diagnostics: &mut Vec<String>) -> BTreeSet<String> {
    let mut locators = BTreeSet::new();
    for (index, file) in files.iter().enumerate() {
        let field = format!("translation_manifest.generated_files[{index}]");
        push_non_empty(diagnostics, &format!("{field}.locator"), &file.locator);
        if !is_blake3_hex(&file.blake3) {
            diagnostics.push(format!("{field}.blake3 must be a lowercase 64-character BLAKE3 hex digest"));
        }
        if !locators.insert(file.locator.clone()) {
            diagnostics.push(format!("{field}.locator duplicates a generated file"));
        }
    }
    locators
}

fn validate_source_span(field: &str, span: &AeneasSourceSpan, diagnostics: &mut Vec<String>) {
    push_non_empty(diagnostics, &format!("{field}.locator"), &span.locator);
    if span.start_byte > span.end_byte {
        diagnostics.push(format!("{field} has start_byte after end_byte"));
    }
}

/// Return the stage names in their recorded order without performing I/O.
#[must_use]
pub fn aeneas_stage_chain_stage_order(chain: &AeneasStageChain) -> Vec<String> {
    chain.stages.iter().map(|stage| stage.kind.as_str().to_owned()).collect()
}

/// Validate ordered stage roles, BLAKE3 identities, and adjacent stage links.
#[must_use]
pub fn validate_aeneas_stage_chain(chain: &AeneasStageChain) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_exact(
        &mut diagnostics,
        "stage_chain.schema_version",
        AENEAS_STAGE_CHAIN_SCHEMA_VERSION,
        &chain.schema_version,
    );
    validate_stage_artifact(
        "stage_chain.provider_profile",
        &chain.provider_profile,
        AeneasStageArtifactRole::ProviderProfile,
        &mut diagnostics,
    );
    validate_stage_artifact(
        "stage_chain.package_lock",
        &chain.package_lock,
        AeneasStageArtifactRole::PackageLock,
        &mut diagnostics,
    );
    if chain.provider_profile == chain.package_lock {
        diagnostics.push("stage_chain provider_profile and package_lock must be distinct".to_owned());
    }
    if chain.stages.len() != AENEAS_STAGE_COUNT {
        diagnostics
            .push(format!("stage_chain.stages expected {AENEAS_STAGE_COUNT} stages but got {}", chain.stages.len()));
    }

    let mut output_identities = BTreeSet::new();
    for (index, stage) in chain.stages.iter().enumerate() {
        let stage_prefix = format!("stage_chain.stages[{index}]");
        let expected_kind = EXPECTED_STAGE_KINDS.get(index).copied();
        if expected_kind != Some(stage.kind) {
            let expected_name = expected_kind.map_or("missing", AeneasStageKind::as_str);
            diagnostics
                .push(format!("{stage_prefix}.kind expected `{expected_name}` but got `{}`", stage.kind.as_str()));
        }
        if index != 0 || !stage.input_artifacts.is_empty() {
            validate_stage_artifact_list(
                &format!("{stage_prefix}.input_artifacts"),
                &stage.input_artifacts,
                &mut diagnostics,
            );
        }
        validate_stage_artifact_list(
            &format!("{stage_prefix}.output_artifacts"),
            &stage.output_artifacts,
            &mut diagnostics,
        );
        let expected_roles = expected_output_roles(stage.kind);
        let actual_roles = stage.output_artifacts.iter().map(|artifact| artifact.role).collect::<Vec<_>>();
        if actual_roles != expected_roles {
            diagnostics.push(format!(
                "{stage_prefix}.output_artifacts roles do not match `{}` stage contract",
                stage.kind.as_str()
            ));
        }
        for artifact in &stage.output_artifacts {
            if !output_identities.insert((artifact.role, artifact.locator.clone(), artifact.blake3.clone())) {
                diagnostics.push(format!(
                    "{stage_prefix}.output_artifacts duplicates `{}` at `{}`",
                    artifact.role.as_str(),
                    artifact.locator
                ));
            }
        }
        if index == 0 {
            if !stage.input_artifacts.is_empty() {
                diagnostics.push(format!("{stage_prefix}.input_artifacts must be empty for the root stage"));
            }
        } else if let Some(previous_stage) = chain.stages.get(index - 1)
            && stage.input_artifacts != previous_stage.output_artifacts
        {
            diagnostics.push(format!(
                "{stage_prefix}.input_artifacts do not exactly match previous `{}` outputs",
                previous_stage.kind.as_str()
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn expected_output_roles(kind: AeneasStageKind) -> Vec<AeneasStageArtifactRole> {
    match kind {
        AeneasStageKind::RustSource => vec![AeneasStageArtifactRole::RustSource],
        AeneasStageKind::CharonExtraction => vec![
            AeneasStageArtifactRole::CharonOptions,
            AeneasStageArtifactRole::CharonCommand,
            AeneasStageArtifactRole::Llbc,
        ],
        AeneasStageKind::AeneasTranslation => vec![
            AeneasStageArtifactRole::AeneasCommand,
            AeneasStageArtifactRole::TranslationManifest,
        ],
        AeneasStageKind::GeneratedLean => vec![
            AeneasStageArtifactRole::GeneratedLean,
            AeneasStageArtifactRole::ExternalModelInventory,
        ],
        AeneasStageKind::Theorem => vec![
            AeneasStageArtifactRole::TheoremSource,
            AeneasStageArtifactRole::TheoremStatement,
            AeneasStageArtifactRole::TheoremAdequacy,
            AeneasStageArtifactRole::TrustClosure,
        ],
        AeneasStageKind::LeanCheck => vec![
            AeneasStageArtifactRole::LeanToolchain,
            AeneasStageArtifactRole::AeneasLeanLibrary,
            AeneasStageArtifactRole::LeanCommand,
            AeneasStageArtifactRole::CheckerResult,
        ],
    }
}

fn validate_stage_artifact_list(field: &str, artifacts: &[AeneasStageArtifactIdentity], diagnostics: &mut Vec<String>) {
    if artifacts.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    let mut identities = BTreeSet::new();
    for artifact in artifacts {
        validate_stage_artifact(field, artifact, artifact.role, diagnostics);
        if !identities.insert((artifact.role, artifact.locator.clone(), artifact.blake3.clone())) {
            diagnostics.push(format!("{field} contains duplicate `{}`", artifact.role.as_str()));
        }
    }
}

fn validate_stage_artifact(
    field: &str,
    artifact: &AeneasStageArtifactIdentity,
    expected_role: AeneasStageArtifactRole,
    diagnostics: &mut Vec<String>,
) {
    if artifact.role != expected_role {
        diagnostics.push(format!(
            "{field}.role expected `{}` but got `{}`",
            expected_role.as_str(),
            artifact.role.as_str()
        ));
    }
    push_non_empty(diagnostics, &format!("{field}.locator"), &artifact.locator);
    if !is_blake3_hex(&artifact.blake3) {
        diagnostics.push(format!("{field}.blake3 must be a lowercase 64-character BLAKE3 hex digest"));
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH
        && value == value.to_ascii_lowercase()
        && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// Validate the common envelope and its closed provider payload.
#[must_use]
pub fn validate_formal_proof_envelope(envelope: &FormalProofEnvelope) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_exact(&mut diagnostics, "schema_version", FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION, &envelope.schema_version);
    push_non_empty(&mut diagnostics, "provider_id", &envelope.provider_id);
    push_non_empty(&mut diagnostics, "policy_id", &envelope.policy_id);
    push_non_empty(&mut diagnostics, "rail_id", &envelope.rail_id);
    push_non_empty(&mut diagnostics, "checked_scope", &envelope.checked_scope);
    push_non_empty(&mut diagnostics, "claim.theorem_or_property_id", &envelope.claim.theorem_or_property_id);
    push_non_empty(&mut diagnostics, "claim.requirement_id", &envelope.claim.requirement_id);
    push_non_empty(&mut diagnostics, "claim.claim_class", &envelope.claim.claim_class);
    push_non_empty(&mut diagnostics, "package_lock_identity", &envelope.package_lock_identity);
    push_artifact_identities(&mut diagnostics, "source_bindings", &envelope.source_bindings);
    push_artifact_identities(&mut diagnostics, "artifact_identities", &envelope.artifact_identities);
    push_non_empty_strings(&mut diagnostics, "trust_boundaries", &envelope.trust_boundaries);
    push_non_empty_strings(&mut diagnostics, "non_claims", &envelope.non_claims);
    if envelope.provider_id != envelope.provider.provider_id() {
        diagnostics.push(format!(
            "provider_id `{}` does not match closed payload provider `{}`",
            envelope.provider_id,
            envelope.provider.provider_id()
        ));
    }
    if envelope.policy_id == AENEAS_CONSUMER_POLICY_ID && envelope.consumer.is_none() {
        diagnostics.push("Aeneas consumer policy requires its exact consumer binding".to_owned());
    }
    if envelope.consumer.is_some() && envelope.policy_id != AENEAS_CONSUMER_POLICY_ID {
        diagnostics.push("Aeneas consumer binding requires the selected consumer policy".to_owned());
    }
    if envelope.consumer.is_some() && !matches!(envelope.provider, FormalProofProvider::AeneasLean(_)) {
        diagnostics.push("Aeneas consumer binding requires the closed Aeneas provider payload".to_owned());
    }
    if envelope.evidence_tier == FormalProofEvidenceTier::FormalProof && envelope.status != FormalProofStatus::Passed {
        diagnostics.push("formal_proof evidence requires passed provider status".to_owned());
    }
    if let FormalProofProvider::AeneasLean(payload) = &envelope.provider {
        diagnostics.extend(validate_aeneas_lean_payload(payload));
        diagnostics.extend(validate_aeneas_envelope_stage_bindings(envelope, payload));
        if let Some(consumer) = envelope.consumer.as_ref() {
            diagnostics.extend(validate_aeneas_consumer_envelope(envelope, payload, consumer));
        }
        if envelope.evidence_tier == FormalProofEvidenceTier::FormalProof {
            match &payload.theorem_admission {
                Some(admission) => diagnostics.extend(validate_aeneas_formal_proof_promotion(
                    admission,
                    &envelope.claim,
                    envelope.status,
                    &payload.stage_chain,
                    &payload.assumption_policy,
                    &payload.preflight,
                )),
                None => diagnostics.push("formal_proof requires theorem_admission".to_owned()),
            }
        }
        for required in [
            AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS,
            AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS,
            AENEAS_NON_CLAIM_WHOLE_PROGRAM,
        ] {
            if !envelope.non_claims.iter().any(|claim| claim == required) {
                diagnostics.push(format!("non_claims missing required Aeneas boundary `{required}`"));
            }
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.aeneas_consumer_contract.valence_handoff]
fn validate_aeneas_consumer_envelope(
    envelope: &FormalProofEnvelope,
    payload: &AeneasLeanProofPayload,
    consumer: &crate::aeneas_consumer::ConsumerReceiptBinding,
) -> Vec<String> {
    let mut diagnostics = crate::aeneas_consumer::validate_receipt(consumer);
    if envelope.claim.theorem_or_property_id != consumer.theorem.theorem_id
        || envelope.claim.requirement_id != consumer.theorem.requirement_id
        || envelope.claim.claim_class != consumer.theorem.claim_class
    {
        diagnostics.push("formal-proof claim does not match the Aeneas consumer theorem mapping".to_owned());
    }
    let artifact_identities = envelope
        .source_bindings
        .iter()
        .chain(&envelope.artifact_identities)
        .map(|artifact| artifact.blake3.as_str())
        .collect::<Vec<_>>();
    for artifact in &consumer.stage_artifacts {
        let identity = artifact.blake3.strip_prefix("b3:").unwrap_or(&artifact.blake3);
        if !artifact_identities.contains(&identity) {
            diagnostics.push(format!(
                "Aeneas consumer stage artifact `{}` is absent from the verified envelope",
                artifact.role
            ));
        }
        match consumer_provider_stage(&payload.stage_chain, &artifact.role) {
            Some(provider_artifact) if provider_artifact.blake3 == identity => {}
            Some(_) => diagnostics.push(format!(
                "Aeneas consumer stage artifact `{}` does not match the provider stage chain",
                artifact.role
            )),
            None => diagnostics.push(format!(
                "Aeneas consumer stage artifact role `{}` is not supported by the provider chain",
                artifact.role
            )),
        }
    }
    let provider_profile = consumer.provider_profile_identity.strip_prefix("b3:").unwrap_or_default();
    if payload.stage_chain.provider_profile.blake3 != provider_profile {
        diagnostics.push("Aeneas consumer provider profile does not match the provider stage chain".to_owned());
    }
    if consumer.resource_policy.timeout_ms != payload.limits.timeout_ms
        || consumer.resource_policy.stdout_max_bytes != payload.limits.stdout_max_bytes
        || consumer.resource_policy.stderr_max_bytes != payload.limits.stderr_max_bytes
        || consumer.resource_policy.environment_allowlist != payload.environment_allowlist
    {
        diagnostics.push("Aeneas consumer resource policy does not match the provider execution limits".to_owned());
    }
    diagnostics
}

fn consumer_provider_stage<'a>(chain: &'a AeneasStageChain, role: &str) -> Option<&'a AeneasStageArtifactIdentity> {
    let (kind, provider_role) = match role {
        "rust-source" => (AeneasStageKind::RustSource, AeneasStageArtifactRole::RustSource),
        "llbc" => (AeneasStageKind::CharonExtraction, AeneasStageArtifactRole::Llbc),
        "translation-manifest" => (AeneasStageKind::AeneasTranslation, AeneasStageArtifactRole::TranslationManifest),
        "generated-lean" => (AeneasStageKind::GeneratedLean, AeneasStageArtifactRole::GeneratedLean),
        "maintained-theorem" => (AeneasStageKind::Theorem, AeneasStageArtifactRole::TheoremSource),
        _ => return None,
    };
    stage_artifact(chain, kind, provider_role)
}

fn validate_aeneas_envelope_stage_bindings(
    envelope: &FormalProofEnvelope,
    payload: &AeneasLeanProofPayload,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if envelope.package_lock_identity != payload.stage_chain.package_lock.blake3 {
        diagnostics.push("package_lock_identity does not match Aeneas stage_chain.package_lock.blake3".to_owned());
    }
    if let Some(source_artifact) = aeneas_root_source_artifact(&payload.stage_chain)
        && !envelope
            .source_bindings
            .iter()
            .any(|binding| binding.locator == source_artifact.locator && binding.blake3 == source_artifact.blake3)
    {
        diagnostics.push("source_bindings must include the Aeneas stage_chain rust_source artifact".to_owned());
    }
    diagnostics
}

fn aeneas_root_source_artifact(chain: &AeneasStageChain) -> Option<&AeneasStageArtifactIdentity> {
    chain.stages.first().and_then(|stage| {
        stage.output_artifacts.iter().find(|artifact| artifact.role == AeneasStageArtifactRole::RustSource)
    })
}

/// Validate the Aeneas provider payload without process or filesystem access.
#[must_use]
pub fn validate_aeneas_lean_payload(payload: &AeneasLeanProofPayload) -> Vec<String> {
    let mut diagnostics = validate_aeneas_stage_chain(&payload.stage_chain);
    for field in REQUIRED_AENEAS_FIELDS {
        let value = match *field {
            "aeneas_release" => &payload.aeneas_release,
            "charon_revision" => &payload.charon_revision,
            "ocaml_toolchain" => &payload.ocaml_toolchain,
            "lean_toolchain" => &payload.lean_toolchain,
            "aeneas_lean_library" => &payload.aeneas_lean_library,
            "target_triple" => &payload.extraction_policy.target_triple,
            "mir_start_stage" => &payload.extraction_policy.mir_start_stage,
            "drop_policy" => &payload.extraction_policy.drop_policy,
            "start_from" => &payload.extraction_policy.start_from,
            "serialization_format" => &payload.extraction_policy.serialization_format,
            _ => unreachable!("required Aeneas field list contains an unknown field"),
        };
        push_non_empty(&mut diagnostics, field, value);
    }
    push_non_empty_strings(&mut diagnostics, "checked_source_surface", &payload.checked_source_surface);
    diagnostics.extend(validate_aeneas_preflight(&payload.preflight));
    push_non_empty_strings(&mut diagnostics, "command_argv", &payload.command_argv);
    push_non_empty_strings(&mut diagnostics, "environment_allowlist", &payload.environment_allowlist);
    if payload.limits.timeout_ms == 0 {
        diagnostics.push("limits.timeout_ms must be greater than zero".to_owned());
    }
    if payload.limits.stdout_max_bytes == 0 {
        diagnostics.push("limits.stdout_max_bytes must be greater than zero".to_owned());
    }
    if payload.limits.stderr_max_bytes == 0 {
        diagnostics.push("limits.stderr_max_bytes must be greater than zero".to_owned());
    }
    match reviewed_aeneas_provider_profile() {
        Ok(reviewed) => {
            diagnostics.extend(validate_aeneas_provider_pins(payload, &reviewed));
            diagnostics
                .extend(validate_charon_extraction_contract(&payload.extraction_policy, &reviewed.extraction_policy));
        }
        Err(error) => diagnostics.push(error),
    }
    if !payload.assumption_policy.requires_complete_declaration {
        diagnostics.push("assumption_policy.requires_complete_declaration must be true".to_owned());
    }
    push_no_blank_strings(
        &mut diagnostics,
        "assumption_policy.external_models",
        &payload.assumption_policy.external_models,
    );
    push_no_blank_strings(&mut diagnostics, "assumption_policy.axioms", &payload.assumption_policy.axioms);
    push_non_empty(&mut diagnostics, "semantic_reference.revision", &payload.semantic_reference.revision);
    push_non_empty(&mut diagnostics, "semantic_reference.license", &payload.semantic_reference.license);
    push_non_empty(&mut diagnostics, "semantic_reference.source_sentinel", &payload.semantic_reference.source_sentinel);
    if !payload.semantic_reference.recorded_only {
        diagnostics.push("semantic_reference.recorded_only must be true".to_owned());
    }
    if let Some(admission) = &payload.theorem_admission {
        diagnostics.extend(validate_aeneas_theorem_admission(
            admission,
            &payload.stage_chain,
            &payload.assumption_policy,
        ));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Validate theorem-admission shape and its stage/artifact bindings.
// r[impl octet.aeneas_lean_proof_rail.theorem_gate]
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
#[must_use]
pub fn validate_aeneas_theorem_admission(
    admission: &AeneasTheoremAdmission,
    stage_chain: &AeneasStageChain,
    assumption_policy: &AeneasAssumptionPolicy,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_non_empty(&mut diagnostics, "theorem_admission.theorem_id", &admission.theorem_id);
    push_non_empty(&mut diagnostics, "theorem_admission.requirement_id", &admission.requirement_id);
    push_non_empty(&mut diagnostics, "theorem_admission.claim_class", &admission.claim_class);
    if !is_blake3_hex(&admission.approved_statement_blake3) {
        diagnostics.push(
            "theorem_admission.approved_statement_blake3 must be a lowercase 64-character BLAKE3 hex digest".to_owned(),
        );
    }
    push_non_empty(&mut diagnostics, "theorem_admission.lean_check.theorem_id", &admission.lean_check.theorem_id);
    if admission.lean_check.theorem_id != admission.theorem_id {
        diagnostics.push("theorem_admission.lean_check.theorem_id does not match theorem_id".to_owned());
    }
    push_no_blank_strings(
        &mut diagnostics,
        "theorem_admission.adequacy.forbidden_declarations",
        &admission.adequacy.forbidden_declarations,
    );
    validate_trust_closure(&mut diagnostics, &admission.trust_closure);
    if admission.trust_closure.declared_external_models != assumption_policy.external_models {
        diagnostics.push("theorem_admission.trust_closure.external_models do not match assumption_policy".to_owned());
    }
    if admission.trust_closure.declared_axioms != assumption_policy.axioms {
        diagnostics.push("theorem_admission.trust_closure.axioms do not match assumption_policy".to_owned());
    }
    match stage_artifact(stage_chain, AeneasStageKind::Theorem, AeneasStageArtifactRole::TheoremStatement) {
        Some(statement) if statement.blake3 != admission.approved_statement_blake3 => diagnostics.push(
            "theorem_admission.approved_statement_blake3 does not match theorem-stage statement artifact".to_owned(),
        ),
        Some(_) => {}
        None => diagnostics.push("theorem_admission requires a theorem-stage statement artifact".to_owned()),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_trust_closure(diagnostics: &mut Vec<String>, closure: &AeneasTrustClosure) {
    push_no_blank_strings(
        diagnostics,
        "theorem_admission.trust_closure.declared_external_models",
        &closure.declared_external_models,
    );
    push_no_blank_strings(diagnostics, "theorem_admission.trust_closure.declared_axioms", &closure.declared_axioms);
    push_no_blank_strings(
        diagnostics,
        "theorem_admission.trust_closure.declared_opaque_definitions",
        &closure.declared_opaque_definitions,
    );
    push_no_blank_strings(
        diagnostics,
        "theorem_admission.trust_closure.declared_admitted_declarations",
        &closure.declared_admitted_declarations,
    );
    push_no_blank_strings(diagnostics, "theorem_admission.trust_closure.undeclared_inputs", &closure.undeclared_inputs);
}

fn stage_artifact(
    stage_chain: &AeneasStageChain,
    kind: AeneasStageKind,
    role: AeneasStageArtifactRole,
) -> Option<&AeneasStageArtifactIdentity> {
    stage_chain
        .stages
        .iter()
        .find(|stage| stage.kind == kind)
        .and_then(|stage| stage.output_artifacts.iter().find(|artifact| artifact.role == role))
}

/// Validate every condition required to promote a theorem to `formal_proof`.
// r[impl octet.aeneas_lean_proof_rail.theorem_gate]
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
#[must_use]
pub fn validate_aeneas_formal_proof_promotion(
    admission: &AeneasTheoremAdmission,
    claim: &FormalProofClaim,
    status: FormalProofStatus,
    stage_chain: &AeneasStageChain,
    assumption_policy: &AeneasAssumptionPolicy,
    preflight: &AeneasPreflight,
) -> Vec<String> {
    let mut diagnostics = validate_aeneas_theorem_admission(admission, stage_chain, assumption_policy);
    if !admission.named_theorem_present {
        diagnostics.push("formal_proof requires a bound named theorem".to_owned());
    }
    if !admission.statement_approved {
        diagnostics.push("formal_proof requires an approved theorem statement".to_owned());
    }
    if admission.adequacy.decision != AeneasTheoremAdequacyDecision::NonVacuous {
        diagnostics.push("formal_proof requires a non_vacuous theorem adequacy decision".to_owned());
    }
    if !admission.adequacy.forbidden_declarations.is_empty() {
        diagnostics.push("formal_proof rejects forbidden theorem declarations".to_owned());
    }
    if !admission.trust_closure.complete {
        diagnostics.push("formal_proof requires a complete theorem trust closure".to_owned());
    }
    if !admission.trust_closure.undeclared_inputs.is_empty() {
        diagnostics.push("formal_proof rejects undeclared theorem trust inputs".to_owned());
    }
    if !admission.trust_closure.declared_admitted_declarations.is_empty() {
        diagnostics.push("formal_proof rejects admitted theorem declarations".to_owned());
    }
    if admission.lean_check.status != AeneasLeanCheckStatus::Passed {
        diagnostics.push("formal_proof requires a passing Lean checker result".to_owned());
    }
    if status != FormalProofStatus::Passed {
        diagnostics.push("formal_proof requires passed provider status".to_owned());
    }
    if preflight.disposition != AeneasPreflightDisposition::Ready {
        diagnostics
            .push(format!("formal_proof requires ready Aeneas preflight, got `{}`", preflight.disposition.as_str()));
    }
    if claim.theorem_or_property_id != admission.theorem_id {
        diagnostics.push("claim.theorem_or_property_id does not match theorem admission".to_owned());
    }
    if claim.requirement_id != admission.requirement_id {
        diagnostics.push("claim.requirement_id does not match theorem admission".to_owned());
    }
    if claim.claim_class != admission.claim_class {
        diagnostics.push("claim.claim_class does not match theorem admission".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.aeneas_lean_proof_rail.verus_compatibility]
/// Adapt a Verus-v1 receipt without changing or reserializing its receipt schema.
///
/// # Errors
///
/// Returns an error when the existing receipt uses an unknown status spelling.
pub fn adapt_verus_v1_receipt(receipt: VerusProofReceipt) -> Result<FormalProofEnvelope, String> {
    let status = match receipt.status.as_str() {
        PROOF_STATUS_PASSED => FormalProofStatus::Passed,
        PROOF_STATUS_FAILED => FormalProofStatus::Failed,
        status => return Err(format!("unsupported Verus-v1 receipt status `{status}`")),
    };
    let evidence_tier = if status == FormalProofStatus::Passed {
        FormalProofEvidenceTier::FormalProof
    } else {
        FormalProofEvidenceTier::RecordedOnly
    };
    let rail_id = format!("{}#rail-{}", receipt.policy_id, receipt.rail_index);
    let source_bindings = receipt
        .source_bindings
        .iter()
        .map(|binding| FormalProofArtifactIdentity {
            locator: binding.path.clone(),
            blake3: binding.blake3.clone(),
        })
        .collect::<Vec<_>>();
    let artifact_identities = receipt
        .proof_bindings
        .iter()
        .map(|binding| FormalProofArtifactIdentity {
            locator: binding.path.clone(),
            blake3: binding.blake3.clone(),
        })
        .chain([
            FormalProofArtifactIdentity {
                locator: receipt.stdout_artifact.locator.clone(),
                blake3: receipt.stdout_artifact.blake3.clone(),
            },
            FormalProofArtifactIdentity {
                locator: receipt.stderr_artifact.locator.clone(),
                blake3: receipt.stderr_artifact.blake3.clone(),
            },
        ])
        .collect::<Vec<_>>();
    Ok(FormalProofEnvelope {
        schema_version: FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION.to_owned(),
        provider_id: VERUS_V1_PROVIDER_ID.to_owned(),
        policy_id: receipt.policy_id.clone(),
        rail_id: rail_id.clone(),
        checked_scope: receipt.checked_scope.clone(),
        status,
        evidence_tier,
        claim: FormalProofClaim {
            theorem_or_property_id: rail_id,
            requirement_id: receipt.policy_id.clone(),
            claim_class: VERUS_COMPATIBILITY_CLAIM_CLASS.to_owned(),
        },
        package_lock_identity: receipt.package_lock_blake3.clone(),
        source_bindings,
        artifact_identities,
        trust_boundaries: receipt.trust_boundaries.clone(),
        non_claims: receipt.non_claims.clone(),
        consumer: None,
        provider: FormalProofProvider::VerusV1(VerusCompatibilityPayload { receipt }),
    })
}

fn push_exact(diagnostics: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if actual != expected {
        diagnostics.push(format!("{field} expected `{expected}` but got `{actual}`"));
    }
}

fn push_non_empty(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
    }
}

fn push_non_empty_strings(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    if values.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    push_no_blank_strings(diagnostics, field, values);
}

fn push_no_blank_strings(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    for value in values {
        if value.trim().is_empty() {
            diagnostics.push(format!("{field} contains an empty value"));
        }
    }
}

fn push_artifact_identities(diagnostics: &mut Vec<String>, field: &str, identities: &[FormalProofArtifactIdentity]) {
    if identities.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return;
    }
    for identity in identities {
        push_non_empty(diagnostics, &format!("{field}.locator"), &identity.locator);
        push_non_empty(diagnostics, &format!("{field}.blake3"), &identity.blake3);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof::VerusLogArtifact;
    use crate::proof::VerusProofBinding;
    use crate::proof::VerusSolverIdentity;
    use crate::proof::VerusToolchainIdentity;
    use crate::proof::VerusVerifierIdentity;

    const TEST_TIMEOUT_MS: u64 = 30_000;
    const TEST_LOG_MAX_BYTES: usize = 4_096;
    const TEST_DRIFT_POINTER_WIDTH_BITS: u16 = 32;
    const TEST_SOURCE_PATH: &str = "src/verified/addition.rs";
    const TEST_FIRST_START_BYTE: usize = 10;
    const TEST_FIRST_END_BYTE: usize = 20;
    const TEST_SECOND_START_BYTE: usize = 30;
    const TEST_SECOND_END_BYTE: usize = 40;
    const TEST_NON_CLAIMS: &[&str] = &[
        AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS,
        AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS,
        AENEAS_NON_CLAIM_WHOLE_PROGRAM,
    ];

    // r[verify octet.aeneas_lean_proof_rail.provider_contract]
    #[test]
    fn aeneas_payload_and_envelope_accept_complete_typed_contract() {
        let envelope = valid_aeneas_envelope();
        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        let rendered = serde_json::to_string(&envelope).expect("serialize complete envelope");
        let parsed: FormalProofEnvelope = serde_json::from_str(&rendered).expect("parse complete envelope");
        assert_eq!(parsed, envelope);
    }

    // r[verify octet.aeneas_lean_proof_rail.unsupported_surface]
    #[test]
    fn aeneas_preflight_accepts_supported_fixture() {
        let preflight = AeneasPreflight {
            unsupported_inputs: Vec::new(),
            disposition: AeneasPreflightDisposition::Ready,
        };

        assert_eq!(classify_aeneas_preflight(&preflight.unsupported_inputs), AeneasPreflightDisposition::Ready);
        assert!(validate_aeneas_preflight(&preflight).is_empty());
    }

    // r[verify octet.aeneas_lean_proof_rail.unsupported_surface]
    #[test]
    fn aeneas_preflight_rejects_unsupported_inputs_and_promotion() {
        let unsupported_inputs = vec![
            AeneasUnsupportedInput::UnsafeRust,
            AeneasUnsupportedInput::Concurrency,
            AeneasUnsupportedInput::UnsupportedLoopControl,
            AeneasUnsupportedInput::UnresolvedExternal,
        ];
        let blocked = AeneasPreflight {
            unsupported_inputs,
            disposition: AeneasPreflightDisposition::Blocked,
        };
        assert_eq!(classify_aeneas_preflight(&blocked.unsupported_inputs), AeneasPreflightDisposition::Blocked);
        assert!(validate_aeneas_preflight(&blocked).is_empty());

        let mut mismatched = blocked.clone();
        mismatched.disposition = AeneasPreflightDisposition::Ready;
        let mismatch_diagnostics = validate_aeneas_preflight(&mismatched);
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("expected `blocked`")));

        let failed = AeneasPreflight {
            unsupported_inputs: vec![AeneasUnsupportedInput::TranslationFailure],
            disposition: AeneasPreflightDisposition::Failed,
        };
        assert_eq!(classify_aeneas_preflight(&failed.unsupported_inputs), AeneasPreflightDisposition::Failed);
        assert!(validate_aeneas_preflight(&failed).is_empty());

        let mut envelope = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut envelope.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.preflight = blocked;
        let diagnostics = validate_formal_proof_envelope(&envelope);
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("requires ready Aeneas preflight")),
            "unsupported source must not promote to formal proof: {diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn charon_extraction_contract_accepts_reviewed_profile() {
        let reviewed = reviewed_aeneas_provider_profile().expect("generated provider profile").extraction_policy;

        let diagnostics = validate_charon_extraction_contract(&reviewed, &reviewed);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn charon_extraction_contract_rejects_drift_and_weakened_options() {
        let reviewed = reviewed_aeneas_provider_profile().expect("generated provider profile").extraction_policy;
        let mut observed = reviewed.clone();
        observed.has_errors = true;
        observed.skip_borrowck = true;
        observed.target_triple = "aarch64-unknown-linux-gnu".to_owned();
        observed.pointer_width_bits = TEST_DRIFT_POINTER_WIDTH_BITS;
        observed.endianness = "middle".to_owned();
        observed.mir_start_stage = "after-borrowck".to_owned();
        observed.drop_policy = "exclude-drops".to_owned();
        observed.start_from = "item".to_owned();
        observed.include_rules = vec!["src/other".to_owned()];
        observed.exclude_rules = vec!["src/verified".to_owned()];
        observed.opaque_rules = vec!["core::fmt".to_owned()];
        observed.transformation_settings = vec!["erase-spans".to_owned()];
        observed.serialization_format = "llbc-json-v2".to_owned();

        let diagnostics = validate_charon_extraction_contract(&observed, &reviewed);

        for field in [
            "has_errors",
            "skip_borrowck",
            "target_triple",
            "pointer_width_bits",
            "endianness",
            "mir_start_stage",
            "drop_policy",
            "start_from",
            "include_rules",
            "exclude_rules",
            "opaque_rules",
            "transformation_settings",
            "serialization_format",
        ] {
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(field)),
                "missing diagnostic for {field}: {diagnostics:?}"
            );
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.provider_contract]
    #[test]
    fn aeneas_payload_rejects_provider_pin_drift() {
        let mut payload = valid_aeneas_payload();
        payload.charon_revision = "wrong-charon-revision".to_owned();

        let diagnostics = validate_aeneas_lean_payload(&payload);

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("charon_revision drifted")),
            "wrong provider pin must fail closed: {diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.provider_contract]
    #[test]
    fn aeneas_payload_rejects_weak_contract_and_unknown_fields() {
        let mut envelope = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut envelope.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.extraction_policy.skip_borrowck = true;
        payload.semantic_reference.recorded_only = false;
        payload.command_argv.clear();
        envelope.non_claims.retain(|claim| *claim != AENEAS_NON_CLAIM_WHOLE_PROGRAM);

        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("skip_borrowck")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("command_argv")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program")));

        let rendered = serde_json::to_string(&valid_aeneas_payload()).expect("serialize payload");
        let with_unknown_field = rendered.trim_end_matches('}').to_owned() + ",\"unknown\":true}";
        assert!(
            serde_json::from_str::<AeneasLeanProofPayload>(&with_unknown_field).is_err(),
            "closed Aeneas payload must reject unknown fields"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_payload_rejects_broken_stage_link_before_provider_admission() {
        let mut payload = valid_aeneas_payload();
        let theorem_stage = payload
            .stage_chain
            .stages
            .iter_mut()
            .find(|stage| stage.kind == AeneasStageKind::Theorem)
            .expect("fixture theorem stage");
        theorem_stage.input_artifacts.clear();

        let diagnostics = validate_aeneas_lean_payload(&payload);

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("stage_chain.stages[4].input_artifacts")),
            "broken theorem-stage input must be reported: {diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_envelope_rejects_unlinked_package_and_source_stage_identities() {
        let mut envelope = valid_aeneas_envelope();
        envelope.package_lock_identity = "stale-package-lock".to_owned();
        envelope.source_bindings[0].locator = "src/verified/stale.rs".to_owned();

        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("package_lock_identity")),
            "package-lock drift must be reported: {diagnostics:?}"
        );
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("source_bindings")),
            "source-stage drift must be reported: {diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.artifact_evidence]
    #[test]
    fn aeneas_export_tamper_matrix_rejects_digest_role_status_theorem_and_receipt_links() {
        let mut digest = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut digest.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.stage_chain.stages[2].output_artifacts[0].blake3 = "stale-digest".to_owned();

        let mut role = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut role.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.stage_chain.stages[2].output_artifacts[0].role = AeneasStageArtifactRole::RustSource;

        let mut status = valid_aeneas_envelope();
        status.status = FormalProofStatus::Failed;

        let mut theorem = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut theorem.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.theorem_admission.as_mut().expect("fixture theorem admission").lean_check.theorem_id =
            "aeneas.fixture.false_theorem".to_owned();

        let mut receipt_link = valid_aeneas_envelope();
        receipt_link.source_bindings[0].blake3 = "stale-source-link".to_owned();

        for (label, envelope, expected) in [
            ("digest", digest, "BLAKE3"),
            ("role", role, "role"),
            ("status", status, "passed provider status"),
            ("theorem", theorem, "theorem_id"),
            ("receipt link", receipt_link, "source_bindings"),
        ] {
            let diagnostics = validate_formal_proof_envelope(&envelope);
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{label} tamper must be rejected: {diagnostics:?}"
            );
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_stage_chain_accepts_ordered_hash_linked_pipeline() {
        let chain = valid_stage_chain();

        let diagnostics = validate_aeneas_stage_chain(&chain);
        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(aeneas_stage_chain_stage_order(&chain), vec![
            "rust_source",
            "charon_extraction",
            "aeneas_translation",
            "generated_lean",
            "theorem",
            "lean_check",
        ]);
        let rendered = serde_json::to_string(&chain).expect("serialize stage chain");
        let parsed: AeneasStageChain = serde_json::from_str(&rendered).expect("parse stage chain");
        assert_eq!(parsed, chain);
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    #[test]
    fn aeneas_stage_chain_rejects_stale_swapped_duplicate_and_malformed_links() {
        let mut chain = valid_stage_chain();
        chain.stages[2].input_artifacts[0].blake3 = "stale".to_owned();
        chain.stages[1].output_artifacts.swap(1, 2);
        chain.stages[3].output_artifacts[0].blake3 = "stale-generated-lean".to_owned();
        chain.stages[4].input_artifacts.reverse();
        chain.stages[5].output_artifacts[0].blake3 = "not-a-blake3-digest".to_owned();

        let diagnostics = validate_aeneas_stage_chain(&chain);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("previous")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("roles")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate") || diagnostic.contains("match previous"))
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn translation_manifest_accepts_complete_current_run_links() {
        let (manifest, llbc_declarations) = valid_translation_manifest();

        let diagnostics = validate_aeneas_translation_manifest(&manifest, &llbc_declarations);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        let rendered = serde_json::to_string(&manifest).expect("serialize translation manifest");
        let parsed: AeneasTranslationManifest = serde_json::from_str(&rendered).expect("parse translation manifest");
        assert_eq!(parsed, manifest);
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn translation_manifest_rejects_stale_links_bad_inventory_and_durable_def_id() {
        let (mut manifest, mut llbc_declarations) = valid_translation_manifest();
        llbc_declarations.push(llbc_declarations[0].clone());
        manifest.declarations[0].rust_span.end_byte += 1;
        manifest.declarations[1].def_id = "run:missing".to_owned();
        manifest.declarations[1].durable_identity = Some("run:missing".to_owned());
        manifest.declarations[1].lean_file = "generated/Missing.lean".to_owned();
        manifest.generated_files[0].blake3 = "stale".to_owned();

        let diagnostics = validate_aeneas_translation_manifest(&manifest, &llbc_declarations);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("rust_span")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing from the current LLBC run")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("durable_identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("generated_files")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("BLAKE3")));
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicates a current-run declaration")),
            "reused def_id must remain a rejected join key: {diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.verus_compatibility]
    #[test]
    fn verus_adapter_preserves_receipt_and_uses_common_identity() {
        let receipt = valid_verus_receipt();
        let original = receipt.clone();
        let original_text = serde_json::to_string(&original).expect("serialize original Verus receipt");
        let envelope = adapt_verus_v1_receipt(receipt).expect("adapt Verus receipt");

        assert_eq!(envelope.provider_id, VERUS_V1_PROVIDER_ID);
        assert_eq!(envelope.status, FormalProofStatus::Passed);
        assert_eq!(envelope.evidence_tier, FormalProofEvidenceTier::FormalProof);
        assert_eq!(envelope.package_lock_identity, original.package_lock_blake3);
        let FormalProofProvider::VerusV1(payload) = envelope.provider else {
            panic!("adapter must retain Verus provider payload");
        };
        assert_eq!(payload.receipt, original);
        assert_eq!(serde_json::to_string(&payload.receipt).expect("serialize adapted Verus receipt"), original_text);
    }

    // r[verify octet.aeneas_lean_proof_rail.verus_compatibility]
    #[test]
    fn verus_adapter_rejects_unknown_receipt_status_without_reinterpretation() {
        let mut receipt = valid_verus_receipt();
        receipt.status = "proved".to_owned();

        let error = adapt_verus_v1_receipt(receipt).expect_err("unknown status must fail closed");

        assert!(error.contains("unsupported Verus-v1 receipt status"));
    }

    fn valid_aeneas_payload() -> AeneasLeanProofPayload {
        let profile = reviewed_aeneas_provider_profile().expect("generated provider profile");
        AeneasLeanProofPayload {
            aeneas_release: profile.aeneas_release,
            charon_revision: profile.charon_revision,
            ocaml_toolchain: profile.ocaml_toolchain,
            lean_toolchain: profile.lean_toolchain,
            aeneas_lean_library: profile.aeneas_lean_library,
            checked_source_surface: vec!["safe sequential Rust".to_owned()],
            preflight: AeneasPreflight {
                unsupported_inputs: Vec::new(),
                disposition: AeneasPreflightDisposition::Ready,
            },
            command_argv: vec!["aeneas-lean".to_owned(), "--emit-json".to_owned()],
            limits: AeneasExecutionLimits {
                timeout_ms: TEST_TIMEOUT_MS,
                stdout_max_bytes: TEST_LOG_MAX_BYTES,
                stderr_max_bytes: TEST_LOG_MAX_BYTES,
            },
            environment_allowlist: vec!["PATH".to_owned()],
            extraction_policy: profile.extraction_policy,
            assumption_policy: AeneasAssumptionPolicy {
                external_models: vec!["none".to_owned()],
                axioms: vec!["none".to_owned()],
                requires_complete_declaration: true,
            },
            theorem_admission: Some(valid_theorem_admission()),
            stage_chain: valid_stage_chain(),
            semantic_reference: SemanticReference {
                revision: "mechanized-llbc@deadbeef".to_owned(),
                license: "MIT".to_owned(),
                source_sentinel: "README:semantic-foundation".to_owned(),
                recorded_only: true,
            },
        }
    }

    fn valid_theorem_admission() -> AeneasTheoremAdmission {
        let theorem_id = "aeneas.fixture.safe_add".to_owned();
        AeneasTheoremAdmission {
            theorem_id: theorem_id.clone(),
            requirement_id: "octet.aeneas_lean_proof_rail.theorem_adequacy".to_owned(),
            claim_class: AENEAS_NAMED_THEOREM_CLAIM_CLASS.to_owned(),
            source_disposition: AeneasTheoremSourceDisposition::Maintained,
            approved_statement_blake3: crate::proof_artifact::blake3_hex_text("theorems/addition.statement"),
            named_theorem_present: true,
            statement_approved: true,
            adequacy: AeneasTheoremAdequacy {
                decision: AeneasTheoremAdequacyDecision::NonVacuous,
                forbidden_declarations: Vec::new(),
            },
            trust_closure: AeneasTrustClosure {
                complete: true,
                declared_external_models: vec!["none".to_owned()],
                declared_axioms: vec!["none".to_owned()],
                declared_opaque_definitions: Vec::new(),
                declared_admitted_declarations: Vec::new(),
                undeclared_inputs: Vec::new(),
            },
            lean_check: AeneasLeanCheckResult {
                theorem_id,
                status: AeneasLeanCheckStatus::Passed,
            },
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_gate]
    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_theorem_statement_approval_accepts_match_and_rejects_missing_or_mismatched() {
        let approved = validate_aeneas_theorem_statement_approval("approved-digest", "approved-digest")
            .expect("matching theorem approval");
        assert_eq!(approved, "approved-digest");

        let missing =
            validate_aeneas_theorem_statement_approval("", "observed-digest").expect_err("missing theorem approval");
        assert!(missing.contains("must declare"));

        let mismatch = validate_aeneas_theorem_statement_approval("approved-digest", "observed-digest")
            .expect_err("mismatched theorem approval");
        assert!(mismatch.contains("mismatch"));
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_gate]
    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_theorem_source_assessment_accepts_non_vacuous_statement() {
        let adequacy = assess_aeneas_theorem_source("theorem safe_add_matches_model (x : Nat) : x = x := by rfl");

        assert_eq!(adequacy.decision, AeneasTheoremAdequacyDecision::NonVacuous);
        assert!(adequacy.forbidden_declarations.is_empty());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_theorem_source_assessment_rejects_vacuous_and_forbidden_statements() {
        for source in [
            "theorem vacuous : True := by trivial",
            "theorem parenthesized : ( True ) := by trivial",
            "theorem conjunction : True ∧ True := by constructor <;> trivial",
        ] {
            let vacuous = assess_aeneas_theorem_source(source);
            assert_eq!(vacuous.decision, AeneasTheoremAdequacyDecision::Vacuous, "source: {source}");
        }

        let forbidden = assess_aeneas_theorem_source("theorem admitted : Nat := by sorry");
        assert_eq!(forbidden.decision, AeneasTheoremAdequacyDecision::NonVacuous);
        assert_eq!(forbidden.forbidden_declarations, vec!["sorry".to_owned()]);
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_formal_proof_rejects_false_and_vacuous_theorems() {
        let mut false_theorem = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut false_theorem.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.theorem_admission.as_mut().expect("fixture theorem admission").statement_approved = false;
        let false_diagnostics = validate_formal_proof_envelope(&false_theorem);
        assert!(
            false_diagnostics.iter().any(|diagnostic| diagnostic.contains("approved theorem statement")),
            "false theorem must fail statement approval: {false_diagnostics:?}"
        );

        let mut vacuous_theorem = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut vacuous_theorem.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.theorem_admission.as_mut().expect("fixture theorem admission").adequacy.decision =
            AeneasTheoremAdequacyDecision::Vacuous;
        let vacuous_diagnostics = validate_formal_proof_envelope(&vacuous_theorem);
        assert!(
            vacuous_diagnostics.iter().any(|diagnostic| diagnostic.contains("non_vacuous")),
            "vacuous theorem must fail adequacy: {vacuous_diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.stage_chain]
    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    // r[verify octet.aeneas_lean_proof_rail.theorem_gate]
    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_formal_proof_rejects_missing_theorem_trust_gaps_and_role_promotion() {
        let mut envelope = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut envelope.provider else {
            panic!("fixture must use Aeneas provider");
        };
        let admission = payload.theorem_admission.as_mut().expect("fixture theorem admission");
        admission.named_theorem_present = false;
        admission.adequacy.forbidden_declarations = vec!["sorry".to_owned(), "admit".to_owned()];
        admission.trust_closure.undeclared_inputs = vec!["model:missing".to_owned(), "axiom:missing".to_owned()];
        admission.trust_closure.declared_admitted_declarations = vec!["theorem:admitted".to_owned()];

        let diagnostics = validate_formal_proof_envelope(&envelope);

        for expected in [
            "requires a bound named theorem",
            "rejects forbidden theorem declarations",
            "rejects undeclared theorem trust inputs",
            "rejects admitted theorem declarations",
        ] {
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "missing promotion diagnostic `{expected}`: {diagnostics:?}"
            );
        }

        let mut promoted_recorded_only = valid_aeneas_envelope();
        let FormalProofProvider::AeneasLean(payload) = &mut promoted_recorded_only.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.theorem_admission = None;
        let promotion_diagnostics = validate_formal_proof_envelope(&promoted_recorded_only);
        assert!(
            promotion_diagnostics.iter().any(|diagnostic| diagnostic.contains("requires theorem_admission")),
            "recorded-only payload must not be promoted without a theorem: {promotion_diagnostics:?}"
        );
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_gate]
    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_translation_only_output_remains_recorded_only() {
        let mut envelope = valid_aeneas_envelope();
        envelope.status = FormalProofStatus::Failed;
        envelope.evidence_tier = FormalProofEvidenceTier::RecordedOnly;
        let FormalProofProvider::AeneasLean(payload) = &mut envelope.provider else {
            panic!("fixture must use Aeneas provider");
        };
        payload.theorem_admission = None;

        let diagnostics = validate_formal_proof_envelope(&envelope);

        assert!(
            diagnostics.is_empty(),
            "translation-only output must remain valid recorded evidence: {diagnostics:?}"
        );
    }

    fn valid_translation_manifest() -> (AeneasTranslationManifest, Vec<AeneasLlbcDeclaration>) {
        let first_span = AeneasSourceSpan {
            locator: TEST_SOURCE_PATH.to_owned(),
            start_byte: TEST_FIRST_START_BYTE,
            end_byte: TEST_FIRST_END_BYTE,
        };
        let second_span = AeneasSourceSpan {
            locator: TEST_SOURCE_PATH.to_owned(),
            start_byte: TEST_SECOND_START_BYTE,
            end_byte: TEST_SECOND_END_BYTE,
        };
        let llbc_declarations = vec![
            AeneasLlbcDeclaration {
                def_id: "run:0".to_owned(),
                rust_name: "safe_add".to_owned(),
                rust_span: first_span.clone(),
            },
            AeneasLlbcDeclaration {
                def_id: "run:1".to_owned(),
                rust_name: "safe_add_helper".to_owned(),
                rust_span: second_span.clone(),
            },
        ];
        let manifest = AeneasTranslationManifest {
            schema_version: AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION.to_owned(),
            declarations: vec![
                AeneasTranslationDeclaration {
                    def_id: "run:0".to_owned(),
                    rust_name: "safe_add".to_owned(),
                    rust_span: first_span,
                    lean_name: "safeAdd".to_owned(),
                    lean_file: "generated/Add.lean".to_owned(),
                    is_opaque: false,
                    can_fail: false,
                    can_diverge: false,
                    is_recursive: false,
                    loop_parent_def_id: None,
                    durable_identity: None,
                },
                AeneasTranslationDeclaration {
                    def_id: "run:1".to_owned(),
                    rust_name: "safe_add_helper".to_owned(),
                    rust_span: second_span,
                    lean_name: "safeAddHelper".to_owned(),
                    lean_file: "generated/Helper.lean".to_owned(),
                    is_opaque: false,
                    can_fail: true,
                    can_diverge: false,
                    is_recursive: true,
                    loop_parent_def_id: Some("run:0".to_owned()),
                    durable_identity: None,
                },
            ],
            generated_files: vec![
                AeneasGeneratedLeanFile {
                    locator: "generated/Add.lean".to_owned(),
                    blake3: crate::proof_artifact::blake3_hex_text("generated/Add.lean"),
                },
                AeneasGeneratedLeanFile {
                    locator: "generated/Helper.lean".to_owned(),
                    blake3: crate::proof_artifact::blake3_hex_text("generated/Helper.lean"),
                },
            ],
        };
        (manifest, llbc_declarations)
    }

    fn valid_stage_chain() -> AeneasStageChain {
        let source = stage_artifact(AeneasStageArtifactRole::RustSource, "src/verified/addition.rs");
        let charon_options = stage_artifact(AeneasStageArtifactRole::CharonOptions, "stages/charon-options.json");
        let charon_command = stage_artifact(AeneasStageArtifactRole::CharonCommand, "stages/charon.argv");
        let llbc = stage_artifact(AeneasStageArtifactRole::Llbc, "stages/addition.llbc");
        let aeneas_command = stage_artifact(AeneasStageArtifactRole::AeneasCommand, "stages/aeneas.argv");
        let translation = stage_artifact(AeneasStageArtifactRole::TranslationManifest, "stages/translation.json");
        let generated_lean = stage_artifact(AeneasStageArtifactRole::GeneratedLean, "generated/Add.lean");
        let external_models =
            stage_artifact(AeneasStageArtifactRole::ExternalModelInventory, "generated/external-models.json");
        let theorem_source = stage_artifact(AeneasStageArtifactRole::TheoremSource, "theorems/addition.lean");
        let theorem_statement =
            stage_artifact(AeneasStageArtifactRole::TheoremStatement, "theorems/addition.statement");
        let theorem_adequacy =
            stage_artifact(AeneasStageArtifactRole::TheoremAdequacy, "theorems/addition.adequacy.json");
        let trust_closure = stage_artifact(AeneasStageArtifactRole::TrustClosure, "theorems/addition.trust.json");
        let lean_toolchain = stage_artifact(AeneasStageArtifactRole::LeanToolchain, "lean-toolchain");
        let aeneas_library = stage_artifact(AeneasStageArtifactRole::AeneasLeanLibrary, "lake-manifest.json");
        let lean_command = stage_artifact(AeneasStageArtifactRole::LeanCommand, "stages/lean.argv");
        let checker_result = stage_artifact(AeneasStageArtifactRole::CheckerResult, "stages/lean-check.json");
        AeneasStageChain {
            schema_version: AENEAS_STAGE_CHAIN_SCHEMA_VERSION.to_owned(),
            provider_profile: stage_artifact(AeneasStageArtifactRole::ProviderProfile, "profile.json"),
            package_lock: stage_artifact(AeneasStageArtifactRole::PackageLock, "package-lock.json"),
            stages: vec![
                AeneasStage {
                    kind: AeneasStageKind::RustSource,
                    input_artifacts: Vec::new(),
                    output_artifacts: vec![source],
                },
                AeneasStage {
                    kind: AeneasStageKind::CharonExtraction,
                    input_artifacts: vec![stage_artifact(
                        AeneasStageArtifactRole::RustSource,
                        "src/verified/addition.rs",
                    )],
                    output_artifacts: vec![charon_options, charon_command, llbc],
                },
                AeneasStage {
                    kind: AeneasStageKind::AeneasTranslation,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::CharonOptions, "stages/charon-options.json"),
                        stage_artifact(AeneasStageArtifactRole::CharonCommand, "stages/charon.argv"),
                        stage_artifact(AeneasStageArtifactRole::Llbc, "stages/addition.llbc"),
                    ],
                    output_artifacts: vec![aeneas_command, translation],
                },
                AeneasStage {
                    kind: AeneasStageKind::GeneratedLean,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::AeneasCommand, "stages/aeneas.argv"),
                        stage_artifact(AeneasStageArtifactRole::TranslationManifest, "stages/translation.json"),
                    ],
                    output_artifacts: vec![generated_lean, external_models],
                },
                AeneasStage {
                    kind: AeneasStageKind::Theorem,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::GeneratedLean, "generated/Add.lean"),
                        stage_artifact(
                            AeneasStageArtifactRole::ExternalModelInventory,
                            "generated/external-models.json",
                        ),
                    ],
                    output_artifacts: vec![theorem_source, theorem_statement, theorem_adequacy, trust_closure],
                },
                AeneasStage {
                    kind: AeneasStageKind::LeanCheck,
                    input_artifacts: vec![
                        stage_artifact(AeneasStageArtifactRole::TheoremSource, "theorems/addition.lean"),
                        stage_artifact(AeneasStageArtifactRole::TheoremStatement, "theorems/addition.statement"),
                        stage_artifact(AeneasStageArtifactRole::TheoremAdequacy, "theorems/addition.adequacy.json"),
                        stage_artifact(AeneasStageArtifactRole::TrustClosure, "theorems/addition.trust.json"),
                    ],
                    output_artifacts: vec![lean_toolchain, aeneas_library, lean_command, checker_result],
                },
            ],
        }
    }

    fn stage_artifact(role: AeneasStageArtifactRole, locator: &str) -> AeneasStageArtifactIdentity {
        AeneasStageArtifactIdentity {
            role,
            locator: locator.to_owned(),
            blake3: crate::proof_artifact::blake3_hex_text(locator),
        }
    }

    fn valid_aeneas_envelope() -> FormalProofEnvelope {
        let provider = valid_aeneas_payload();
        let source = provider
            .stage_chain
            .stages
            .first()
            .expect("fixture source stage")
            .output_artifacts
            .first()
            .expect("fixture source artifact");
        let package_lock_identity = provider.stage_chain.package_lock.blake3.clone();
        FormalProofEnvelope {
            schema_version: FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION.to_owned(),
            provider_id: AENEAS_LEAN_PROVIDER_ID.to_owned(),
            policy_id: "octet.aeneas_lean_proof_rail.fixture".to_owned(),
            rail_id: "octet.aeneas_lean_proof_rail.fixture#rail-0".to_owned(),
            checked_scope: "formal-proof:fixture-only:aeneas#rail-0".to_owned(),
            status: FormalProofStatus::Passed,
            evidence_tier: FormalProofEvidenceTier::FormalProof,
            claim: FormalProofClaim {
                theorem_or_property_id: "aeneas.fixture.safe_add".to_owned(),
                requirement_id: "octet.aeneas_lean_proof_rail.theorem_adequacy".to_owned(),
                claim_class: "named-theorem-over-generated-model".to_owned(),
            },
            package_lock_identity,
            source_bindings: vec![FormalProofArtifactIdentity {
                locator: source.locator.clone(),
                blake3: source.blake3.clone(),
            }],
            artifact_identities: vec![artifact("proofs/aeneas.translation.json")],
            trust_boundaries: vec!["fixture-only provider contract".to_owned()],
            non_claims: TEST_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
            consumer: None,
            provider: FormalProofProvider::AeneasLean(provider),
        }
    }

    fn valid_verus_receipt() -> VerusProofReceipt {
        VerusProofReceipt {
            schema_version: "octet-verus-proof-receipt/v1".to_owned(),
            policy_id: "octet.verus.fixture".to_owned(),
            rail_index: 0,
            backend: "verus".to_owned(),
            verifier_mode: "fixture".to_owned(),
            producer_id: "verus@fixture".to_owned(),
            checker_id: "octet-standards".to_owned(),
            package_lock_blake3: "b3:package-lock".to_owned(),
            verifier_identity: VerusVerifierIdentity {
                verifier_id: "verus@fixture".to_owned(),
                verifier_digest: "b3:verifier".to_owned(),
                verifier_binary_path: "verus".to_owned(),
                verifier_binary_digest: "b3:binary".to_owned(),
                ..VerusVerifierIdentity::default()
            },
            toolchain_identity: VerusToolchainIdentity {
                toolchain_id: "rust@fixture".to_owned(),
                toolchain_digest: "b3:toolchain".to_owned(),
            },
            solver_identity: VerusSolverIdentity::default(),
            command_blake3: "b3:command".to_owned(),
            source_bindings: vec![VerusProofBinding {
                path: TEST_SOURCE_PATH.to_owned(),
                blake3: "b3:source".to_owned(),
            }],
            proof_bindings: vec![VerusProofBinding {
                path: "verus/proof.rs".to_owned(),
                blake3: "b3:proof".to_owned(),
            }],
            trust_boundaries: vec!["fixture-only".to_owned()],
            checked_scope: "formal-proof:fixture-only:verus#rail-0".to_owned(),
            status: PROOF_STATUS_PASSED.to_owned(),
            pass_signal: "proof-result: pass".to_owned(),
            proof_artifact_identity: "proofs/verus.receipt".to_owned(),
            stdout_artifact: VerusLogArtifact {
                locator: "proofs/verus.stdout".to_owned(),
                blake3: "b3:stdout".to_owned(),
                bytes: 1,
                truncated: false,
            },
            stderr_artifact: VerusLogArtifact {
                locator: "proofs/verus.stderr".to_owned(),
                blake3: "b3:stderr".to_owned(),
                bytes: 0,
                truncated: false,
            },
            non_claims: vec!["not whole-program correctness".to_owned()],
        }
    }

    fn artifact(locator: &str) -> FormalProofArtifactIdentity {
        FormalProofArtifactIdentity {
            locator: locator.to_owned(),
            blake3: "b3:artifact".to_owned(),
        }
    }
}
