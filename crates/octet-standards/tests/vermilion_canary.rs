//! Recorded-only Vermilion canary validation tests.
// r[verify octet.vermilion_canary_rail.recorded_only]

use octet_standards::vermilion_canary::VermilionCanaryReceipt;
use octet_standards::vermilion_canary::receipt_identity;
use octet_standards::vermilion_canary::validate;

const RECEIPT: &str =
    include_str!("../../../standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/receipt.json");

fn receipt() -> VermilionCanaryReceipt {
    serde_json::from_str(RECEIPT).expect("measured canary receipt")
}

#[test]
fn measured_canary_is_accepted_and_identity_is_exact() {
    let receipt = receipt();
    assert!(validate(&receipt).is_empty(), "{:?}", validate(&receipt));
    assert_eq!(receipt_identity(RECEIPT.as_bytes()), receipt_identity(RECEIPT.as_bytes()));
    assert!(receipt_identity(RECEIPT.as_bytes()).starts_with("b3:"));
}

#[test]
fn source_drift_is_rejected() {
    let mut receipt = receipt();
    receipt.source_blake3 = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();
    assert!(validate(&receipt).iter().any(|diagnostic| diagnostic.contains("source path or BLAKE3")));
}

#[test]
fn toolchain_drift_is_rejected() {
    let mut receipt = receipt();
    receipt.lean_toolchain = "leanprover/lean4:v4.0.0".to_owned();
    assert!(validate(&receipt).iter().any(|diagnostic| diagnostic.contains("toolchain pins")));
}

#[test]
fn outcome_drift_is_rejected() {
    let mut receipt = receipt();
    receipt.outcome = "failed".to_owned();
    assert!(validate(&receipt).iter().any(|diagnostic| diagnostic.contains("outcome or obligation count")));
}

#[test]
fn missing_non_claim_is_rejected() {
    let mut receipt = receipt();
    receipt
        .non_claims
        .retain(|claim| !claim.contains("does not prove the generated obligations refine"));
    assert!(validate(&receipt).iter().any(|diagnostic| diagnostic.contains("missing non-claim")));
}
