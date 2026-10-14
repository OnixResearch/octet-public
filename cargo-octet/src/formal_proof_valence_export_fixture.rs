const FORMAL_PROOF_TIER: &str = "formal_proof";
const MANUAL_REVIEW_TIER: &str = "manual_review_required";
const STATIC_LINT_TIER: &str = "static_lint";
const PASSED_STATUS: &str = "passed";
const REQUIRED_NON_CLAIM: &str =
    "formal-proof Valence export is adapter input only; Octet proof receipts remain authoritative";
const DIAGNOSTIC_MISSING_RECEIPT: &str = "formal_proof_export_missing_receipt";
const DIAGNOSTIC_FAILED_RECEIPT: &str = "formal_proof_export_failed_receipt";
const DIAGNOSTIC_STALE_PROOF_CARD: &str = "formal_proof_export_stale_proof_card";
const DIAGNOSTIC_SKIPPED_VERIFY: &str = "formal_proof_export_skipped_artifact_verify";
const DIAGNOSTIC_TIER_PROMOTION: &str = "formal_proof_export_tier_promotion";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "formal_proof_export_missing_non_claim";
const DIAGNOSTIC_OVERCLAIM: &str = "formal_proof_export_overclaim";

// r[impl octet.evidence_provenance.formal_proof_valence_export_fixture.contract]
// r[impl octet.evidence_provenance.formal_proof_valence_export_fixture.contract.fields]
#[derive(Clone, Debug, Eq, PartialEq)]
struct FormalProofExportFixture {
    proof_receipt_id: String,
    proof_receipt_status: String,
    proof_receipt_tier: String,
    proof_card_receipt_id: String,
    proof_card_verifier_identity: String,
    proof_card_digest: String,
    observed_proof_card_digest: String,
    provenance_record_tier: String,
    valence_export_tier: String,
    artifact_verification_passed: bool,
    imported_trellis_reference: bool,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct FormalProofExportReport {
    valid: bool,
    diagnostics: Vec<String>,
    evidence_tier: String,
    supported_claim: String,
}

// r[impl octet.evidence_provenance.formal_proof_valence_export_fixture.validation]
// r[impl octet.evidence_provenance.formal_proof_valence_export_fixture.validation.core]
fn validate_formal_proof_export_fixture(fixture: &FormalProofExportFixture) -> FormalProofExportReport {
    let mut diagnostics = Vec::new();
    if fixture.proof_receipt_id.trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_RECEIPT.to_owned());
    }
    if fixture.proof_receipt_status != PASSED_STATUS {
        diagnostics.push(DIAGNOSTIC_FAILED_RECEIPT.to_owned());
    }
    if fixture.proof_card_receipt_id != fixture.proof_receipt_id
        || fixture.proof_card_verifier_identity.trim().is_empty()
        || fixture.proof_card_digest != fixture.observed_proof_card_digest
    {
        diagnostics.push(DIAGNOSTIC_STALE_PROOF_CARD.to_owned());
    }
    if !fixture.artifact_verification_passed {
        diagnostics.push(DIAGNOSTIC_SKIPPED_VERIFY.to_owned());
    }
    if fixture.imported_trellis_reference && fixture.valence_export_tier == FORMAL_PROOF_TIER {
        diagnostics.push(DIAGNOSTIC_TIER_PROMOTION.to_owned());
    }
    if fixture.proof_receipt_tier != FORMAL_PROOF_TIER
        && (fixture.provenance_record_tier == FORMAL_PROOF_TIER || fixture.valence_export_tier == FORMAL_PROOF_TIER)
    {
        diagnostics.push(DIAGNOSTIC_TIER_PROMOTION.to_owned());
    }
    if fixture.provenance_record_tier != fixture.valence_export_tier {
        diagnostics.push(DIAGNOSTIC_TIER_PROMOTION.to_owned());
    }
    if !fixture.non_claims.iter().any(|non_claim| non_claim == REQUIRED_NON_CLAIM) {
        diagnostics.push(DIAGNOSTIC_MISSING_NON_CLAIM.to_owned());
    }
    let joined = fixture.non_claims.join(" ").to_lowercase();
    if joined.contains("proves whole-program correctness")
        || joined.contains("proves release eligibility")
        || joined.contains("proves verifier soundness")
    {
        diagnostics.push(DIAGNOSTIC_OVERCLAIM.to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    FormalProofExportReport {
        valid: diagnostics.is_empty(),
        diagnostics,
        evidence_tier: fixture.valence_export_tier.clone(),
        supported_claim: REQUIRED_NON_CLAIM.to_owned(),
    }
}

fn render_fixture_report(fixture: &FormalProofExportFixture) -> String {
    let report = validate_formal_proof_export_fixture(fixture);
    format!("valid={} tier={} claim={}", report.valid, report.evidence_tier, report.supported_claim)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECEIPT_ID: &str = "octet.verus_proof_rails.addition-bound-production#rail-0";
    const CARD_DIGEST: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STALE_CARD_DIGEST: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.positive]
    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.positive.verified]
    #[test]
    fn formal_proof_valence_export_fixture_accepts_verified_scoped_receipt_chain() {
        let fixture = valid_fixture();
        let report = validate_formal_proof_export_fixture(&fixture);

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.evidence_tier, FORMAL_PROOF_TIER);
        assert_eq!(report.supported_claim, REQUIRED_NON_CLAIM);
    }

    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.negative]
    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.negative.skipped_verify]
    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.negative.manual_review]
    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.final_validation]
    // r[verify octet.evidence_provenance.formal_proof_valence_export_fixture.final_validation.fixtures]
    #[test]
    fn formal_proof_valence_export_fixture_rejects_missing_stale_unverified_promoted_and_overclaiming_rows() {
        let mut missing_receipt = valid_fixture();
        missing_receipt.proof_receipt_id.clear();
        let mut failed_receipt = valid_fixture();
        failed_receipt.proof_receipt_status = "failed".to_owned();
        let mut stale_card = valid_fixture();
        stale_card.observed_proof_card_digest = STALE_CARD_DIGEST.to_owned();
        let mut skipped_verify = valid_fixture();
        skipped_verify.artifact_verification_passed = false;
        let mut manual_promoted = valid_fixture();
        manual_promoted.proof_receipt_tier = MANUAL_REVIEW_TIER.to_owned();
        let mut nonproof_promoted = valid_fixture();
        nonproof_promoted.proof_receipt_tier = STATIC_LINT_TIER.to_owned();
        let mut trellis_promoted = valid_fixture();
        trellis_promoted.imported_trellis_reference = true;
        let mut missing_non_claim = valid_fixture();
        missing_non_claim.non_claims.clear();
        let mut overclaim = valid_fixture();
        overclaim.non_claims.push("proves whole-program correctness".to_owned());

        let cases = vec![
            (missing_receipt, DIAGNOSTIC_MISSING_RECEIPT),
            (failed_receipt, DIAGNOSTIC_FAILED_RECEIPT),
            (stale_card, DIAGNOSTIC_STALE_PROOF_CARD),
            (skipped_verify, DIAGNOSTIC_SKIPPED_VERIFY),
            (manual_promoted, DIAGNOSTIC_TIER_PROMOTION),
            (nonproof_promoted, DIAGNOSTIC_TIER_PROMOTION),
            (trellis_promoted, DIAGNOSTIC_TIER_PROMOTION),
            (missing_non_claim, DIAGNOSTIC_MISSING_NON_CLAIM),
            (overclaim, DIAGNOSTIC_OVERCLAIM),
        ];

        for (fixture, expected) in cases {
            let report = validate_formal_proof_export_fixture(&fixture);
            assert!(!report.valid, "{expected} should fail");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic == expected),
                "expected {expected} in {:?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn formal_proof_valence_export_fixture_shell_report_is_bounded() {
        let report = render_fixture_report(&valid_fixture());

        assert!(report.contains("valid=true"));
        assert!(report.contains(FORMAL_PROOF_TIER));
        assert!(report.contains("adapter input only"));
    }

    fn valid_fixture() -> FormalProofExportFixture {
        FormalProofExportFixture {
            proof_receipt_id: RECEIPT_ID.to_owned(),
            proof_receipt_status: PASSED_STATUS.to_owned(),
            proof_receipt_tier: FORMAL_PROOF_TIER.to_owned(),
            proof_card_receipt_id: RECEIPT_ID.to_owned(),
            proof_card_verifier_identity: "verus-production-fixture".to_owned(),
            proof_card_digest: CARD_DIGEST.to_owned(),
            observed_proof_card_digest: CARD_DIGEST.to_owned(),
            provenance_record_tier: FORMAL_PROOF_TIER.to_owned(),
            valence_export_tier: FORMAL_PROOF_TIER.to_owned(),
            artifact_verification_passed: true,
            imported_trellis_reference: false,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }
}
