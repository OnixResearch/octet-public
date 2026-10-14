use super::*;

// r[verify octet.dynamic_evidence.cargo_fuzz_verification]
// r[verify octet.dynamic_evidence.named_profile_regression]
const PROFILE_IDENTITY: &str = EXPECTED_PROFILE_IDENTITY;
const COMPILER_IDENTITY: &str = "b3:2222222222222222222222222222222222222222222222222222222222222222";
const LOG_IDENTITY: &str = "b3:3333333333333333333333333333333333333333333333333333333333333333";
const SEED_IDENTITY: &str = "b3:4444444444444444444444444444444444444444444444444444444444444444";
const CRASH_IDENTITY: &str = "b3:5555555555555555555555555555555555555555555555555555555555555555";
const REPRODUCER_IDENTITY: &str = "b3:6666666666666666666666666666666666666666666666666666666666666666";

#[test]
fn clean_and_crash_receipts_preserve_bounded_identity() {
    let clean = fixture();
    assert!(validate(&clean).is_empty());
    assert_eq!(clean.initial_corpus_root, clean.final_corpus_root);

    let mut crash = clean;
    crash.outcome = "crash".to_owned();
    crash.crash_input = Some(artifact("artifacts/crash-deadbeef", CRASH_IDENTITY));
    let reproducer = artifact("artifacts/minimized-deadbeef", REPRODUCER_IDENTITY);
    crash.replay_argv = vec![
        "cargo".to_owned(),
        "fuzz".to_owned(),
        "run".to_owned(),
        TARGET.to_owned(),
        reproducer.path.clone(),
    ];
    crash.minimized_reproducer = Some(reproducer);
    let evaluation = evaluate_envelope(&serde_json::to_string(&crash).expect("serialize crash fixture"));
    assert!(evaluation.diagnostics.is_empty());
    assert!(evaluation.is_crash_observed);
}

#[test]
fn malformed_stale_unsafe_and_over_limit_receipts_fail_closed() {
    assert!(evaluate_envelope("not-json").details.is_none());

    let mut stale = fixture();
    stale.final_corpus_root = PROFILE_IDENTITY.to_owned();
    stale.final_corpus[0].path = "../escape".to_owned();
    stale.final_corpus[0].blake3 = "sha256:not-blake3".to_owned();
    stale.budget.max_runs = MAX_RUNS + 1;
    stale.claimed_properties = vec!["formal_proof".to_owned()];
    let diagnostics = validate(&stale);
    assert!(diagnostics.iter().any(|value| value.contains("root is stale")));
    assert!(diagnostics.iter().any(|value| value.contains("path is unsafe")));
    assert!(diagnostics.iter().any(|value| value.contains("must be BLAKE3")));
    assert!(diagnostics.iter().any(|value| value.contains("run budget")));
    assert!(diagnostics.iter().any(|value| value.contains("proof claims")));
}

#[test]
fn crash_artifacts_replay_and_tier_are_required() {
    let mut invalid = fixture();
    invalid.outcome = "crash".to_owned();
    invalid.evidence_tier = "formal_proof".to_owned();
    invalid.non_claims.clear();
    let diagnostics = validate(&invalid);
    assert!(diagnostics.iter().any(|value| value.contains("original crash input")));
    assert!(diagnostics.iter().any(|value| value.contains("minimized reproducer")));
    assert!(diagnostics.iter().any(|value| value.contains("transcript_assertion")));
    assert!(diagnostics.iter().any(|value| value.contains("required non-claim")));
}

#[test]
fn checked_in_clean_crash_and_invalid_fixtures_are_classified() {
    let clean = evaluate_envelope(include_str!("../../../tests/fixtures/evidence-rails/cargo-fuzz-clean.json"));
    assert!(clean.diagnostics.is_empty(), "{:?}", clean.diagnostics);
    assert!(!clean.is_crash_observed);

    let crash = evaluate_envelope(include_str!("../../../tests/fixtures/evidence-rails/cargo-fuzz-crash.json"));
    assert!(crash.diagnostics.is_empty(), "{:?}", crash.diagnostics);
    assert!(crash.is_crash_observed);

    let invalid = evaluate_envelope(include_str!("../../../tests/fixtures/evidence-rails/cargo-fuzz-invalid.json"));
    assert!(invalid.diagnostics.len() >= 8, "{:?}", invalid.diagnostics);
}

#[test]
fn generated_nickel_profile_is_fresh_and_matches_rust_policy() {
    let bytes = include_bytes!("../../../../dynamic-evidence-profiles.json");
    assert_eq!(format!("b3:{}", blake3::hash(bytes).to_hex()), EXPECTED_PROFILE_IDENTITY);
    let registry: serde_json::Value = serde_json::from_slice(bytes).expect("parse generated dynamic registry");
    let profiles = registry["cargo_fuzz_profiles"].as_array().expect("cargo-fuzz profiles");
    assert_eq!(profiles.len(), 1);
    let profile = &profiles[0];
    assert_eq!(profile["name"], PROFILE_ID);
    assert_eq!(profile["engine"], ENGINE);
    assert_eq!(profile["target"], TARGET);
    assert_eq!(profile["toolchain"], TOOLCHAIN);
    assert_eq!(profile["sanitizer"], SANITIZER);
    assert_eq!(profile["command_argv"], serde_json::json!(command_argv()));
    assert_eq!(profile["max_total_time_seconds"], MAX_TOTAL_TIME_SECONDS);
    assert_eq!(profile["max_runs"], MAX_RUNS);
    assert_eq!(profile["seed"], FIXED_SEED);
    assert_eq!(profile["max_artifact_count"], MAX_ARTIFACT_COUNT);
    assert_eq!(profile["max_artifact_bytes"], MAX_ARTIFACT_BYTES);
    assert_eq!(profile["max_path_bytes"], MAX_PATH_BYTES);
    assert_eq!(profile["non_claims"][0], REQUIRED_NON_CLAIM);
}

#[test]
fn command_shape_and_corpus_root_are_deterministic() {
    assert_eq!(
        command_argv(),
        [
            "cargo",
            "fuzz",
            "run",
            TARGET,
            "--",
            "-max_total_time=60",
            "-runs=100000",
            "-seed=1",
        ]
        .into_iter()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
    );
    let first = artifact("corpus/a", SEED_IDENTITY);
    let second = artifact("corpus/b", CRASH_IDENTITY);
    assert_eq!(corpus_root(&[first.clone(), second.clone()]), corpus_root(&[second, first]));
}

fn fixture() -> ReceiptDetails {
    let corpus = vec![artifact("corpus/seed", SEED_IDENTITY)];
    let root = corpus_root(&corpus);
    ReceiptDetails {
        schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        target: TARGET.to_owned(),
        engine: ENGINE.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        compiler_identity: COMPILER_IDENTITY.to_owned(),
        sanitizer: SANITIZER.to_owned(),
        command_argv: command_argv(),
        budget: Budget {
            max_total_time_seconds: MAX_TOTAL_TIME_SECONDS,
            max_runs: MAX_RUNS,
            seed: FIXED_SEED,
        },
        initial_corpus: corpus.clone(),
        initial_corpus_root: root.clone(),
        final_corpus: corpus,
        final_corpus_root: root,
        dictionary: None,
        outcome: "clean".to_owned(),
        crash_input: None,
        minimized_reproducer: None,
        replay_argv: Vec::new(),
        bounded_log_identity: LOG_IDENTITY.to_owned(),
        evidence_tier: EVIDENCE_TIER.to_owned(),
        claimed_properties: Vec::new(),
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    }
}

fn artifact(path: &str, identity: &str) -> ArtifactIdentity {
    ArtifactIdentity {
        path: path.to_owned(),
        length: 4,
        blake3: identity.to_owned(),
    }
}
