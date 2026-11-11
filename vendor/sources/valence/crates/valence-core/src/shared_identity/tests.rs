use serde_json::json;

use super::*;

const DIGEST_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const OTHER_DIGEST_HEX: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

fn required_non_claims() -> Vec<String> {
    vec![SHARED_IDENTITY_REQUIRED_NON_CLAIM.to_owned()]
}

#[test]
fn supported_digest_spellings_normalize_by_profile() {
    // r[verify valence.evidence_ir.shared_identity.fixtures.positive]
    let candidate = SharedDigestCandidate {
        role: DigestRole::ArtifactIdentity,
        expected_role: Some(DigestRole::ArtifactIdentity),
        digest: format!("{B3_PREFIX}{DIGEST_HEX}"),
        profile: STRICT_B3_PROFILE,
        non_claims: required_non_claims(),
    };

    let report = validate_shared_digest(&candidate);

    assert!(report.valid);
    let digest = report.digest.expect("valid digest should be present");
    assert_eq!(digest.normalized_hex, DIGEST_HEX);
    assert_eq!(digest.role, DigestRole::ArtifactIdentity);
    assert!(report.supported_claim.contains("canonical content identity"));
}

#[test]
fn malformed_or_unsupported_digest_fails_closed() {
    // r[verify valence.evidence_ir.shared_identity.fixtures.negative]
    let candidate = SharedDigestCandidate {
        role: DigestRole::ArtifactIdentity,
        expected_role: None,
        digest: OTHER_DIGEST_HEX.to_owned(),
        profile: STRICT_B3_PROFILE,
        non_claims: required_non_claims(),
    };

    let report = validate_shared_digest(&candidate);

    assert!(!report.valid);
    assert!(report.issues.iter().any(|issue| issue.code == "unsupported_digest_spelling"));
}

#[test]
fn digest_roles_are_not_interchangeable() {
    // r[verify valence.evidence_ir.shared_identity.digest_roles]
    let candidate = SharedDigestCandidate {
        role: DigestRole::BinaryBytes,
        expected_role: Some(DigestRole::SidecarBytes),
        digest: format!("{B3_PREFIX}{DIGEST_HEX}"),
        profile: STRICT_B3_PROFILE,
        non_claims: required_non_claims(),
    };

    let report = validate_shared_digest(&candidate);

    assert!(!report.valid);
    assert!(report.issues.iter().any(|issue| issue.code == "role_mismatch"));
}

#[test]
fn canonical_json_hash_is_order_stable() {
    // r[verify valence.evidence_ir.shared_identity.fixtures.positive]
    let left = json!({"b": [2, 1], "a": {"z": true, "m": false}});
    let right = json!({"a": {"m": false, "z": true}, "b": [2, 1]});

    let left_hash = canonical_json_hash(&left).expect("JSON Value serialization should succeed");
    let right_hash = canonical_json_hash(&right).expect("JSON Value serialization should succeed");

    assert_eq!(left_hash, right_hash);
}

#[test]
fn receipt_hash_exclusion_removes_self_reference_only() {
    // r[verify valence.evidence_ir.shared_identity.self_hash_exclusion.receipt]
    let with_self_hash = json!({
        "receipt_hash": "stale",
        "preserves_content_hash": "stale",
        "payload": {"stable": true}
    });
    let without_self_hash = json!({"payload": {"stable": true}});

    let actual = canonical_json_hash_with_exclusion(&with_self_hash, Some(RECEIPT_SELF_HASH_EXCLUSION))
        .expect("known exclusion profile should hash");
    let expected = canonical_json_hash(&without_self_hash).expect("JSON Value serialization should succeed");

    assert_eq!(actual, expected);
}

#[test]
fn missing_exclusion_profile_fails_closed() {
    // r[verify valence.evidence_ir.shared_identity.self_hash_exclusion.unknown]
    let value = json!({"receipt_hash": "stale"});

    let issue = canonical_json_hash_with_exclusion(&value, None).expect_err("missing profile should fail");

    assert_eq!(issue.field, FIELD_SELF_HASH_EXCLUSION);
    assert_eq!(issue.code, "missing_self_hash_exclusion_profile");
}

#[test]
fn non_claim_boundary_rejects_overclaim() {
    // r[verify valence.evidence_ir.shared_identity.final_validation]
    let candidate = SharedDigestCandidate {
        role: DigestRole::ReceiptIdentity,
        expected_role: None,
        digest: DIGEST_HEX.to_owned(),
        profile: RAW_BLAKE3_PROFILE,
        non_claims: vec!["proves behavioral correctness".to_owned()],
    };

    let report = validate_shared_digest(&candidate);

    assert!(!report.valid);
    assert!(report.issues.iter().any(|issue| issue.code == "non_claims_weakened"));
    assert!(report.issues.iter().any(|issue| issue.code == "overclaim_detected"));
}

#[test]
fn canonical_profile_names_are_visible() {
    assert_eq!(CanonicalHashProfile::JsonValue.as_str(), "json_value");
    assert_eq!(CanonicalHashProfile::PreservesText.as_str(), "preserves_text");
}
