const PRESERVES_SCHEMA_LABEL: &str = "preserves.receipt-envelope/v1";
const REQUIRED_NON_CLAIM: &str =
    "Preserves export is an adapter over verified Octet artifacts; Octet source receipts remain authoritative";
const BLAKE3_PREFIX: &str = "b3:";
const DIAGNOSTIC_MISSING_SOURCE: &str = "preserves_missing_source_receipt";
const DIAGNOSTIC_STALE_DIGEST: &str = "preserves_stale_source_digest";
const DIAGNOSTIC_SKIPPED_VERIFY: &str = "preserves_skipped_artifact_verification";
const DIAGNOSTIC_ROLE_PROMOTION: &str = "preserves_role_promotion";
const DIAGNOSTIC_UNSUPPORTED_SCHEMA: &str = "preserves_unsupported_schema";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "preserves_missing_non_claim";
const DIAGNOSTIC_OVERCLAIM: &str = "preserves_overclaim";
const DIAGNOSTIC_MISSING_CONTEXT: &str = "preserves_missing_context";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceRole {
    AdapterInput,
    ReleaseEligibility,
    FormalProof,
}

// r[impl octet.evidence_provenance.preserves_receipt_export.contract]
#[derive(Clone, Debug, Eq, PartialEq)]
struct PreservesExportRow {
    source_artifact_dir: String,
    source_receipt_digest: String,
    observed_source_receipt_digest: String,
    artifact_verification_passed: bool,
    rail: String,
    schema_label: String,
    preserves_envelope_digest: String,
    measured_envelope_digest: String,
    evidence_role: EvidenceRole,
    consumer_profile: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreservesExportReport {
    valid: bool,
    diagnostics: Vec<String>,
    source_receipt_digest: String,
    supported_claim: String,
}

// r[impl octet.evidence_provenance.preserves_receipt_export.validation]
fn validate_preserves_export(row: &PreservesExportRow) -> PreservesExportReport {
    let mut diagnostics = Vec::new();
    if row.source_artifact_dir.trim().is_empty() || row.source_receipt_digest.trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_SOURCE.to_owned());
    }
    if row.rail.trim().is_empty() || row.consumer_profile.trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_CONTEXT.to_owned());
    }
    if row.source_receipt_digest != row.observed_source_receipt_digest {
        diagnostics.push(DIAGNOSTIC_STALE_DIGEST.to_owned());
    }
    if !row.artifact_verification_passed {
        diagnostics.push(DIAGNOSTIC_SKIPPED_VERIFY.to_owned());
    }
    if row.schema_label != PRESERVES_SCHEMA_LABEL
        || !row.preserves_envelope_digest.starts_with(BLAKE3_PREFIX)
        || row.preserves_envelope_digest != row.measured_envelope_digest
    {
        diagnostics.push(DIAGNOSTIC_UNSUPPORTED_SCHEMA.to_owned());
    }
    if row.evidence_role != EvidenceRole::AdapterInput {
        diagnostics.push(DIAGNOSTIC_ROLE_PROMOTION.to_owned());
    }
    if !row.non_claims.iter().any(|non_claim| non_claim == REQUIRED_NON_CLAIM) {
        diagnostics.push(DIAGNOSTIC_MISSING_NON_CLAIM.to_owned());
    }
    let joined = row.non_claims.join(" ").to_lowercase();
    if joined.contains("proves release eligibility")
        || joined.contains("proves formal proof")
        || joined.contains("preserves is authoritative")
    {
        diagnostics.push(DIAGNOSTIC_OVERCLAIM.to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    PreservesExportReport {
        valid: diagnostics.is_empty(),
        diagnostics,
        source_receipt_digest: row.source_receipt_digest.clone(),
        supported_claim: REQUIRED_NON_CLAIM.to_owned(),
    }
}

fn measure_preserves_envelope(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_DIGEST: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STALE_DIGEST: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const ENVELOPE_BYTES: &[u8] = b"canonical preserves envelope fixture";

    // r[verify octet.evidence_provenance.preserves_receipt_export.fixtures.positive]
    #[test]
    fn preserves_receipt_export_accepts_verified_octet_receipt_envelope() {
        let row = valid_row();
        let report = validate_preserves_export(&row);

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.source_receipt_digest, SOURCE_DIGEST);
        assert_eq!(report.supported_claim, REQUIRED_NON_CLAIM);
    }

    // r[verify octet.evidence_provenance.preserves_receipt_export.fixtures.negative]
    // r[verify octet.evidence_provenance.preserves_receipt_export.final_validation]
    // r[verify octet.evidence_provenance.preserves_receipt_export.final_validation.fixtures]
    #[test]
    fn preserves_receipt_export_rejects_missing_stale_unverified_promoted_and_overclaiming_rows() {
        let mut missing = valid_row();
        missing.source_receipt_digest.clear();
        let mut stale = valid_row();
        stale.observed_source_receipt_digest = STALE_DIGEST.to_owned();
        let mut skipped_verify = valid_row();
        skipped_verify.artifact_verification_passed = false;
        let mut promoted_release = valid_row();
        promoted_release.evidence_role = EvidenceRole::ReleaseEligibility;
        let mut promoted_proof = valid_row();
        promoted_proof.evidence_role = EvidenceRole::FormalProof;
        let mut unsupported_schema = valid_row();
        unsupported_schema.schema_label = "preserves.unsupported/v9".to_owned();
        let mut missing_non_claim = valid_row();
        missing_non_claim.non_claims.clear();
        let mut overclaim = valid_row();
        overclaim.non_claims.push("Preserves export proves release eligibility".to_owned());

        let cases = vec![
            (missing, DIAGNOSTIC_MISSING_SOURCE),
            (stale, DIAGNOSTIC_STALE_DIGEST),
            (skipped_verify, DIAGNOSTIC_SKIPPED_VERIFY),
            (promoted_release, DIAGNOSTIC_ROLE_PROMOTION),
            (promoted_proof, DIAGNOSTIC_ROLE_PROMOTION),
            (unsupported_schema, DIAGNOSTIC_UNSUPPORTED_SCHEMA),
            (missing_non_claim, DIAGNOSTIC_MISSING_NON_CLAIM),
            (overclaim, DIAGNOSTIC_OVERCLAIM),
        ];

        for (row, expected) in cases {
            let report = validate_preserves_export(&row);
            assert!(!report.valid, "{expected} should fail");
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic == expected),
                "expected {expected} in {:?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn preserves_receipt_export_shell_measures_canonical_envelope_bytes() {
        let measured = measure_preserves_envelope(ENVELOPE_BYTES);
        let mut row = valid_row();
        row.preserves_envelope_digest = measured.clone();
        row.measured_envelope_digest = measured.clone();

        let report = validate_preserves_export(&row);

        assert!(measured.starts_with(BLAKE3_PREFIX));
        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
    }

    fn valid_row() -> PreservesExportRow {
        let measured = measure_preserves_envelope(ENVELOPE_BYTES);
        PreservesExportRow {
            source_artifact_dir: "artifacts/octet".to_owned(),
            source_receipt_digest: SOURCE_DIGEST.to_owned(),
            observed_source_receipt_digest: SOURCE_DIGEST.to_owned(),
            artifact_verification_passed: true,
            rail: "octet-artifact-verify".to_owned(),
            schema_label: PRESERVES_SCHEMA_LABEL.to_owned(),
            preserves_envelope_digest: measured.clone(),
            measured_envelope_digest: measured,
            evidence_role: EvidenceRole::AdapterInput,
            consumer_profile: "valence-preserves-adapter-fixture".to_owned(),
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }
}
