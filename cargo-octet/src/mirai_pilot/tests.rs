// r[verify octet.mirai_semantic_pilot.verification]
use std::path::Path;

use super::*;

const OTHER_PARENT: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SOURCE: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn profile() -> PilotProfile {
    selected_policy().expect("checked MIRAI pilot policy")
}

fn fact(parent: &str, source: &str, value: &str) -> AnalyzerFact {
    AnalyzerFact {
        parent_identity: parent.to_owned(),
        source_identity: source.to_owned(),
        value: value.to_owned(),
    }
}

#[test]
fn checked_policy_is_typed_exact_and_blocked() {
    let profile = profile();
    assert_eq!(profile.profile_id, PROFILE_ID);
    assert_eq!(profile.provider.revision, EXPECTED_REVISION);
    assert_eq!(profile.provider.rust_toolchain, EXPECTED_TOOLCHAIN);
    assert_eq!(profile.output_contract.reason_code, EXPECTED_BLOCKER);
    assert!(!profile.output_contract.stable_machine_schema);
}

#[test]
fn selected_corpus_bytes_match_checked_identities() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace parent");
    assert_eq!(shell::verify_corpus(root, &profile()), Vec::<String>::new());
}

#[test]
fn provider_plan_is_fixed_but_execution_is_blocked_before_spawn() {
    let plan = shell::reviewed_plan(&profile()).expect("reviewed plan");
    assert_eq!(plan.command_argv.iter().map(String::as_str).collect::<Vec<_>>(), EXPECTED_COMMAND);
    assert_eq!(plan.environment.iter().map(String::as_str).collect::<Vec<_>>(), EXPECTED_ENVIRONMENT);
    assert!(plan.clear_environment);
    assert!(!plan.network_allowed);
    let error = shell::execute(&plan).expect_err("blocked output contract cannot execute");
    assert!(error.contains(EXPECTED_BLOCKER));
}

#[test]
fn corpus_keeps_supported_oracles_and_unsupported_case_visible() {
    let profile = profile();
    for class in ["safe", "panic", "precondition", "call-edge", "effect"] {
        assert!(profile.corpus.iter().any(|fixture| fixture.property_class == class && fixture.supported));
    }
    let unsupported = profile
        .corpus
        .iter()
        .find(|fixture| !fixture.supported)
        .expect("unsupported fixture remains reported");
    assert_eq!(unsupported.property_class, "unsupported");
    assert!(!unsupported.unsupported_reason.is_empty());
}

#[test]
fn comparison_classifies_all_declared_outcomes() {
    let fixture = profile().corpus.into_iter().find(|item| item.fixture_id == "safe-path").expect("fixture");
    let source = fixture.source_identity.as_str();
    let mirai = fact(OTHER_PARENT, source, "present");
    let charon = fact(EXPECTED_CHARON_PARENT, source, "present");
    let token = fact(EXPECTED_EFFECT_PARENT, source, "present");
    let disagreeing = fact(EXPECTED_EFFECT_PARENT, source, "absent");
    let rows = [
        comparison::compare(&fixture, Some(&mirai), Some(&charon), Some(&token)),
        comparison::compare(&fixture, Some(&mirai), Some(&charon), Some(&disagreeing)),
        comparison::compare(&fixture, Some(&mirai), None, None),
        comparison::compare(&fixture, None, Some(&charon), None),
        comparison::compare(&fixture, None, None, Some(&token)),
        comparison::compare(&fixture, None, Some(&charon), Some(&token)),
    ];
    assert_eq!(rows.map(|row| row.outcome), [
        "agree",
        "disagree",
        "mirai-only",
        "charon-only",
        "source-token-only",
        "unknown"
    ]);
}

#[test]
fn comparison_source_drift_is_unknown_without_parent_rewrite() {
    let fixture = &profile().corpus[0];
    let drifted = fact(OTHER_PARENT, OTHER_SOURCE, "present");
    let row = comparison::compare(fixture, Some(&drifted), None, None);
    assert_eq!(row.outcome, "unknown");
    assert_eq!(row.parent_identities, [OTHER_PARENT]);
}

#[test]
fn blocked_decision_is_deterministic_and_valid() {
    let profile = profile();
    let first = blocked_receipt(&profile).expect("blocked decision");
    let second = blocked_receipt(&profile).expect("blocked decision");
    assert_eq!(first, second);
    assert_eq!(validate_receipt(&first, &profile), Vec::<String>::new());
    assert_eq!(receipt_bytes(&first).expect("receipt bytes"), receipt_bytes(&second).expect("receipt bytes"));
    println!(
        "MIRAI_DECISION_BEGIN\n{}MIRAI_DECISION_END",
        String::from_utf8(receipt_bytes(&first).expect("JSON")).expect("UTF-8")
    );
}

#[test]
fn checked_decision_artifact_replays_byte_for_byte() {
    let checked = include_str!("../../../standards/mirai/decision.json");
    let parsed = parse_decision(checked).expect("checked decision artifact");
    let generated = blocked_receipt(&profile()).expect("blocked decision");
    assert_eq!(parsed, generated);
    assert_eq!(receipt_bytes(&generated).expect("generated decision bytes"), checked.as_bytes());
}

#[test]
fn malformed_decision_is_rejected() {
    let diagnostics = parse_decision("{not-json").expect_err("malformed decision must fail");
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("parse MIRAI pilot decision")));
}

#[test]
fn provider_and_limit_drift_fail_preflight() {
    let mut revision = profile();
    revision.provider.revision = "drifted".to_owned();
    assert!(shell::reviewed_plan(&revision).expect_err("revision drift").contains("provider identity"));
    let mut timeout = profile();
    timeout.limits.timeout_ms += 1;
    assert!(shell::reviewed_plan(&timeout).expect_err("limit drift").contains("resource limits"));
    let mut source = profile();
    source.corpus[0].source_identity = OTHER_SOURCE.to_owned();
    assert!(shell::reviewed_plan(&source).expect_err("source drift").contains("source identity"));
}

#[test]
fn unmeasured_timeout_cannot_replace_the_exact_blocker() {
    let profile = profile();
    let mut receipt = blocked_receipt(&profile).expect("blocked decision");
    receipt.decision = "defer".to_owned();
    receipt.blocker_code = "provider-timeout".to_owned();
    receipt.execution_attempted = true;
    receipt.receipt_identity.clear();
    assert!(
        validate_receipt(&receipt, &profile)
            .iter()
            .any(|diagnostic| diagnostic.contains("blocked decision"))
    );
}

#[test]
fn missing_comparison_parent_is_rejected() {
    let profile = profile();
    let mut receipt = blocked_receipt(&profile).expect("blocked decision");
    receipt.comparisons[0].parent_identities.clear();
    receipt.receipt_identity.clear();
    assert!(
        validate_receipt(&receipt, &profile)
            .iter()
            .any(|diagnostic| diagnostic.contains("parent authority"))
    );
}

#[test]
fn claim_promotion_is_rejected() {
    let profile = profile();
    let mut receipt = blocked_receipt(&profile).expect("blocked decision");
    receipt.claims.push("MIRAI proves whole-program correctness".to_owned());
    receipt.receipt_identity.clear();
    assert!(validate_receipt(&receipt, &profile).iter().any(|diagnostic| diagnostic.contains("overclaims")));
}

#[test]
fn default_paths_do_not_gain_mirai_provider_dependencies() {
    let manifest = include_str!("../../Cargo.toml");
    let cli = include_str!("../cli.rs");
    assert!(!manifest.contains("mirai-annotations"));
    assert!(!manifest.contains("mirai ="));
    assert!(!cli.contains("MiraiSemanticPilot"));
}
