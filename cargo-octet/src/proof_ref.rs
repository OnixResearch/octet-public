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
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const NON_CLAIM_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_CERTIFICATION: &str = "not certification";
const NON_CLAIM_BUSINESS_DOMAIN: &str = "not business-domain validation";
const NON_CLAIM_SEMANTIC_EQUIVALENCE: &str = "not semantic equivalence";
const NON_CLAIM_FULL_SAFETY: &str = "not full safety";

#[derive(Debug, Deserialize, Serialize)]
pub struct ProofEvidenceRef {
    pub schema_version: String,
    pub reference_id: String,
    pub consumer_context: String,
    #[serde(default)]
    pub consumer_source_path: String,
    #[serde(default)]
    pub consumer_source_blake3: String,
    #[serde(default)]
    pub consumer_source_anchor: String,
    pub proof_or_property_id: String,
    pub source_requirement_or_policy_id: String,
    pub checked_rust_source_scope: String,
    pub expected_evidence_tier: String,
    pub required_status: String,
    pub required_artifacts: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ProofCardDeck {
    schema_version: String,
    cards: Vec<ProofCard>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCardArtifactIdentity {
    artifact: String,
    blake3: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCard {
    proof_or_property_id: String,
    source_requirement_or_policy_id: String,
    checked_rust_source_scope: String,
    evidence_tier: String,
    status: String,
    artifact_blake3_identities: Vec<ProofCardArtifactIdentity>,
    non_claims: Vec<String>,
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
    let mut diagnostics = validate_proof_evidence_ref(&reference, &deck, consumer_source_ref);
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
        consumer_source_matched: consumer_source_matches(&reference, consumer_source_ref, &diagnostics),
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
        fs::write(output, text).map_err(|error| format!("write {}: {error}", output.display()))?;
    } else {
        print!("{text}");
    }
    Ok(EXIT_VALID)
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {} as JSON: {error}", path.display()))
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
    let Some(card) = deck.cards.iter().find(|card| card.proof_or_property_id == proof_or_property_id) else {
        diagnostics.push(format!(
            "proof evidence ref field `{FIELD_PROOF_OR_PROPERTY_ID}` references missing proof card `{proof_or_property_id}`"
        ));
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
    diagnostics.extend(validate_proof_evidence_ref(&reference, deck, Some(consumer_source)));
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
) -> Vec<String> {
    let mut diagnostics = proof_ref_shape_diagnostics(reference);
    diagnostics.extend(proof_card_deck_shape_diagnostics(deck));
    diagnostics.extend(proof_ref_overclaim_diagnostics(reference));
    if diagnostics.is_empty() {
        diagnostics.extend(proof_ref_linkage_diagnostics(reference, deck));
        diagnostics.extend(consumer_source_linkage_diagnostics(reference, consumer_source));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
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
    if deck.schema_version == PROOF_CARDS_SCHEMA_VERSION {
        Vec::new()
    } else {
        vec![format!(
            "proof card deck field `{FIELD_SCHEMA_VERSION}` expected `{PROOF_CARDS_SCHEMA_VERSION}` but got `{}`",
            deck.schema_version
        )]
    }
}

fn proof_ref_overclaim_diagnostics(reference: &ProofEvidenceRef) -> Vec<String> {
    let text = format!(
        "{} {} {}",
        reference.consumer_context,
        reference.consumer_source_anchor,
        reference.non_claims.join(" ")
    )
    .to_lowercase();
    let mut diagnostics = Vec::new();
    push_overclaim(&mut diagnostics, &text, "proves whole-program correctness", "whole-program correctness");
    push_overclaim(&mut diagnostics, &text, "provides certification", "certification");
    push_overclaim(&mut diagnostics, &text, "provides business-domain validation", "business-domain validation");
    push_overclaim(&mut diagnostics, &text, "proves semantic equivalence", "semantic equivalence");
    push_overclaim(&mut diagnostics, &text, "guarantees full safety", "full safety");
    push_overclaim(&mut diagnostics, &text, "verified runtime", "verified runtime");
    diagnostics
}

fn proof_ref_linkage_diagnostics(reference: &ProofEvidenceRef, deck: &ProofCardDeck) -> Vec<String> {
    let Some(card) = matching_card(reference, deck) else {
        return vec![format!(
            "proof evidence ref field `{FIELD_PROOF_OR_PROPERTY_ID}` references missing proof card `{}`",
            reference.proof_or_property_id
        )];
    };
    let mut diagnostics = Vec::new();
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

fn consumer_source_matches(
    reference: &ProofEvidenceRef,
    consumer_source: Option<&ConsumerSourceArtifact>,
    diagnostics: &[String],
) -> bool {
    !reference.consumer_source_path.trim().is_empty()
        && consumer_source.is_some()
        && !diagnostics.iter().any(|diagnostic| diagnostic.contains("consumer_source"))
}

fn matching_card<'a>(reference: &ProofEvidenceRef, deck: &'a ProofCardDeck) -> Option<&'a ProofCard> {
    deck.cards.iter().find(|card| card.proof_or_property_id == reference.proof_or_property_id)
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

fn push_overclaim(diagnostics: &mut Vec<String>, text: &str, needle: &str, claim: &str) {
    if text.contains(needle) {
        diagnostics.push(format!("proof evidence ref contains unsupported {claim} claim"));
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

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn proof_ref_accepts_matching_consumer_source_anchor() {
        let mut reference = matching_reference();
        let source = source_artifact_for(&mut reference);
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, Some(&source));

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert!(consumer_source_matches(&reference, Some(&source), &diagnostics));
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
        let diagnostics = validate_proof_evidence_ref(&scaffolded, &deck, Some(&source));

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

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

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

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

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

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, Some(&source));

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_BLAKE3)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("anchor not found")));
    }

    #[test]
    fn proof_ref_rejects_partial_consumer_source_fields() {
        let mut reference = matching_reference();
        reference.consumer_source_path = "src/codec.rs".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_BLAKE3)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CONSUMER_SOURCE_ANCHOR)));
    }

    #[test]
    fn proof_ref_rejects_overbroad_claim_wording() {
        let mut reference = matching_reference();
        reference.consumer_context = "allow linked proof evidence at src/codec.rs:42".to_owned();
        reference.consumer_source_anchor = "octet-proof-ref claims this proves whole-program correctness".to_owned();
        let deck = deck_with(card());

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program correctness")));
    }

    #[test]
    fn proof_ref_rejects_invalid_shape_before_linkage() {
        let mut reference = matching_reference();
        reference.schema_version = "wrong".to_owned();
        reference.non_claims.clear();
        let mut deck = deck_with(card());
        deck.schema_version = "wrong".to_owned();

        let diagnostics = validate_proof_evidence_ref(&reference, &deck, None);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(PROOF_EVIDENCE_REF_SCHEMA_VERSION)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(PROOF_CARDS_SCHEMA_VERSION)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_NON_CLAIMS)));
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
            checked_rust_source_scope: CHECKED_SCOPE.to_owned(),
            evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
            status: STATUS_PASSED.to_owned(),
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
        }
    }
}
