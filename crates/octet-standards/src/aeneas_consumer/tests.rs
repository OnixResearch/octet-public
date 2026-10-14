// r[verify octet.aeneas_consumer_contract.negative_evidence]
// r[verify octet.aeneas_consumer_contract.final_validation]
use super::*;
use crate::aeneas_consumer::model::ConsumerOutcome;

const SOURCE: &str = "pub fn increment(value: u32) -> u32 {\n    value + 1\n}\n";
const THEOREM: &str = "import Llbc\n\n/-- Octet-owned compatibility theorem for an external consumer profile. -/\ntheorem increment_matches_model : external_consumer.increment 1#u32 = .ok 2#u32 := by\n  rfl\n";
const GENERATED: &str = "def external_consumer.increment := fun value => value + 1\n";
const PROVIDER_LLBC: &str =
    include_str!("../../../../standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/llbc.llbc");
const PROVIDER_TRANSLATION: &str =
    include_str!("../../../../standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/translation.json");
const PROVIDER_GENERATED: &str =
    include_str!("../../../../standards/proof-rails/aeneas-lean/fixtures/external-consumer/expected/Llbc.lean");
const DIAGNOSTIC: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn translation() -> String {
    serde_json::json!({
        "functions": [{
            "def_id": 7,
            "lean_name": "external_consumer.increment",
            "rust_name": "external_consumer::increment",
            "is_local": true,
            "source": {"file": "src/lib.rs", "begin_line": 1, "end_line": 3}
        }]
    })
    .to_string()
}

fn llbc() -> String {
    serde_json::json!({
        "has_errors": false,
        "translated": {
            "files": [{"name": {"Local": "src/lib.rs"}, "contents": SOURCE}],
            "fun_decls": [{
                "def_id": 99,
                "item_meta": {
                    "source_text": SOURCE,
                    "span": {"data": {"beg": {"line": 1}, "end": {"line": 3}}}
                }
            }]
        }
    })
    .to_string()
}

fn input<'a>(llbc: &'a str, translation: &'a str) -> ObservationInput<'a> {
    ObservationInput {
        source_path: "src/lib.rs",
        source_text: SOURCE,
        selected_item: "external_consumer::increment",
        theorem_path: "Theorem.lean",
        theorem_text: THEOREM,
        llbc_text: llbc,
        translation_text: translation,
        generated_lean_text: GENERATED,
    }
}

fn succeeded(stage: &str) -> StageObservation {
    StageObservation {
        stage: stage.to_owned(),
        status: StageStatus::Succeeded,
        construct: None,
        diagnostic_identity: None,
        next_action: None,
    }
}

#[test]
fn checked_profile_is_exact_and_complete() {
    let profile = selected_profile().expect("checked consumer profile");
    assert_eq!(validate_profile(&profile), Vec::<String>::new());
    assert_eq!(profile.profile_id, EXPECTED_PROFILE_ID);
    assert_eq!(profile.provider.profile_identity, EXPECTED_PROVIDER_IDENTITY);
    assert_eq!(profile.provider.compatibility_matrix_identity, EXPECTED_COMPATIBILITY_IDENTITY);
    assert_eq!(profile.source.selected_items.len(), 1);
    assert_eq!(profile.theorems.len(), 1);
}

#[test]
fn exact_source_translation_and_theorem_build_a_stable_binding() {
    let profile = selected_profile().expect("profile");
    let llbc = llbc();
    let translation = translation();
    let first = build_binding(&profile, PROFILE_IDENTITY, &input(&llbc, &translation), ConsumerOutcome::Supported)
        .expect("valid consumer binding");
    let second = build_binding(&profile, PROFILE_IDENTITY, &input(&llbc, &translation), ConsumerOutcome::Supported)
        .expect("repeat consumer binding");
    assert_eq!(first, second);
    assert_eq!(validate_receipt(&first), Vec::<String>::new());
    assert_eq!(first.stage_artifacts.len(), 5);
    assert!(!first.receipt_identity.is_empty());

    let mut downstream_profile = profile;
    downstream_profile.profile_id = "downstream-exact-profile".to_owned();
    downstream_profile.consumer_id = "downstream-consumer".to_owned();
    let downstream_bytes = serde_json::to_vec_pretty(&downstream_profile).expect("downstream profile JSON");
    let (downstream_profile, downstream_identity) = load_profile(&downstream_bytes).expect("measured profile");
    let downstream = build_binding(
        &downstream_profile,
        &downstream_identity,
        &input(&llbc, &translation),
        ConsumerOutcome::Supported,
    )
    .expect("downstream binding");
    assert_eq!(
        validate_receipt_against(&downstream, &downstream_profile, &downstream_identity),
        Vec::<String>::new()
    );
    assert!(!validate_receipt(&downstream).is_empty());
}

#[test]
fn checked_provider_artifacts_match_the_external_consumer_profile() {
    let profile = selected_profile().expect("checked consumer profile");
    let input = ObservationInput {
        source_path: "src/lib.rs",
        source_text: SOURCE,
        selected_item: "external_consumer::increment",
        theorem_path: "Theorem.lean",
        theorem_text: THEOREM,
        llbc_text: PROVIDER_LLBC,
        translation_text: PROVIDER_TRANSLATION,
        generated_lean_text: PROVIDER_GENERATED,
    };
    assert_eq!(observation::validate(&profile, &input), Vec::<String>::new());
}

#[test]
fn def_id_changes_do_not_change_durable_binding() {
    let profile = selected_profile().expect("profile");
    let llbc_first = llbc();
    let llbc_second = llbc_first.replace("99", "314");
    let translation_first = translation();
    let translation_second = translation_first.replace("\"def_id\":7", "\"def_id\":271");
    assert_eq!(observation::validate(&profile, &input(&llbc_first, &translation_first)), Vec::<String>::new());
    assert_eq!(observation::validate(&profile, &input(&llbc_second, &translation_second)), Vec::<String>::new());
}

#[test]
fn provider_source_item_and_theorem_drift_fail_closed() {
    let mut profile = selected_profile().expect("profile");
    profile.provider.profile_identity = DIAGNOSTIC.to_owned();
    assert!(validate_profile(&profile).iter().any(|item| item.contains("provider")));

    let profile = selected_profile().expect("profile");
    let llbc = llbc();
    let translation = translation();
    let mut stale_source = input(&llbc, &translation);
    stale_source.source_text = "pub fn increment(value: u32) -> u32 { value }\n";
    assert!(observation::validate(&profile, &stale_source).iter().any(|item| item.contains("source")));

    let mut stale_theorem = input(&llbc, &translation);
    stale_theorem.theorem_text = "theorem increment_matches_model : True := by trivial\n";
    assert!(observation::validate(&profile, &stale_theorem).iter().any(|item| item.contains("theorem")));
}

#[test]
fn ambiguous_malformed_stale_and_trust_inputs_fail_closed() {
    let mut profile = selected_profile().expect("profile");
    profile.source.selected_items.push(profile.source.selected_items[0].clone());
    assert!(validate_profile(&profile).iter().any(|item| item.contains("ambiguous")));

    let profile = selected_profile().expect("profile");
    let llbc = llbc();
    let translation = translation();
    let malformed = ObservationInput {
        translation_text: "not-json",
        ..input(&llbc, &translation)
    };
    assert!(observation::validate(&profile, &malformed).iter().any(|item| item.contains("parse")));

    let mut trust = profile.clone();
    trust.trust_inputs.push(crate::aeneas_consumer::model::TrustInput {
        input_id: "ambient".to_owned(),
        identity: "stale".to_owned(),
    });
    assert!(validate_profile(&trust).iter().any(|item| item.contains("trust input")));
}

#[test]
fn supported_blocked_and_failed_outcomes_are_distinct() {
    let supported = classify(&[succeeded("charon"), succeeded("aeneas"), succeeded("lean")]);
    assert_eq!(supported.outcome, ConsumerOutcome::Supported);

    let blocked = classify(&[StageObservation {
        stage: "charon".to_owned(),
        status: StageStatus::Unsupported,
        construct: Some("unsupported-trait-dispatch".to_owned()),
        diagnostic_identity: Some(DIAGNOSTIC.to_owned()),
        next_action: Some("reduce the exact fixture".to_owned()),
    }]);
    assert_eq!(blocked.outcome, ConsumerOutcome::Blocked);
    assert_eq!(blocked.stage.as_deref(), Some("charon"));

    for status in [
        StageStatus::ProcessFailed,
        StageStatus::PolicyFailed,
        StageStatus::ArtifactFailed,
        StageStatus::TimedOut,
        StageStatus::BudgetExceeded,
    ] {
        let failed = classify(&[StageObservation {
            stage: "provider".to_owned(),
            status,
            construct: None,
            diagnostic_identity: None,
            next_action: None,
        }]);
        assert_eq!(failed.outcome, ConsumerOutcome::Failed);
    }
}

#[test]
fn incomplete_blocker_and_receipt_tampering_cannot_promote() {
    let incomplete = classify(&[StageObservation {
        stage: "charon".to_owned(),
        status: StageStatus::Unsupported,
        construct: None,
        diagnostic_identity: None,
        next_action: None,
    }]);
    assert_eq!(incomplete.outcome, ConsumerOutcome::Failed);

    let profile = selected_profile().expect("profile");
    let llbc = llbc();
    let translation = translation();
    let mut receipt =
        build_binding(&profile, PROFILE_IDENTITY, &input(&llbc, &translation), ConsumerOutcome::Supported)
            .expect("valid receipt");
    receipt.profile_identity = DIAGNOSTIC.to_owned();
    receipt.verification_role = "boundary".to_owned();
    receipt.stage_artifacts[0].blake3 = "stale".to_owned();
    assert!(!validate_receipt(&receipt).is_empty());
}

#[test]
fn profile_json_rejects_unknown_fields() {
    let text = include_str!("../../../../standards/proof-rails/aeneas-lean/generated/consumer-profile.json");
    let mut value: serde_json::Value = serde_json::from_str(text).expect("profile JSON");
    value["ambient_provider"] = serde_json::json!("host");
    let error = serde_json::from_value::<ConsumerProfile>(value).expect_err("unknown field must fail");
    assert!(error.to_string().contains("unknown field"));
}
