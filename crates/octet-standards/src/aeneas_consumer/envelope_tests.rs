// r[verify octet.aeneas_consumer_contract.artifacts]
// r[verify octet.aeneas_consumer_contract.valence_handoff]
// r[verify octet.aeneas_consumer_contract.negative_evidence]
use super::receipt_identity;
use crate::formal_proof::FormalProofEnvelope;
use crate::formal_proof::validate_formal_proof_envelope;

const CHECKED_RECEIPT: &str = include_str!(
    "../../../../standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/formal-proof-receipt.json"
);
const DIFFERENT_IDENTITY: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const EXPECTED_TIMEOUT_MS: u64 = 900_000;

#[test]
fn checked_consumer_receipt_preserves_exact_stage_resource_and_policy_links() {
    let envelope = checked_envelope();
    let consumer = envelope.consumer.as_ref().expect("checked consumer binding");

    assert_eq!(consumer.resource_policy.timeout_ms, EXPECTED_TIMEOUT_MS);
    assert!(validate_formal_proof_envelope(&envelope).is_empty());
}

#[test]
fn consumer_receipt_tampering_cannot_survive_formal_or_valence_validation() {
    let mut stage = checked_envelope();
    stage.consumer.as_mut().expect("consumer").stage_artifacts[0].blake3 = DIFFERENT_IDENTITY.to_owned();
    refresh_consumer_identity(&mut stage);

    let mut resource = checked_envelope();
    resource.consumer.as_mut().expect("consumer").resource_policy.timeout_ms = 1;
    refresh_consumer_identity(&mut resource);

    let mut role = checked_envelope();
    role.consumer.as_mut().expect("consumer").stage_artifacts[0].role = "invented-stage".to_owned();
    refresh_consumer_identity(&mut role);

    let mut profile = checked_envelope();
    profile.consumer.as_mut().expect("consumer").provider_profile_identity = DIFFERENT_IDENTITY.to_owned();
    refresh_consumer_identity(&mut profile);

    let mut missing = checked_envelope();
    missing.consumer = None;

    let mut wrong_policy = checked_envelope();
    wrong_policy.policy_id = "octet.aeneas_lean_proof_rails.fixture".to_owned();

    for (envelope, expected) in [
        (stage, "provider stage chain"),
        (resource, "resource policy"),
        (role, "not supported by the provider chain"),
        (profile, "provider profile"),
        (missing, "requires its exact consumer binding"),
        (wrong_policy, "requires the selected consumer policy"),
    ] {
        let diagnostics = validate_formal_proof_envelope(&envelope);
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "tamper must fail with `{expected}`: {diagnostics:?}"
        );
    }
}

#[test]
fn consumer_receipt_schema_rejects_unknown_fields() {
    let mut value: serde_json::Value = serde_json::from_str(CHECKED_RECEIPT).expect("checked receipt JSON");
    value["consumer"]["unknown_authority"] = serde_json::Value::Bool(true);

    assert!(serde_json::from_value::<FormalProofEnvelope>(value).is_err());
}

fn checked_envelope() -> FormalProofEnvelope {
    serde_json::from_str(CHECKED_RECEIPT).expect("checked formal-proof receipt")
}

fn refresh_consumer_identity(envelope: &mut FormalProofEnvelope) {
    let consumer = envelope.consumer.as_mut().expect("consumer binding");
    consumer.receipt_identity = receipt_identity(consumer).expect("consumer receipt identity");
}
