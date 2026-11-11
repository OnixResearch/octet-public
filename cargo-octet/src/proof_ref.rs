use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::ProofRefArgs;
use crate::cli::ProofRefMode;
use crate::cli::ProofRefScaffoldArgs;
use crate::cli::ProofRefValidateArgs;
use crate::proof_goal_state::positive_phrase_present;

const PROOF_EVIDENCE_REF_SCHEMA_VERSION: &str = "octet-proof-evidence-ref/v1";
const PROOF_CARDS_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const EXIT_VALID: i32 = 0;
const EXIT_INVALID: i32 = 1;
const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_REFERENCE_ID: &str = "reference_id";
const FIELD_CONSUMER_CONTEXT: &str = "consumer_context";
const FIELD_CONSUMER_SOURCE_PATH: &str = "consumer_source_path";
const FIELD_CONSUMER_SOURCE_BLAKE3: &str = "consumer_source_blake3";
const FIELD_CONSUMER_SOURCE_ANCHOR: &str = "consumer_source_anchor";
const FIELD_PROOF_OR_PROPERTY_ID: &str = "proof_or_property_id";
const FIELD_SOURCE_REQUIREMENT_OR_POLICY_ID: &str = "source_requirement_or_policy_id";
const FIELD_CHECKED_RUST_SOURCE_SCOPE: &str = "checked_rust_source_scope";
const FIELD_EXPECTED_EVIDENCE_TIER: &str = "expected_evidence_tier";
const FIELD_REQUIRED_STATUS: &str = "required_status";
const FIELD_REQUIRED_ARTIFACTS: &str = "required_artifacts";
const FIELD_NON_CLAIMS: &str = "non_claims";
const EVIDENCE_TIER_FORMAL_PROOF: &str = "formal_proof";
const EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
/// Negative-control record written before verifier attribution was required. It remains
/// deserializable, but its annotated `property_demonstrated` status cannot be verified.
pub(crate) const NEGATIVE_CHECK_SCHEMA_VERSION_V1: &str = "octet-proof-negative-check/v1";
/// Negative-control record that retains typed verifier evidence, measured source and deck
/// identities, the declared mutation identity, and the two-card binding surface.
pub(crate) const NEGATIVE_CHECK_SCHEMA_VERSION_V2: &str = "octet-proof-negative-check/v2";
pub(crate) const NEGATIVE_CHECK_EVIDENCE_SCHEMA_VERSION: &str = "octet-proof-negative-check-evidence/v1";
pub(crate) const NEGATIVE_STATUS_DEMONSTRATED: &str = "property_demonstrated";
pub(crate) const NEGATIVE_STATUS_NOT_DEMONSTRATED: &str = "property_not_demonstrated";
pub(crate) const NEGATIVE_STATUS_INCONCLUSIVE: &str = "inconclusive";
pub(crate) const NEGATIVE_RUN_ACCEPTED: &str = "accepted";
pub(crate) const NEGATIVE_RUN_REJECTED: &str = "rejected";
pub(crate) const NEGATIVE_RUN_COMPILE_ERROR: &str = "compile_error";
pub(crate) const NEGATIVE_RUN_CRASH: &str = "crash";
pub(crate) const NEGATIVE_RUN_TIMEOUT: &str = "timeout";
pub(crate) const NEGATIVE_RUN_NO_EVIDENCE: &str = "no_evidence";
const NON_CLAIM_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_CERTIFICATION: &str = "not certification";
const NON_CLAIM_BUSINESS_DOMAIN: &str = "not business-domain validation";
const NON_CLAIM_SEMANTIC_EQUIVALENCE: &str = "not semantic equivalence";
const NON_CLAIM_FULL_SAFETY: &str = "not full safety";
/// Claim subjects matched per field after lowercasing and replacing hyphens with
/// spaces, so any verb (`proves`, `guarantees`, `certifies`) and hyphenated
/// spellings are caught while negated non-claims such as `not verified runtime`
/// are not.
const OVERCLAIM_PHRASES: &[(&str, &str)] = &[
    ("whole program correctness", "whole-program correctness"),
    ("certification", "certification"),
    ("certifies", "certification"),
    ("business domain validation", "business-domain validation"),
    ("semantic equivalence", "semantic equivalence"),
    ("full safety", "full safety"),
    ("verified runtime", "verified runtime"),
];

#[derive(Debug, Deserialize, Serialize)]
pub struct ProofEvidenceRef {
    pub(crate) schema_version: String,
    pub(crate) reference_id: String,
    pub(crate) consumer_context: String,
    #[serde(default)]
    pub(crate) consumer_source_path: String,
    #[serde(default)]
    pub(crate) consumer_source_blake3: String,
    #[serde(default)]
    pub(crate) consumer_source_anchor: String,
    pub(crate) proof_or_property_id: String,
    pub(crate) source_requirement_or_policy_id: String,
    pub(crate) checked_rust_source_scope: String,
    pub(crate) expected_evidence_tier: String,
    pub(crate) required_status: String,
    pub(crate) required_artifacts: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ProofCardDeck {
    pub(crate) schema_version: String,
    pub(crate) cards: Vec<ProofCard>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProofCardArtifactIdentity {
    pub(crate) artifact: String,
    pub(crate) blake3: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProofCard {
    pub(crate) proof_or_property_id: String,
    pub(crate) source_requirement_or_policy_id: String,
    #[serde(default)]
    pub(crate) policy_id: String,
    pub(crate) checked_rust_source_scope: String,
    #[serde(default)]
    pub(crate) source_paths: Vec<String>,
    pub(crate) evidence_tier: String,
    pub(crate) evidence_origin: String,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) verifier_mode: String,
    #[serde(default)]
    pub(crate) verifier_identity_summary: String,
    #[serde(default)]
    pub(crate) toolchain_identity_summary: String,
    pub(crate) artifact_blake3_identities: Vec<ProofCardArtifactIdentity>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) negative_check: Option<ProofCardNegativeCheck>,
}

/// Bounded negative-check record attached to one proof card.
///
/// A mutant rail run is evidence that the rail discriminated the mutation, not evidence that the
/// theorem matches the intended property in general. `status` is `property_demonstrated` only when
/// the retained verifier evidence attributes a rejection to the expected obligation; a
/// still-passing mutant yields `property_not_demonstrated`.
///
/// v1 records carry only an annotated `status` and the mutant card fields, so validation cannot
/// establish verifier attribution from them. v2 records retain `evidence` and `measured` so
/// validation recomputes the decision instead of trusting `status`.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ProofCardNegativeCheck {
    pub(crate) schema_version: String,
    pub(crate) status: String,
    pub(crate) mutant_source: String,
    pub(crate) mutant_card_status: String,
    pub(crate) mutant_card_evidence_tier: String,
    pub(crate) mutant_cards_blake3: String,
    pub(crate) non_claims: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) evidence: Option<NegativeControlEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) measured: Option<NegativeControlMeasurement>,
}

/// Explicit verifier evidence supplied to the negative-control recorder.
///
/// The document binds both runs to one approved specification, obligation, checked scope,
/// provider, toolchain, and verification configuration. Each run supplies its own outcome,
/// rejection attribution, source and deck identities, and optional raw-output identity.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(default)]
pub(crate) struct NegativeControlEvidence {
    pub(crate) schema_version: String,
    pub(crate) verifier: String,
    pub(crate) toolchain: String,
    pub(crate) verification_mode: String,
    pub(crate) command_argv: Vec<String>,
    pub(crate) verification_configuration_blake3: String,
    pub(crate) approved_specification_id: String,
    pub(crate) proof_or_property_id: String,
    pub(crate) source_requirement_or_policy_id: String,
    pub(crate) checked_rust_source_scope: String,
    pub(crate) evidence_tier: String,
    pub(crate) mutation: NegativeControlMutation,
    pub(crate) baseline: NegativeControlVerifierRun,
    pub(crate) mutant: NegativeControlVerifierRun,
    pub(crate) non_claims: Vec<String>,
}

/// The one source mutation the control is allowed to differ by.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(default)]
pub(crate) struct NegativeControlMutation {
    pub(crate) mutation_id: String,
    pub(crate) patch_blake3: String,
    pub(crate) baseline_source_blake3: String,
    pub(crate) mutant_source_blake3: String,
}

/// One verifier run, bound to the shared evidence identity and its measured artifacts.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(default)]
pub(crate) struct NegativeControlVerifierRun {
    pub(crate) outcome: String,
    pub(crate) rejection_attribution: Option<String>,
    pub(crate) approved_specification_id: String,
    pub(crate) proof_or_property_id: String,
    pub(crate) source_requirement_or_policy_id: String,
    pub(crate) checked_rust_source_scope: String,
    pub(crate) source_path: String,
    pub(crate) source_blake3: String,
    pub(crate) deck_path: String,
    pub(crate) deck_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) raw_output: Option<NegativeControlRawOutput>,
}

/// Optional retained verifier output. When declared it must be readable, non-empty, and match.
#[derive(Clone, Debug, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(default)]
pub(crate) struct NegativeControlRawOutput {
    pub(crate) artifact: String,
    pub(crate) blake3: String,
}

/// Identities the recorder measured from the actual files, plus the two-card binding surface.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub(crate) struct NegativeControlMeasurement {
    pub(crate) baseline_source_blake3: String,
    pub(crate) mutant_source_blake3: String,
    pub(crate) mutation_id: String,
    pub(crate) patch_blake3: String,
    pub(crate) baseline_deck_blake3: String,
    pub(crate) mutant_deck_blake3: String,
    pub(crate) binding: NegativeControlCardBinding,
}

/// The proof-card fields both runs must agree on. Only the declared source mutation may differ.
#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub(crate) struct NegativeControlCardBinding {
    pub(crate) baseline_proof_or_property_id: String,
    pub(crate) mutant_proof_or_property_id: String,
    pub(crate) baseline_source_requirement_or_policy_id: String,
    pub(crate) mutant_source_requirement_or_policy_id: String,
    pub(crate) baseline_policy_id: String,
    pub(crate) mutant_policy_id: String,
    pub(crate) baseline_checked_rust_source_scope: String,
    pub(crate) mutant_checked_rust_source_scope: String,
    pub(crate) baseline_evidence_tier: String,
    pub(crate) mutant_evidence_tier: String,
    pub(crate) baseline_verifier_mode: String,
    pub(crate) mutant_verifier_mode: String,
    pub(crate) baseline_verifier_identity_summary: String,
    pub(crate) mutant_verifier_identity_summary: String,
    pub(crate) baseline_toolchain_identity_summary: String,
    pub(crate) mutant_toolchain_identity_summary: String,
    pub(crate) baseline_source_paths: Vec<String>,
    pub(crate) mutant_source_paths: Vec<String>,
}

/// Card fields the retained record is validated against.
pub(crate) struct NegativeControlExpectation<'a> {
    pub(crate) proof_or_property_id: &'a str,
    pub(crate) source_requirement_or_policy_id: &'a str,
    pub(crate) policy_id: &'a str,
    pub(crate) checked_rust_source_scope: &'a str,
    pub(crate) evidence_tier: &'a str,
    pub(crate) baseline_card_status: &'a str,
    pub(crate) baseline_card_diagnostics: &'a [String],
    pub(crate) verifier_mode: &'a str,
    pub(crate) verifier_identity_summary: &'a str,
    pub(crate) toolchain_identity_summary: &'a str,
    pub(crate) baseline_source_paths: &'a [String],
    /// The recorder hashes the files it was given; validation only re-hashes readable paths.
    pub(crate) require_readable_artifacts: bool,
}

pub(crate) struct NegativeControlVerdict {
    pub(crate) status: &'static str,
    pub(crate) problems: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProofRefReport {
    schema_version: &'static str,
    valid: bool,
    reference_id: String,
    proof_or_property_id: String,
    matched_card: bool,
    consumer_source_matched: bool,
    diagnostics: Vec<String>,
}

#[derive(Debug)]
struct ProofRefValidation {
    diagnostics: Vec<String>,
    /// True only when the consumer-source linkage check ran and passed.
    consumer_source_matched: bool,
}

#[derive(Debug)]
struct ConsumerSourceArtifact {
    path: String,
    blake3: String,
    text: String,
}

pub fn run(args: &ProofRefArgs) -> Result<i32, String> {
    match &args.mode {
        ProofRefMode::Validate(validate_args) => run_validate(validate_args),
        ProofRefMode::Scaffold(scaffold_args) => run_scaffold(scaffold_args),
    }
}

fn run_validate(args: &ProofRefValidateArgs) -> Result<i32, String> {
    let reference = read_json::<ProofEvidenceRef>(&args.reference)?;
    let deck = read_json::<ProofCardDeck>(&args.proof_cards)?;
    let consumer_source = read_consumer_source_artifact(&reference);
    let consumer_source_ref = match &consumer_source {
        Ok(Some(source)) => Some(source),
        Ok(None) | Err(_) => None,
    };
    let validation = validate_proof_evidence_ref(&reference, &deck, consumer_source_ref);
    let mut diagnostics = validation.diagnostics;
    if let Err(diagnostic) = &consumer_source {
        diagnostics.push(diagnostic.clone());
        diagnostics.sort();
        diagnostics.dedup();
    }
    let report = ProofRefReport {
        schema_version: PROOF_EVIDENCE_REF_SCHEMA_VERSION,
        valid: diagnostics.is_empty(),
        reference_id: reference.reference_id.clone(),
        proof_or_property_id: reference.proof_or_property_id.clone(),
        matched_card: matching_card(&reference, &deck).is_some(),
        consumer_source_matched: validation.consumer_source_matched,
        diagnostics,
    };
    print_report(args.output_format, &report)?;
    Ok(if report.valid { EXIT_VALID } else { EXIT_INVALID })
}

fn run_scaffold(args: &ProofRefScaffoldArgs) -> Result<i32, String> {
    let deck = read_json::<ProofCardDeck>(&args.proof_cards)?;
    let source = read_scaffold_source_artifact(&args.consumer_source)?;
    let reference = scaffold_proof_evidence_ref(
        &deck,
        &args.proof_or_property_id,
        &args.reference_id,
        &args.consumer_context,
        &source,
        &args.anchor,
    )
    .map_err(|diagnostics| format!("proof-ref scaffold failed: {}", diagnostics.join("; ")))?;
    let text =
        serde_json::to_string_pretty(&reference).map_err(|error| format!("serialize proof evidence ref: {error}"))?;
    let text = format!("{text}\n");
    if let Some(output) = &args.output {
        refuse_output_aliasing_inputs(output, &[
            ("--proof-cards", &args.proof_cards),
            ("--consumer-source", &args.consumer_source),
        ])?;
        fs::write(output, text).map_err(|error| format!("write {}: {error}", output.display()))?;
    } else {
        print!("{text}");
    }
    Ok(EXIT_VALID)
}

/// Refuses an `--output` path that resolves to one of the scaffold inputs, so scaffold never
/// overwrites the proof-card deck or consumer source it read. A not-yet-existing output is
/// resolved through its canonical parent directory.
fn refuse_output_aliasing_inputs(output: &Path, inputs: &[(&str, &PathBuf)]) -> Result<(), String> {
    let output_identity = canonical_output_path(output)?;
    for (flag, input) in inputs {
        let input_identity =
            fs::canonicalize(input).map_err(|error| format!("resolve {flag} {}: {error}", input.display()))?;
        if output_identity == input_identity {
            return Err(format!(
                "proof-ref scaffold refuses --output {} because it resolves to {flag} {}",
                output.display(),
                input.display()
            ));
        }
    }
    Ok(())
}

fn canonical_output_path(output: &Path) -> Result<PathBuf, String> {
    match fs::canonicalize(output) {
        Ok(path) => Ok(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let file_name = output
                .file_name()
                .ok_or_else(|| format!("resolve --output {}: missing file name", output.display()))?;
            let parent = output.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
            let parent = fs::canonicalize(parent)
                .map_err(|error| format!("resolve --output parent {}: {error}", parent.display()))?;
            Ok(parent.join(file_name))
        }
        Err(error) => Err(format!("resolve --output {}: {error}", output.display())),
    }
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {} as JSON: {error}", path.display()))
}

/// Reads a saved `ProofEvidenceRef` for the proof-candidate lister.
pub(crate) fn read_reference_file(path: &Path) -> Result<ProofEvidenceRef, String> {
    read_json(path)
}

/// Reads a saved `proof-cards.json` deck for the proof-candidate lister.
pub(crate) fn read_deck_file(path: &Path) -> Result<ProofCardDeck, String> {
    read_json(path)
}

/// Reads the explicit typed verifier evidence supplied to the negative-control recorder.
pub(crate) fn read_negative_check_evidence(path: &Path) -> Result<NegativeControlEvidence, String> {
    read_json(path)
}

/// Validates a saved reference against a deck and the already-loaded candidate source text.
///
/// The candidate lister resolves source paths relative to the scanned workspace rather than the
/// process working directory, so it hands the exact bytes and display path to this helper.
pub(crate) fn validate_reference_against_source(
    reference: &ProofEvidenceRef,
    deck: &ProofCardDeck,
    source_path: &str,
    source_text: &str,
) -> Vec<String> {
    let source = ConsumerSourceArtifact {
        path: source_path.to_owned(),
        blake3: blake3::hash(source_text.as_bytes()).to_hex().to_string(),
        text: source_text.to_owned(),
    };
    validate_proof_evidence_ref(reference, deck, Some(&source)).diagnostics
}

fn read_scaffold_source_artifact(path: &Path) -> Result<ConsumerSourceArtifact, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read consumer source {}: {error}", path.display()))?;
    Ok(ConsumerSourceArtifact {
        path: normalized_display(path),
        blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        text,
    })
}

fn normalized_display(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn read_consumer_source_artifact(reference: &ProofEvidenceRef) -> Result<Option<ConsumerSourceArtifact>, String> {
    if reference.consumer_source_path.trim().is_empty() {
        return Ok(None);
    }
    let path = PathBuf::from(&reference.consumer_source_path);
    let text = fs::read_to_string(&path).map_err(|error| {
        format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_PATH}` cannot read `{}`: {error}",
            reference.consumer_source_path
        )
    })?;
    Ok(Some(ConsumerSourceArtifact {
        path: reference.consumer_source_path.clone(),
        blake3: blake3::hash(text.as_bytes()).to_hex().to_string(),
        text,
    }))
}

fn scaffold_proof_evidence_ref(
    deck: &ProofCardDeck,
    proof_or_property_id: &str,
    reference_id: &str,
    consumer_context: &str,
    consumer_source: &ConsumerSourceArtifact,
    anchor: &str,
) -> Result<ProofEvidenceRef, Vec<String>> {
    let mut diagnostics = proof_card_deck_shape_diagnostics(deck);
    let Some(card) = unique_card(deck, proof_or_property_id) else {
        diagnostics.push(unmatched_card_diagnostic(deck, proof_or_property_id));
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    };
    if !consumer_source.text.contains(anchor) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_ANCHOR}` anchor not found in `{}`",
            consumer_source.path
        ));
    }
    let reference = ProofEvidenceRef {
        schema_version: PROOF_EVIDENCE_REF_SCHEMA_VERSION.to_owned(),
        reference_id: reference_id.to_owned(),
        consumer_context: consumer_context.to_owned(),
        consumer_source_path: consumer_source.path.clone(),
        consumer_source_blake3: consumer_source.blake3.clone(),
        consumer_source_anchor: anchor.to_owned(),
        proof_or_property_id: card.proof_or_property_id.clone(),
        source_requirement_or_policy_id: card.source_requirement_or_policy_id.clone(),
        checked_rust_source_scope: card.checked_rust_source_scope.clone(),
        expected_evidence_tier: card.evidence_tier.clone(),
        required_status: card.status.clone(),
        required_artifacts: card.artifact_blake3_identities.iter().map(|identity| identity.artifact.clone()).collect(),
        non_claims: card.non_claims.clone(),
    };
    diagnostics.extend(validate_proof_evidence_ref(&reference, deck, Some(consumer_source)).diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(reference)
    } else {
        Err(diagnostics)
    }
}

fn validate_proof_evidence_ref(
    reference: &ProofEvidenceRef,
    deck: &ProofCardDeck,
    consumer_source: Option<&ConsumerSourceArtifact>,
) -> ProofRefValidation {
    let mut diagnostics = proof_ref_shape_diagnostics(reference);
    diagnostics.extend(proof_card_deck_shape_diagnostics(deck));
    diagnostics.extend(proof_ref_overclaim_diagnostics(reference));
    let mut consumer_source_matched = false;
    if diagnostics.is_empty() {
        diagnostics.extend(proof_ref_linkage_diagnostics(reference, deck));
        let consumer_source_diagnostics = consumer_source_linkage_diagnostics(reference, consumer_source);
        consumer_source_matched = !reference.consumer_source_path.trim().is_empty()
            && consumer_source.is_some()
            && consumer_source_diagnostics.is_empty();
        diagnostics.extend(consumer_source_diagnostics);
    }
    diagnostics.sort();
    diagnostics.dedup();
    ProofRefValidation {
        diagnostics,
        consumer_source_matched,
    }
}

fn proof_ref_shape_diagnostics(reference: &ProofEvidenceRef) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if reference.schema_version != PROOF_EVIDENCE_REF_SCHEMA_VERSION {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_SCHEMA_VERSION}` expected `{PROOF_EVIDENCE_REF_SCHEMA_VERSION}` but got `{}`",
            reference.schema_version
        ));
    }
    push_missing_field(&mut diagnostics, FIELD_REFERENCE_ID, &reference.reference_id);
    push_missing_field(&mut diagnostics, FIELD_CONSUMER_CONTEXT, &reference.consumer_context);
    diagnostics.extend(consumer_source_shape_diagnostics(reference));
    push_missing_field(&mut diagnostics, FIELD_PROOF_OR_PROPERTY_ID, &reference.proof_or_property_id);
    push_missing_field(
        &mut diagnostics,
        FIELD_SOURCE_REQUIREMENT_OR_POLICY_ID,
        &reference.source_requirement_or_policy_id,
    );
    push_missing_field(&mut diagnostics, FIELD_CHECKED_RUST_SOURCE_SCOPE, &reference.checked_rust_source_scope);
    push_missing_field(&mut diagnostics, FIELD_EXPECTED_EVIDENCE_TIER, &reference.expected_evidence_tier);
    push_missing_field(&mut diagnostics, FIELD_REQUIRED_STATUS, &reference.required_status);
    push_missing_collection(&mut diagnostics, FIELD_REQUIRED_ARTIFACTS, &reference.required_artifacts);
    push_missing_collection(&mut diagnostics, FIELD_NON_CLAIMS, &reference.non_claims);
    if !matches!(
        reference.expected_evidence_tier.as_str(),
        EVIDENCE_TIER_FORMAL_PROOF | EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED
    ) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_EXPECTED_EVIDENCE_TIER}` has unsupported tier `{}`",
            reference.expected_evidence_tier
        ));
    }
    if !matches!(reference.required_status.as_str(), STATUS_PASSED | STATUS_MANUAL_REVIEW_REQUIRED) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_REQUIRED_STATUS}` has unsupported status `{}`",
            reference.required_status
        ));
    }
    for required in required_non_claims() {
        if !reference.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("proof evidence ref field `{FIELD_NON_CLAIMS}` missing `{required}`"));
        }
    }
    diagnostics
}

fn consumer_source_shape_diagnostics(reference: &ProofEvidenceRef) -> Vec<String> {
    let source_fields_present = [
        reference.consumer_source_path.as_str(),
        reference.consumer_source_blake3.as_str(),
        reference.consumer_source_anchor.as_str(),
    ]
    .iter()
    .any(|value| !value.trim().is_empty());
    if !source_fields_present {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    push_missing_field(&mut diagnostics, FIELD_CONSUMER_SOURCE_PATH, &reference.consumer_source_path);
    push_missing_field(&mut diagnostics, FIELD_CONSUMER_SOURCE_BLAKE3, &reference.consumer_source_blake3);
    push_missing_field(&mut diagnostics, FIELD_CONSUMER_SOURCE_ANCHOR, &reference.consumer_source_anchor);
    if !reference.consumer_source_anchor.contains(&reference.reference_id) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_ANCHOR}` must contain reference ID `{}`",
            reference.reference_id
        ));
    }
    if !reference.consumer_source_anchor.contains(&reference.proof_or_property_id) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_ANCHOR}` must contain proof or property ID `{}`",
            reference.proof_or_property_id
        ));
    }
    diagnostics
}

fn proof_card_deck_shape_diagnostics(deck: &ProofCardDeck) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if deck.schema_version != PROOF_CARDS_SCHEMA_VERSION {
        diagnostics.push(format!(
            "proof card deck field `{FIELD_SCHEMA_VERSION}` expected `{PROOF_CARDS_SCHEMA_VERSION}` but got `{}`",
            deck.schema_version
        ));
    }
    for (index, card) in deck.cards.iter().enumerate() {
        if card.proof_or_property_id.trim().is_empty() {
            diagnostics.push(format!("proof card deck card {index} is missing field `{FIELD_PROOF_OR_PROPERTY_ID}`"));
        } else if deck.cards.iter().take(index).any(|other| other.proof_or_property_id == card.proof_or_property_id) {
            diagnostics.push(format!(
                "proof card deck contains duplicate proof_or_property_id `{}`",
                card.proof_or_property_id
            ));
        }
        if let Some(record) = card.negative_check.as_ref()
            && !matches!(
                record.schema_version.as_str(),
                NEGATIVE_CHECK_SCHEMA_VERSION_V1 | NEGATIVE_CHECK_SCHEMA_VERSION_V2
            )
        {
            diagnostics.push(format!(
                "proof card `{}` negative_check has unsupported schema `{}`",
                card.proof_or_property_id, record.schema_version
            ));
        }
    }
    diagnostics
}

/// Recomputes the negative-control decision from the retained record, evidence, and measured
/// identities instead of trusting the annotated `status`. A still-passing mutant is
/// `property_not_demonstrated`; absent or unattributable verifier evidence is `inconclusive`.
pub(crate) fn negative_control_verdict(
    record: &ProofCardNegativeCheck,
    expected: &NegativeControlExpectation<'_>,
) -> NegativeControlVerdict {
    let mut problems = Vec::new();
    if expected.proof_or_property_id.trim().is_empty() {
        problems.push("negative control expected proof or property ID is missing".to_owned());
    }
    if expected.baseline_card_status != STATUS_PASSED {
        problems.push(format!(
            "negative control requires a passing baseline proof card; found `{}`",
            expected.baseline_card_status
        ));
        return inconclusive_negative_verdict(problems);
    }
    if !expected.baseline_card_diagnostics.is_empty() {
        problems.push(format!(
            "negative control requires a clean baseline proof card; it has diagnostics: {}",
            expected.baseline_card_diagnostics.join("; ")
        ));
        return inconclusive_negative_verdict(problems);
    }
    match record.schema_version.as_str() {
        NEGATIVE_CHECK_SCHEMA_VERSION_V1 => return v1_negative_control_verdict(record, problems),
        NEGATIVE_CHECK_SCHEMA_VERSION_V2 => {}
        other => {
            problems.push(format!("negative control record has unsupported schema `{other}`"));
            return inconclusive_negative_verdict(problems);
        }
    }
    if record.mutant_card_status == STATUS_PASSED {
        if record.evidence.as_ref().is_some_and(|evidence| evidence.mutant.outcome == NEGATIVE_RUN_REJECTED) {
            problems.push("mutant proof card still passes but verifier evidence claims rejection".to_owned());
        }
        return NegativeControlVerdict {
            status: NEGATIVE_STATUS_NOT_DEMONSTRATED,
            problems,
        };
    }
    if record.mutant_card_status != STATUS_FAILED {
        problems.push(format!(
            "negative control mutant proof card status `{}` is not a verifier rejection",
            record.mutant_card_status
        ));
        return inconclusive_negative_verdict(problems);
    }
    let Some(evidence) = record.evidence.as_ref() else {
        problems.push("negative control is missing verifier evidence".to_owned());
        return inconclusive_negative_verdict(problems);
    };
    let Some(measured) = record.measured.as_ref() else {
        problems.push("negative control is missing measured source and deck identities".to_owned());
        return inconclusive_negative_verdict(problems);
    };
    negative_evidence_shape_problems(evidence, &mut problems);
    negative_evidence_binding_problems(evidence, expected, &mut problems);
    negative_run_binding_problems(&evidence.baseline, "baseline", evidence, &mut problems);
    negative_run_binding_problems(&evidence.mutant, "mutant", evidence, &mut problems);
    negative_outcome_problems(evidence, &mut problems);
    negative_measurement_binding_problems(&measured.binding, expected, evidence, &mut problems);
    negative_measurement_artifact_problems(record, evidence, measured, expected, &mut problems);
    if problems.is_empty() {
        NegativeControlVerdict {
            status: NEGATIVE_STATUS_DEMONSTRATED,
            problems,
        }
    } else {
        inconclusive_negative_verdict(problems)
    }
}

fn v1_negative_control_verdict(record: &ProofCardNegativeCheck, mut problems: Vec<String>) -> NegativeControlVerdict {
    match record.status.as_str() {
        NEGATIVE_STATUS_DEMONSTRATED => {
            problems.push(format!(
                "negative control schema `{NEGATIVE_CHECK_SCHEMA_VERSION_V1}` cannot attribute verifier rejection; re-record with `{NEGATIVE_CHECK_SCHEMA_VERSION_V2}` and explicit verifier evidence"
            ));
            inconclusive_negative_verdict(problems)
        }
        NEGATIVE_STATUS_NOT_DEMONSTRATED => NegativeControlVerdict {
            status: NEGATIVE_STATUS_NOT_DEMONSTRATED,
            problems: Vec::new(),
        },
        NEGATIVE_STATUS_INCONCLUSIVE => NegativeControlVerdict {
            status: NEGATIVE_STATUS_INCONCLUSIVE,
            problems: Vec::new(),
        },
        other => {
            problems.push(format!("negative control record has unsupported status `{other}`"));
            inconclusive_negative_verdict(problems)
        }
    }
}

fn inconclusive_negative_verdict(problems: Vec<String>) -> NegativeControlVerdict {
    NegativeControlVerdict {
        status: NEGATIVE_STATUS_INCONCLUSIVE,
        problems,
    }
}

fn negative_evidence_shape_problems(evidence: &NegativeControlEvidence, problems: &mut Vec<String>) {
    if evidence.schema_version != NEGATIVE_CHECK_EVIDENCE_SCHEMA_VERSION {
        problems.push(format!(
            "negative control evidence field `schema_version` expected `{NEGATIVE_CHECK_EVIDENCE_SCHEMA_VERSION}` but got `{}`",
            evidence.schema_version
        ));
    }
    for (field, value) in [
        ("verifier", evidence.verifier.as_str()),
        ("toolchain", evidence.toolchain.as_str()),
        ("verification_mode", evidence.verification_mode.as_str()),
        ("verification_configuration_blake3", evidence.verification_configuration_blake3.as_str()),
        ("approved_specification_id", evidence.approved_specification_id.as_str()),
        ("proof_or_property_id", evidence.proof_or_property_id.as_str()),
        ("source_requirement_or_policy_id", evidence.source_requirement_or_policy_id.as_str()),
        ("checked_rust_source_scope", evidence.checked_rust_source_scope.as_str()),
        ("evidence_tier", evidence.evidence_tier.as_str()),
        ("mutation_id", evidence.mutation.mutation_id.as_str()),
        ("patch_blake3", evidence.mutation.patch_blake3.as_str()),
        ("mutation.baseline_source_blake3", evidence.mutation.baseline_source_blake3.as_str()),
        ("mutation.mutant_source_blake3", evidence.mutation.mutant_source_blake3.as_str()),
    ] {
        if value.trim().is_empty() {
            problems.push(format!("negative control evidence field `{field}` is missing"));
        }
    }
    if evidence.command_argv.is_empty() || evidence.command_argv.iter().any(|argument| argument.trim().is_empty()) {
        problems.push("negative control evidence field `command_argv` must retain a non-empty command".to_owned());
    }
}

fn negative_evidence_binding_problems(
    evidence: &NegativeControlEvidence,
    expected: &NegativeControlExpectation<'_>,
    problems: &mut Vec<String>,
) {
    for (field, actual, expected_value) in [
        ("proof_or_property_id", evidence.proof_or_property_id.as_str(), expected.proof_or_property_id),
        (
            "source_requirement_or_policy_id",
            evidence.source_requirement_or_policy_id.as_str(),
            expected.source_requirement_or_policy_id,
        ),
        (
            "checked_rust_source_scope",
            evidence.checked_rust_source_scope.as_str(),
            expected.checked_rust_source_scope,
        ),
        ("evidence_tier", evidence.evidence_tier.as_str(), expected.evidence_tier),
        ("verifier", evidence.verifier.as_str(), expected.verifier_identity_summary),
        ("toolchain", evidence.toolchain.as_str(), expected.toolchain_identity_summary),
        ("verification_mode", evidence.verification_mode.as_str(), expected.verifier_mode),
    ] {
        if expected_value.trim().is_empty() {
            problems.push(format!("negative control proof card field `{field}` is missing"));
        } else if actual != expected_value {
            problems.push(format!(
                "negative control evidence field `{field}` expected `{expected_value}` from the proof card but got `{actual}`"
            ));
        }
    }
}

fn negative_run_binding_problems(
    run: &NegativeControlVerifierRun,
    label: &str,
    evidence: &NegativeControlEvidence,
    problems: &mut Vec<String>,
) {
    for (field, value) in [
        ("outcome", run.outcome.as_str()),
        ("approved_specification_id", run.approved_specification_id.as_str()),
        ("proof_or_property_id", run.proof_or_property_id.as_str()),
        ("source_requirement_or_policy_id", run.source_requirement_or_policy_id.as_str()),
        ("checked_rust_source_scope", run.checked_rust_source_scope.as_str()),
        ("source_path", run.source_path.as_str()),
        ("source_blake3", run.source_blake3.as_str()),
        ("deck_path", run.deck_path.as_str()),
        ("deck_blake3", run.deck_blake3.as_str()),
    ] {
        if value.trim().is_empty() {
            problems.push(format!("negative control {label} verifier evidence field `{field}` is missing"));
        }
    }
    for (field, actual, expected) in [
        (
            "approved_specification_id",
            run.approved_specification_id.as_str(),
            evidence.approved_specification_id.as_str(),
        ),
        ("proof_or_property_id", run.proof_or_property_id.as_str(), evidence.proof_or_property_id.as_str()),
        (
            "source_requirement_or_policy_id",
            run.source_requirement_or_policy_id.as_str(),
            evidence.source_requirement_or_policy_id.as_str(),
        ),
        (
            "checked_rust_source_scope",
            run.checked_rust_source_scope.as_str(),
            evidence.checked_rust_source_scope.as_str(),
        ),
    ] {
        if actual != expected {
            problems.push(format!(
                "negative control {label} verifier evidence field `{field}` expected `{expected}` but got `{actual}`"
            ));
        }
    }
}

fn negative_outcome_problems(evidence: &NegativeControlEvidence, problems: &mut Vec<String>) {
    for (label, run) in [("baseline", &evidence.baseline), ("mutant", &evidence.mutant)] {
        if !run.outcome.is_empty() && !negative_outcome_supported(&run.outcome) {
            problems.push(format!("negative control {label} verifier outcome `{}` is unsupported", run.outcome));
        }
    }
    if evidence.baseline.outcome != NEGATIVE_RUN_ACCEPTED {
        problems.push(format!(
            "negative control baseline verifier outcome `{}` is not an accepted run",
            evidence.baseline.outcome
        ));
    }
    if evidence
        .baseline
        .rejection_attribution
        .as_deref()
        .is_some_and(|attribution| !attribution.trim().is_empty())
    {
        problems.push("negative control baseline verifier run unexpectedly attributes a rejection".to_owned());
    }
    match evidence.mutant.outcome.as_str() {
        NEGATIVE_RUN_REJECTED => match evidence.mutant.rejection_attribution.as_deref() {
            Some(attribution) if attribution == evidence.proof_or_property_id => {}
            Some(attribution) => problems.push(format!(
                "negative control mutant rejection attributed to `{attribution}` instead of expected obligation `{}`",
                evidence.proof_or_property_id
            )),
            None => problems.push("negative control mutant rejection has no obligation attribution".to_owned()),
        },
        NEGATIVE_RUN_ACCEPTED => {
            problems.push("negative control mutant verifier run was accepted; property not discriminated".to_owned());
        }
        other => problems.push(format!(
            "negative control mutant verifier outcome `{other}` is not an attributable obligation rejection"
        )),
    }
}

fn negative_outcome_supported(outcome: &str) -> bool {
    matches!(
        outcome,
        NEGATIVE_RUN_ACCEPTED
            | NEGATIVE_RUN_REJECTED
            | NEGATIVE_RUN_COMPILE_ERROR
            | NEGATIVE_RUN_CRASH
            | NEGATIVE_RUN_TIMEOUT
            | NEGATIVE_RUN_NO_EVIDENCE
    )
}

fn negative_measurement_binding_problems(
    binding: &NegativeControlCardBinding,
    expected: &NegativeControlExpectation<'_>,
    evidence: &NegativeControlEvidence,
    problems: &mut Vec<String>,
) {
    for (field, baseline, mutant) in [
        (
            "proof_or_property_id",
            binding.baseline_proof_or_property_id.as_str(),
            binding.mutant_proof_or_property_id.as_str(),
        ),
        (
            "source_requirement_or_policy_id",
            binding.baseline_source_requirement_or_policy_id.as_str(),
            binding.mutant_source_requirement_or_policy_id.as_str(),
        ),
        ("policy_id", binding.baseline_policy_id.as_str(), binding.mutant_policy_id.as_str()),
        (
            "checked_rust_source_scope",
            binding.baseline_checked_rust_source_scope.as_str(),
            binding.mutant_checked_rust_source_scope.as_str(),
        ),
        ("evidence_tier", binding.baseline_evidence_tier.as_str(), binding.mutant_evidence_tier.as_str()),
        ("verifier_mode", binding.baseline_verifier_mode.as_str(), binding.mutant_verifier_mode.as_str()),
        (
            "verifier_identity_summary",
            binding.baseline_verifier_identity_summary.as_str(),
            binding.mutant_verifier_identity_summary.as_str(),
        ),
        (
            "toolchain_identity_summary",
            binding.baseline_toolchain_identity_summary.as_str(),
            binding.mutant_toolchain_identity_summary.as_str(),
        ),
    ] {
        if baseline.trim().is_empty() {
            problems.push(format!("negative control baseline proof card field `{field}` is missing"));
        }
        if mutant.trim().is_empty() {
            problems.push(format!("negative control mutant proof card field `{field}` is missing"));
        }
        if baseline != mutant {
            problems.push(format!(
                "negative control proof cards differ on `{field}`: baseline `{baseline}` vs mutant `{mutant}`"
            ));
        }
    }
    for (field, actual, expected_value) in [
        (
            "proof_or_property_id",
            binding.baseline_proof_or_property_id.as_str(),
            expected.proof_or_property_id,
        ),
        (
            "source_requirement_or_policy_id",
            binding.baseline_source_requirement_or_policy_id.as_str(),
            expected.source_requirement_or_policy_id,
        ),
        ("policy_id", binding.baseline_policy_id.as_str(), expected.policy_id),
        (
            "checked_rust_source_scope",
            binding.baseline_checked_rust_source_scope.as_str(),
            expected.checked_rust_source_scope,
        ),
        ("evidence_tier", binding.baseline_evidence_tier.as_str(), expected.evidence_tier),
        ("verifier_mode", binding.baseline_verifier_mode.as_str(), expected.verifier_mode),
        (
            "verifier_identity_summary",
            binding.baseline_verifier_identity_summary.as_str(),
            expected.verifier_identity_summary,
        ),
        (
            "toolchain_identity_summary",
            binding.baseline_toolchain_identity_summary.as_str(),
            expected.toolchain_identity_summary,
        ),
    ] {
        if actual != expected_value {
            problems.push(format!(
                "negative control retained baseline proof card field `{field}` expected `{expected_value}` but got `{actual}`"
            ));
        }
    }
    if !negative_path_sets_match(&binding.baseline_source_paths, expected.baseline_source_paths) {
        problems.push(
            "negative control retained baseline source_paths do not match the proof card source locators".to_owned(),
        );
    }
    if binding.baseline_source_paths.is_empty() || binding.mutant_source_paths.is_empty() {
        problems.push("negative control proof cards must retain source_paths for locator binding".to_owned());
    }
    if !binding
        .baseline_source_paths
        .iter()
        .any(|path| negative_locators_match(path, &evidence.baseline.source_path))
    {
        problems.push(format!(
            "negative control verifier evidence baseline source `{}` is not declared by the baseline proof card source_paths",
            evidence.baseline.source_path
        ));
    }
    if !binding
        .mutant_source_paths
        .iter()
        .any(|path| negative_locators_match(path, &evidence.mutant.source_path))
    {
        problems.push(format!(
            "negative control verifier evidence mutant source `{}` is not declared by the mutant proof card source_paths",
            evidence.mutant.source_path
        ));
    }
    if !declared_mutation_only_source_paths(
        &binding.baseline_source_paths,
        &binding.mutant_source_paths,
        &evidence.baseline.source_path,
        &evidence.mutant.source_path,
    ) {
        problems
            .push("negative control proof card source_paths differ outside the declared source mutation".to_owned());
    }
}

fn negative_measurement_artifact_problems(
    record: &ProofCardNegativeCheck,
    evidence: &NegativeControlEvidence,
    measured: &NegativeControlMeasurement,
    expected: &NegativeControlExpectation<'_>,
    problems: &mut Vec<String>,
) {
    for (field, value) in [
        ("baseline_source_blake3", measured.baseline_source_blake3.as_str()),
        ("mutant_source_blake3", measured.mutant_source_blake3.as_str()),
        ("mutation_id", measured.mutation_id.as_str()),
        ("patch_blake3", measured.patch_blake3.as_str()),
        ("baseline_deck_blake3", measured.baseline_deck_blake3.as_str()),
        ("mutant_deck_blake3", measured.mutant_deck_blake3.as_str()),
    ] {
        if value.trim().is_empty() {
            problems.push(format!("negative control measured field `{field}` is missing"));
        }
    }
    for (field, actual, expected_value) in [
        (
            "baseline_source_blake3",
            measured.baseline_source_blake3.as_str(),
            evidence.baseline.source_blake3.as_str(),
        ),
        (
            "mutant_source_blake3",
            measured.mutant_source_blake3.as_str(),
            evidence.mutant.source_blake3.as_str(),
        ),
        (
            "baseline_deck_blake3",
            measured.baseline_deck_blake3.as_str(),
            evidence.baseline.deck_blake3.as_str(),
        ),
        ("mutant_deck_blake3", measured.mutant_deck_blake3.as_str(), evidence.mutant.deck_blake3.as_str()),
        ("mutation_id", measured.mutation_id.as_str(), evidence.mutation.mutation_id.as_str()),
        ("patch_blake3", measured.patch_blake3.as_str(), evidence.mutation.patch_blake3.as_str()),
        (
            "mutation.baseline_source_blake3",
            measured.baseline_source_blake3.as_str(),
            evidence.mutation.baseline_source_blake3.as_str(),
        ),
        (
            "mutation.mutant_source_blake3",
            measured.mutant_source_blake3.as_str(),
            evidence.mutation.mutant_source_blake3.as_str(),
        ),
    ] {
        if actual != expected_value {
            problems.push(format!(
                "negative control measured `{field}` `{actual}` does not match retained evidence `{expected_value}`"
            ));
        }
    }
    if record.mutant_cards_blake3 != measured.mutant_deck_blake3 {
        problems.push(format!(
            "negative control mutant deck identity `{}` does not match measured mutant deck `{}`",
            record.mutant_cards_blake3, measured.mutant_deck_blake3
        ));
    }
    if measured.baseline_source_blake3 == measured.mutant_source_blake3 {
        problems.push("negative control mutant source bytes are unchanged from the baseline source".to_owned());
    }
    if !negative_locators_match(&record.mutant_source, &evidence.mutant.source_path) {
        problems.push(format!(
            "negative control mutant source locator `{}` does not match verifier evidence source `{}`",
            record.mutant_source, evidence.mutant.source_path
        ));
    }
    let require_readable = expected.require_readable_artifacts;
    for (label, path, expected_hash, measured_hash) in [
        (
            "baseline source",
            evidence.baseline.source_path.as_str(),
            evidence.baseline.source_blake3.as_str(),
            measured.baseline_source_blake3.as_str(),
        ),
        (
            "mutant source",
            evidence.mutant.source_path.as_str(),
            evidence.mutant.source_blake3.as_str(),
            measured.mutant_source_blake3.as_str(),
        ),
    ] {
        if let Some(actual) = verify_retained_artifact(path, expected_hash, label, require_readable, false, problems)
            && actual != measured_hash
        {
            problems.push(format!(
                "negative control measured `{label}` BLAKE3 `{measured_hash}` does not match current bytes `{actual}`"
            ));
        }
    }
    if require_readable {
        for (label, path, expected_hash, measured_hash) in [
            (
                "baseline proof-card deck",
                evidence.baseline.deck_path.as_str(),
                evidence.baseline.deck_blake3.as_str(),
                measured.baseline_deck_blake3.as_str(),
            ),
            (
                "mutant proof-card deck",
                evidence.mutant.deck_path.as_str(),
                evidence.mutant.deck_blake3.as_str(),
                measured.mutant_deck_blake3.as_str(),
            ),
        ] {
            if let Some(actual) = verify_retained_artifact(path, expected_hash, label, true, true, problems)
                && actual != measured_hash
            {
                problems.push(format!(
                    "negative control measured `{label}` BLAKE3 `{measured_hash}` does not match current bytes `{actual}`"
                ));
            }
        }
    }
    for (label, run) in [("baseline", &evidence.baseline), ("mutant", &evidence.mutant)] {
        if let Some(raw_output) = run.raw_output.as_ref() {
            verify_retained_artifact(
                &raw_output.artifact,
                &raw_output.blake3,
                &format!("{label} verifier output"),
                true,
                true,
                problems,
            );
        }
    }
}

/// Re-hashes a retained artifact when it is readable. A readable artifact must match the retained
/// identity; the recorder additionally requires every declared artifact to be readable.
fn verify_retained_artifact(
    path: &str,
    expected_blake3: &str,
    label: &str,
    require_readable: bool,
    require_non_empty: bool,
    problems: &mut Vec<String>,
) -> Option<String> {
    if path.trim().is_empty() {
        problems.push(format!("negative control {label} path is missing"));
        return None;
    }
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            if require_readable {
                problems.push(format!("negative control {label} `{path}` is not readable: {error}"));
            }
            return None;
        }
    };
    if require_non_empty && bytes.is_empty() {
        problems.push(format!("negative control {label} `{path}` is empty"));
    }
    let actual = blake3::hash(&bytes).to_hex().to_string();
    if expected_blake3.trim().is_empty() {
        problems.push(format!("negative control {label} BLAKE3 identity is missing"));
    } else if actual != expected_blake3 {
        problems.push(format!(
            "negative control {label} `{path}` BLAKE3 `{actual}` does not match retained identity `{expected_blake3}`"
        ));
    }
    Some(actual)
}

pub(crate) fn normalize_negative_locator(path: &str) -> String {
    let mut normalized = path.trim().replace('\\', "/");
    while let Some(rest) = normalized.strip_prefix("./") {
        normalized = rest.to_owned();
    }
    normalized
}

pub(crate) fn negative_locators_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let normalized_left = normalize_negative_locator(left);
    let normalized_right = normalize_negative_locator(right);
    if !normalized_left.is_empty() && normalized_left == normalized_right {
        return true;
    }
    matches!(
        (fs::canonicalize(left), fs::canonicalize(right)),
        (Ok(left), Ok(right)) if left == right
    )
}

fn normalized_sorted_source_paths(paths: &[String]) -> Vec<String> {
    let mut normalized = paths
        .iter()
        .map(|path| normalize_negative_locator(path))
        .filter(|path| !path.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn negative_path_sets_match(left: &[String], right: &[String]) -> bool {
    normalized_sorted_source_paths(left) == normalized_sorted_source_paths(right)
}

fn declared_mutation_only_source_paths(
    baseline_paths: &[String],
    mutant_paths: &[String],
    baseline_source: &str,
    mutant_source: &str,
) -> bool {
    let baseline_paths = normalized_sorted_source_paths(baseline_paths);
    let mutant_paths = normalized_sorted_source_paths(mutant_paths);
    if baseline_paths == mutant_paths {
        return true;
    }
    let baseline_source = normalize_negative_locator(baseline_source);
    let Some(index) = baseline_paths.iter().position(|path| negative_locators_match(path, &baseline_source)) else {
        return false;
    };
    let mut substituted = baseline_paths;
    substituted[index] = normalize_negative_locator(mutant_source);
    substituted.sort();
    substituted.dedup();
    substituted == mutant_paths
}

fn proof_ref_overclaim_diagnostics(reference: &ProofEvidenceRef) -> Vec<String> {
    let fields = [&reference.consumer_context, &reference.consumer_source_anchor]
        .into_iter()
        .chain(&reference.non_claims);
    let mut diagnostics = Vec::new();
    for field in fields {
        let wording = field.to_ascii_lowercase().replace('-', " ");
        for (phrase, claim) in OVERCLAIM_PHRASES {
            if positive_phrase_present(&wording, phrase) {
                diagnostics.push(format!("proof evidence ref contains unsupported {claim} claim"));
            }
        }
    }
    diagnostics
}

/// Fails validation when a card claims a demonstrated negative control that the retained evidence
/// does not reproduce. Non-demonstrated records make no positive claim, but an unsupported schema
/// still fails so a malformed record cannot be mistaken for a valid one.
fn negative_check_linkage_diagnostics(card: &ProofCard) -> Vec<String> {
    let Some(record) = card.negative_check.as_ref() else {
        return Vec::new();
    };
    if !matches!(record.schema_version.as_str(), NEGATIVE_CHECK_SCHEMA_VERSION_V1 | NEGATIVE_CHECK_SCHEMA_VERSION_V2) {
        return vec![format!(
            "proof card `{}` negative_check has unsupported schema `{}`",
            card.proof_or_property_id, record.schema_version
        )];
    }
    let expected = NegativeControlExpectation {
        proof_or_property_id: &card.proof_or_property_id,
        source_requirement_or_policy_id: &card.source_requirement_or_policy_id,
        policy_id: &card.policy_id,
        checked_rust_source_scope: &card.checked_rust_source_scope,
        evidence_tier: &card.evidence_tier,
        baseline_card_status: &card.status,
        baseline_card_diagnostics: &card.diagnostics,
        verifier_mode: &card.verifier_mode,
        verifier_identity_summary: &card.verifier_identity_summary,
        toolchain_identity_summary: &card.toolchain_identity_summary,
        baseline_source_paths: &card.source_paths,
        require_readable_artifacts: false,
    };
    let verdict = negative_control_verdict(record, &expected);
    if record.status == NEGATIVE_STATUS_DEMONSTRATED {
        let mut diagnostics = verdict.problems;
        if verdict.status != NEGATIVE_STATUS_DEMONSTRATED {
            diagnostics.push(format!(
                "proof card `{}` negative_check claims `{NEGATIVE_STATUS_DEMONSTRATED}` but retained artifacts recompute `{}`",
                card.proof_or_property_id, verdict.status
            ));
        }
        diagnostics
    } else if record.status == NEGATIVE_STATUS_NOT_DEMONSTRATED || record.status == NEGATIVE_STATUS_INCONCLUSIVE {
        Vec::new()
    } else {
        vec![format!(
            "proof card `{}` negative_check has unsupported status `{}`",
            card.proof_or_property_id, record.status
        )]
    }
}

fn proof_ref_linkage_diagnostics(reference: &ProofEvidenceRef, deck: &ProofCardDeck) -> Vec<String> {
    let Some(card) = matching_card(reference, deck) else {
        return vec![unmatched_card_diagnostic(deck, &reference.proof_or_property_id)];
    };
    let mut diagnostics = Vec::new();
    diagnostics.extend(negative_check_linkage_diagnostics(card));
    if !card.diagnostics.is_empty() {
        diagnostics.push(format!(
            "proof card `{}` has diagnostics: {}",
            card.proof_or_property_id,
            card.diagnostics.join("; ")
        ));
    }
    if card.evidence_origin == crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS
        && card.evidence_tier == EVIDENCE_TIER_FORMAL_PROOF
    {
        diagnostics.push(format!(
            "proof card `{}` promotes imported Trellis evidence to formal_proof",
            card.proof_or_property_id
        ));
    }
    push_mismatch(
        &mut diagnostics,
        FIELD_SOURCE_REQUIREMENT_OR_POLICY_ID,
        &reference.source_requirement_or_policy_id,
        &card.source_requirement_or_policy_id,
    );
    push_mismatch(
        &mut diagnostics,
        FIELD_CHECKED_RUST_SOURCE_SCOPE,
        &reference.checked_rust_source_scope,
        &card.checked_rust_source_scope,
    );
    push_mismatch(
        &mut diagnostics,
        FIELD_EXPECTED_EVIDENCE_TIER,
        &reference.expected_evidence_tier,
        &card.evidence_tier,
    );
    push_mismatch(&mut diagnostics, FIELD_REQUIRED_STATUS, &reference.required_status, &card.status);
    for artifact in &reference.required_artifacts {
        if !card.artifact_blake3_identities.iter().any(|identity| identity.artifact == *artifact) {
            diagnostics.push(format!(
                "proof evidence ref field `{FIELD_REQUIRED_ARTIFACTS}` requires artifact `{artifact}` missing from proof card"
            ));
        }
    }
    for non_claim in &reference.non_claims {
        if !card.non_claims.iter().any(|card_claim| card_claim == non_claim) {
            diagnostics.push(format!(
                "proof evidence ref field `{FIELD_NON_CLAIMS}` requires non-claim `{non_claim}` missing from proof card"
            ));
        }
    }
    diagnostics
}

fn consumer_source_linkage_diagnostics(
    reference: &ProofEvidenceRef,
    consumer_source: Option<&ConsumerSourceArtifact>,
) -> Vec<String> {
    if reference.consumer_source_path.trim().is_empty() {
        return Vec::new();
    }
    let Some(source) = consumer_source else {
        return vec![format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_PATH}` has no readable source artifact `{}`",
            reference.consumer_source_path
        )];
    };
    let mut diagnostics = Vec::new();
    push_consumer_source_mismatch(
        &mut diagnostics,
        FIELD_CONSUMER_SOURCE_PATH,
        &reference.consumer_source_path,
        &source.path,
    );
    push_consumer_source_mismatch(
        &mut diagnostics,
        FIELD_CONSUMER_SOURCE_BLAKE3,
        &reference.consumer_source_blake3,
        &source.blake3,
    );
    if !source.text.contains(&reference.consumer_source_anchor) {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_CONSUMER_SOURCE_ANCHOR}` anchor not found in `{}`",
            reference.consumer_source_path
        ));
    }
    diagnostics
}

fn matching_card<'a>(reference: &ProofEvidenceRef, deck: &'a ProofCardDeck) -> Option<&'a ProofCard> {
    unique_card(deck, &reference.proof_or_property_id)
}

/// Returns the card for `proof_or_property_id` only when exactly one card in the deck carries it.
fn unique_card<'a>(deck: &'a ProofCardDeck, proof_or_property_id: &str) -> Option<&'a ProofCard> {
    let mut matches = deck.cards.iter().filter(|card| card.proof_or_property_id == proof_or_property_id);
    let card = matches.next()?;
    matches.next().is_none().then_some(card)
}

fn unmatched_card_diagnostic(deck: &ProofCardDeck, proof_or_property_id: &str) -> String {
    let matches = deck.cards.iter().filter(|card| card.proof_or_property_id == proof_or_property_id).count();
    if matches == 0 {
        format!(
            "proof evidence ref field `{FIELD_PROOF_OR_PROPERTY_ID}` references missing proof card `{proof_or_property_id}`"
        )
    } else {
        format!(
            "proof evidence ref field `{FIELD_PROOF_OR_PROPERTY_ID}` matches {matches} duplicate proof cards `{proof_or_property_id}`; the deck must contain exactly one"
        )
    }
}

fn push_missing_field(diagnostics: &mut Vec<String>, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("proof evidence ref missing required field `{field_name}`"));
    }
}

fn push_missing_collection(diagnostics: &mut Vec<String>, field_name: &str, values: &[String]) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("proof evidence ref missing required field `{field_name}`"));
    }
}

fn push_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &str, actual: &str) {
    if expected != actual {
        diagnostics.push(format!(
            "proof evidence ref field `{field_name}` expected `{expected}` but proof card has `{actual}`"
        ));
    }
}

fn push_consumer_source_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &str, actual: &str) {
    if expected != actual {
        diagnostics.push(format!(
            "proof evidence ref field `{field_name}` expected `{expected}` but consumer source has `{actual}`"
        ));
    }
}

fn required_non_claims() -> &'static [&'static str] {
    &[
        NON_CLAIM_WHOLE_PROGRAM,
        NON_CLAIM_CERTIFICATION,
        NON_CLAIM_BUSINESS_DOMAIN,
        NON_CLAIM_SEMANTIC_EQUIVALENCE,
        NON_CLAIM_FULL_SAFETY,
    ]
}

fn print_report(output_format: OutputFormat, report: &ProofRefReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet proof-ref reference_id={} proof_or_property_id={} valid={} matched_card={} consumer_source_matched={} diagnostics={}",
                report.reference_id,
                report.proof_or_property_id,
                report.valid,
                report.matched_card,
                report.consumer_source_matched,
                report.diagnostics.len()
            );
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            let text =
                serde_json::to_string_pretty(report).map_err(|error| format!("serialize proof-ref report: {error}"))?;
            println!("{text}");
            Ok(())
        }
        OutputFormat::Sarif => Err("proof-ref does not support SARIF output; use human or json".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROPERTY_ID: &str = "octet.verus_proof_rails.addition-bound-production#rail-0";
    const POLICY_ID: &str = "octet.verus_proof_rails.addition-bound-production";
    const CHECKED_SCOPE: &str = "formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0";
    const RECEIPT_ARTIFACT: &str = "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt";

    #[test]
    fn proof_ref_accepts_matching_formal_proof_card() {
        let reference = matching_reference();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn proof_ref_accepts_matching_consumer_source_anchor() {
        let mut reference = matching_reference();
        let source = source_artifact_for(&mut reference);
        let deck = deck_with(card());

        let validation = validate_proof_evidence_ref(&reference, &deck, Some(&source));

        assert!(validation.diagnostics.is_empty(), "unexpected diagnostics: {:?}", validation.diagnostics);
        assert!(validation.consumer_source_matched);
    }

    #[test]
    fn proof_ref_scaffold_builds_valid_reference_from_card_and_source_anchor() {
        let mut anchor_reference = matching_reference();
        let source = source_artifact_for(&mut anchor_reference);
        let deck = deck_with(card());

        let scaffolded = scaffold_proof_evidence_ref(
            &deck,
            PROPERTY_ID,
            &anchor_reference.reference_id,
            &anchor_reference.consumer_context,
            &source,
            &anchor_reference.consumer_source_anchor,
        )
        .expect("scaffold reference");
        let diagnostics = validate_proof_evidence_ref(&scaffolded, &deck, Some(&source)).diagnostics;

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(scaffolded.consumer_source_blake3, source.blake3);
        assert!(scaffolded.required_artifacts.iter().any(|artifact| artifact == RECEIPT_ARTIFACT));
    }

    #[test]
    fn proof_ref_scaffold_command_writes_reference_json() {
        let temp = tempfile::tempdir().expect("tempdir");
        let deck_path = temp.path().join("proof-cards.json");
        fs::write(&deck_path, serde_json::to_string_pretty(&deck_with(card())).expect("deck json"))
            .expect("write deck");
        let source_path = temp.path().join("src/codec.rs");
        fs::create_dir_all(source_path.parent().expect("source parent")).expect("create source parent");
        let anchor = format!("octet-proof-ref: {} -> {PROPERTY_ID}", matching_reference().reference_id);
        let source_text = format!("// {anchor}\npub fn codec() {{}}\n");
        fs::write(&source_path, &source_text).expect("write source");
        let output = temp.path().join("proof-ref.json");

        let code = run(&ProofRefArgs {
            mode: ProofRefMode::Scaffold(ProofRefScaffoldArgs {
                proof_cards: deck_path,
                proof_or_property_id: PROPERTY_ID.to_owned(),
                reference_id: "unchecked-narrowing-range-proof".to_owned(),
                consumer_context: "allow tigerstyle::unchecked_narrowing at src/codec.rs:42".to_owned(),
                consumer_source: source_path.clone(),
                anchor: anchor.clone(),
                output: Some(output.clone()),
            }),
        })
        .expect("scaffold command");
        let reference = read_json::<ProofEvidenceRef>(&output).expect("read generated reference");

        assert_eq!(code, EXIT_VALID);
        assert_eq!(reference.consumer_source_path, source_path.display().to_string().replace('\\', "/"));
        assert_eq!(reference.consumer_source_blake3, blake3::hash(source_text.as_bytes()).to_hex().to_string());
        assert_eq!(reference.consumer_source_anchor, anchor);
        assert_eq!(reference.proof_or_property_id, PROPERTY_ID);
    }

    #[test]
    fn proof_ref_scaffold_rejects_missing_card_or_missing_anchor() {
        let mut anchor_reference = matching_reference();
        let mut source = source_artifact_for(&mut anchor_reference);
        let deck = deck_with(card());

        let missing_card = scaffold_proof_evidence_ref(
            &deck,
            "missing#rail-0",
            &anchor_reference.reference_id,
            &anchor_reference.consumer_context,
            &source,
            &anchor_reference.consumer_source_anchor,
        )
        .expect_err("missing proof card should fail");
        assert!(missing_card.iter().any(|diagnostic| diagnostic.contains("references missing proof card")));

        source.text = "pub fn codec() {}\n".to_owned();
        let missing_anchor = scaffold_proof_evidence_ref(
            &deck,
            PROPERTY_ID,
            &anchor_reference.reference_id,
            &anchor_reference.consumer_context,
            &source,
            &anchor_reference.consumer_source_anchor,
        )
        .expect_err("missing anchor should fail");
        assert!(missing_anchor.iter().any(|diagnostic| diagnostic.contains("anchor not found")));
    }

    #[test]
    fn proof_ref_rejects_missing_card() {
        let mut reference = matching_reference();
        reference.proof_or_property_id = "missing#rail-0".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("references missing proof card")));
    }

    #[test]
    fn proof_ref_rejects_mismatched_scope_tier_status_artifact_and_non_claim() {
        let mut reference = matching_reference();
        reference.checked_rust_source_scope = "wrong-scope".to_owned();
        reference.expected_evidence_tier = EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED.to_owned();
        reference.required_status = STATUS_MANUAL_REVIEW_REQUIRED.to_owned();
        reference.required_artifacts.push("missing-artifact".to_owned());
        reference.non_claims.push("not theorem prover completeness".to_owned());
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CHECKED_RUST_SOURCE_SCOPE)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_EXPECTED_EVIDENCE_TIER)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_REQUIRED_STATUS)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing-artifact")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("not theorem prover completeness")));
    }

    #[test]
    fn proof_ref_rejects_stale_consumer_source_identity_and_missing_anchor() {
        let mut reference = matching_reference();
        let mut source = source_artifact_for(&mut reference);
        reference.consumer_source_blake3 = "stale-digest".to_owned();
        source.text = "pub fn codec() {}\n".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, Some(&source)).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_BLAKE3)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("anchor not found")));
    }

    #[test]
    fn proof_ref_rejects_partial_consumer_source_fields() {
        let mut reference = matching_reference();
        reference.consumer_source_path = "src/codec.rs".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_BLAKE3)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_ANCHOR)));
    }

    #[test]
    fn proof_ref_rejects_overbroad_claim_wording() {
        let mut reference = matching_reference();
        reference.consumer_context = "allow linked proof evidence at src/codec.rs:42".to_owned();
        reference.consumer_source_anchor = "octet-proof-ref claims this proves whole-program correctness".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program correctness")));
    }

    #[test]
    fn proof_ref_rejects_invalid_shape_before_linkage() {
        let mut reference = matching_reference();
        reference.schema_version = "wrong".to_owned();
        reference.non_claims.clear();
        let mut deck = deck_with(card());
        deck.schema_version = "wrong".to_owned();

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(PROOF_EVIDENCE_REF_SCHEMA_VERSION)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(PROOF_CARDS_SCHEMA_VERSION)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_NON_CLAIMS)));
    }

    #[test]
    fn proof_ref_accepts_negated_verified_runtime_non_claim() {
        let mut reference = matching_reference();
        reference.non_claims.push("not verified runtime".to_owned());
        let mut card = card();
        card.non_claims.push("not verified runtime".to_owned());
        let deck = deck_with(card);

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn proof_ref_rejects_overclaims_with_other_verbs_and_hyphenation() {
        for (wording, claim) in [
            ("this guarantees whole-program correctness", "whole-program correctness"),
            ("this certifies the codec", "certification"),
            ("this establishes verified-runtime behavior", "verified runtime"),
        ] {
            let mut reference = matching_reference();
            reference.consumer_context = format!("allow linked proof evidence at src/codec.rs:42; {wording}");
            let deck = deck_with(card());

            let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(&format!("unsupported {claim} claim"))),
                "`{wording}` should be rejected as a {claim} claim: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn proof_ref_consumer_source_unmatched_when_linkage_check_skipped() {
        let mut reference = matching_reference();
        let source = source_artifact_for(&mut reference);
        reference.consumer_source_blake3 = "stale-digest".to_owned();
        reference.consumer_context =
            "allow linked proof evidence at src/codec.rs:42; guarantees full safety".to_owned();
        let deck = deck_with(card());

        let validation = validate_proof_evidence_ref(&reference, &deck, Some(&source));

        assert!(!validation.diagnostics.is_empty());
        assert!(!validation.consumer_source_matched);
    }

    #[test]
    fn proof_ref_rejects_external_formal_proof_card_and_card_diagnostics() {
        let reference = matching_reference();
        for (field, value, expected) in [
            (
                "evidence_origin",
                serde_json::json!(crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS),
                "promotes imported Trellis evidence to formal_proof",
            ),
            ("diagnostics", serde_json::json!(["proof receipt is stale"]), "has diagnostics"),
        ] {
            let mut card_json = serde_json::to_value(card()).expect("card json");
            card_json[field] = value;
            let deck = serde_json::from_value::<ProofCardDeck>(serde_json::json!({
                "schema_version": PROOF_CARDS_SCHEMA_VERSION,
                "cards": [card_json],
            }))
            .expect("deck from json");

            let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "card `{field}` should be rejected with `{expected}`: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn proof_ref_scaffold_accepts_negated_aeneas_card_non_claims() {
        let mut anchor_reference = matching_reference();
        let source = source_artifact_for(&mut anchor_reference);
        let mut card = card();
        card.non_claims.extend(aeneas_non_claims());
        let deck = deck_with(card);

        let scaffolded = scaffold_proof_evidence_ref(
            &deck,
            PROPERTY_ID,
            &anchor_reference.reference_id,
            &anchor_reference.consumer_context,
            &source,
            &anchor_reference.consumer_source_anchor,
        )
        .expect("negated Aeneas non-claims must scaffold");

        for non_claim in aeneas_non_claims() {
            assert!(scaffolded.non_claims.contains(&non_claim), "scaffold dropped `{non_claim}`");
        }
    }

    #[test]
    fn proof_ref_negation_is_decided_per_occurrence() {
        let accepted = aeneas_non_claims().join("; ");
        let reference = reference_with_context(&format!("allow linked proof evidence at src/codec.rs:42; {accepted}"));
        let diagnostics = validate_proof_evidence_ref(&reference, &deck_with(card()), None).diagnostics;
        assert!(diagnostics.is_empty(), "negated Aeneas non-claims rejected: {diagnostics:?}");

        for (wording, claim) in [
            ("guarantees full safety of decode; not full safety of encode", "full safety"),
            (
                "does not prove whole-program correctness of encode; proves whole-program correctness of decode",
                "whole-program correctness",
            ),
            (
                "does not establish semantic equivalence or release eligibility, but guarantees semantic equivalence of decode",
                "semantic equivalence",
            ),
            ("does not cover encode, and guarantees full safety of decode", "full safety"),
        ] {
            let reference =
                reference_with_context(&format!("allow linked proof evidence at src/codec.rs:42; {wording}"));

            let diagnostics = validate_proof_evidence_ref(&reference, &deck_with(card()), None).diagnostics;

            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(&format!("unsupported {claim} claim"))),
                "`{wording}` should be rejected as a {claim} claim: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn proof_ref_rejects_duplicate_proof_card_ids() {
        let mut anchor_reference = matching_reference();
        let source = source_artifact_for(&mut anchor_reference);
        let mut shadow = card();
        shadow.checked_rust_source_scope = "formal-proof:production:shadowed-scope".to_owned();
        let duplicate_deck = ProofCardDeck {
            schema_version: PROOF_CARDS_SCHEMA_VERSION.to_owned(),
            cards: vec![card(), shadow],
        };

        let unique = validate_proof_evidence_ref(&anchor_reference, &deck_with(card()), Some(&source)).diagnostics;
        let duplicate = validate_proof_evidence_ref(&anchor_reference, &duplicate_deck, Some(&source)).diagnostics;
        let scaffold = scaffold_proof_evidence_ref(
            &duplicate_deck,
            PROPERTY_ID,
            &anchor_reference.reference_id,
            &anchor_reference.consumer_context,
            &source,
            &anchor_reference.consumer_source_anchor,
        )
        .expect_err("duplicate proof card IDs must not scaffold");

        assert!(unique.is_empty(), "unique deck rejected: {unique:?}");
        assert!(
            duplicate
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate") && diagnostic.contains(PROPERTY_ID))
        );
        assert!(
            scaffold
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate") && diagnostic.contains(PROPERTY_ID))
        );
        assert!(matching_card(&anchor_reference, &duplicate_deck).is_none());
    }

    #[test]
    fn proof_ref_rejects_missing_proof_card_property_id() {
        let reference = matching_reference();
        let mut blank = card();
        blank.proof_or_property_id.clear();
        let deck = ProofCardDeck {
            schema_version: PROOF_CARDS_SCHEMA_VERSION.to_owned(),
            cards: vec![card(), blank],
        };

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None).diagnostics;

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PROOF_OR_PROPERTY_ID)),
            "missing proof card ID should fail validation: {diagnostics:?}"
        );
    }

    #[test]
    fn proof_ref_scaffold_refuses_output_that_aliases_an_input() {
        let temp = tempfile::tempdir().expect("tempdir");
        let deck_path = temp.path().join("proof-cards.json");
        let deck_text = serde_json::to_string_pretty(&deck_with(card())).expect("deck json");
        fs::write(&deck_path, &deck_text).expect("write deck");
        let source_path = temp.path().join("src/codec.rs");
        fs::create_dir_all(source_path.parent().expect("source parent")).expect("create source parent");
        let anchor = format!("octet-proof-ref: {} -> {PROPERTY_ID}", matching_reference().reference_id);
        let source_text = format!("// {anchor}\npub fn codec() {{}}\n");
        fs::write(&source_path, &source_text).expect("write source");
        let scaffold_args = |output: PathBuf| ProofRefArgs {
            mode: ProofRefMode::Scaffold(ProofRefScaffoldArgs {
                proof_cards: deck_path.clone(),
                proof_or_property_id: PROPERTY_ID.to_owned(),
                reference_id: "unchecked-narrowing-range-proof".to_owned(),
                consumer_context: "allow tigerstyle::unchecked_narrowing at src/codec.rs:42".to_owned(),
                consumer_source: source_path.clone(),
                anchor: anchor.clone(),
                output: Some(output),
            }),
        };

        for aliased in [
            deck_path.clone(),
            temp.path().join("src/../proof-cards.json"),
            source_path.clone(),
        ] {
            let error = run(&scaffold_args(aliased.clone())).expect_err("aliased output must be refused");
            assert!(error.contains("--output"), "unexpected error for `{}`: {error}", aliased.display());
        }
        let fresh = temp.path().join("src/new-proof-ref.json");
        let code = run(&scaffold_args(fresh.clone())).expect("fresh output scaffolds");

        assert_eq!(fs::read_to_string(&deck_path).expect("read deck"), deck_text);
        assert_eq!(fs::read_to_string(&source_path).expect("read source"), source_text);
        assert_eq!(code, EXIT_VALID);
        assert!(fresh.is_file());
    }

    fn aeneas_non_claims() -> Vec<String> {
        [
            octet_standards::formal_proof::AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS,
            octet_standards::formal_proof::AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS,
            octet_standards::formal_proof::AENEAS_NON_CLAIM_WHOLE_PROGRAM,
            crate::prove::AENEAS_NON_CLAIM_SEMANTIC_EQUIVALENCE,
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }

    fn reference_with_context(consumer_context: &str) -> ProofEvidenceRef {
        let mut reference = matching_reference();
        consumer_context.clone_into(&mut reference.consumer_context);
        reference
    }

    fn matching_reference() -> ProofEvidenceRef {
        ProofEvidenceRef {
            schema_version: PROOF_EVIDENCE_REF_SCHEMA_VERSION.to_owned(),
            reference_id: "unchecked-narrowing-range-proof".to_owned(),
            consumer_context: "allow tigerstyle::unchecked_narrowing at src/codec.rs:42".to_owned(),
            consumer_source_path: String::new(),
            consumer_source_blake3: String::new(),
            consumer_source_anchor: String::new(),
            proof_or_property_id: PROPERTY_ID.to_owned(),
            source_requirement_or_policy_id: POLICY_ID.to_owned(),
            checked_rust_source_scope: CHECKED_SCOPE.to_owned(),
            expected_evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
            required_status: STATUS_PASSED.to_owned(),
            required_artifacts: vec![RECEIPT_ARTIFACT.to_owned(), "verus/proofs/addition_bound.rs".to_owned()],
            non_claims: required_non_claims().iter().map(|claim| (*claim).to_owned()).collect(),
        }
    }

    fn source_artifact_for(reference: &mut ProofEvidenceRef) -> ConsumerSourceArtifact {
        reference.consumer_source_path = "src/codec.rs".to_owned();
        reference.consumer_source_anchor =
            format!("octet-proof-ref: {} -> {}", reference.reference_id, reference.proof_or_property_id);
        let text = format!("// {}\npub fn codec() {{}}\n", reference.consumer_source_anchor);
        reference.consumer_source_blake3 = blake3::hash(text.as_bytes()).to_hex().to_string();
        ConsumerSourceArtifact {
            path: reference.consumer_source_path.clone(),
            blake3: reference.consumer_source_blake3.clone(),
            text,
        }
    }

    fn deck_with(card: ProofCard) -> ProofCardDeck {
        ProofCardDeck {
            schema_version: PROOF_CARDS_SCHEMA_VERSION.to_owned(),
            cards: vec![card],
        }
    }

    fn card() -> ProofCard {
        ProofCard {
            proof_or_property_id: PROPERTY_ID.to_owned(),
            source_requirement_or_policy_id: POLICY_ID.to_owned(),
            policy_id: POLICY_ID.to_owned(),
            checked_rust_source_scope: CHECKED_SCOPE.to_owned(),
            source_paths: vec!["src/codec.rs".to_owned()],
            evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
            evidence_origin: "octet_rerun".to_owned(),
            status: STATUS_PASSED.to_owned(),
            verifier_mode: "verus".to_owned(),
            verifier_identity_summary: "verus@0.2024".to_owned(),
            toolchain_identity_summary: "rustc 1.83.0".to_owned(),
            artifact_blake3_identities: vec![
                ProofCardArtifactIdentity {
                    artifact: RECEIPT_ARTIFACT.to_owned(),
                    blake3: "receipt-digest".to_owned(),
                },
                ProofCardArtifactIdentity {
                    artifact: "verus/proofs/addition_bound.rs".to_owned(),
                    blake3: "proof-digest".to_owned(),
                },
            ],
            non_claims: required_non_claims().iter().map(|claim| (*claim).to_owned()).collect(),
            diagnostics: Vec::new(),
            negative_check: None,
        }
    }

    fn v1_negative_check(status: &str) -> ProofCardNegativeCheck {
        ProofCardNegativeCheck {
            schema_version: NEGATIVE_CHECK_SCHEMA_VERSION_V1.to_owned(),
            status: status.to_owned(),
            mutant_source: "src/codec_mutant.rs".to_owned(),
            mutant_card_status: STATUS_FAILED.to_owned(),
            mutant_card_evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
            mutant_cards_blake3: "mutant-deck-digest".to_owned(),
            non_claims: vec!["mutant rejection demonstrates discrimination for this mutation only".to_owned()],
            diagnostics: Vec::new(),
            evidence: None,
            measured: None,
        }
    }

    #[test]
    fn proof_ref_treats_v1_demonstrated_negative_check_as_inconclusive() {
        let mut card = card();
        card.negative_check = Some(v1_negative_check(NEGATIVE_STATUS_DEMONSTRATED));
        let deck = deck_with(card);

        let diagnostics = validate_proof_evidence_ref(&matching_reference(), &deck, None).diagnostics;

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("cannot attribute verifier rejection")),
            "v1 demonstrated status must not be trusted: {diagnostics:?}"
        );
    }

    #[test]
    fn proof_ref_accepts_conservative_v1_negative_check() {
        let mut card = card();
        card.negative_check = Some(v1_negative_check(NEGATIVE_STATUS_NOT_DEMONSTRATED));
        let deck = deck_with(card);

        let diagnostics = validate_proof_evidence_ref(&matching_reference(), &deck, None).diagnostics;

        assert!(diagnostics.is_empty(), "conservative v1 record should validate: {diagnostics:?}");
    }

    #[test]
    fn proof_ref_rejects_unsupported_negative_check_schema() {
        let mut card = card();
        let mut negative = v1_negative_check(NEGATIVE_STATUS_DEMONSTRATED);
        negative.schema_version = "octet-proof-negative-check/v9".to_owned();
        card.negative_check = Some(negative);
        let deck = deck_with(card);

        let diagnostics = validate_proof_evidence_ref(&matching_reference(), &deck, None).diagnostics;

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported schema")),
            "unsupported negative-check schema should fail: {diagnostics:?}"
        );
    }
}
