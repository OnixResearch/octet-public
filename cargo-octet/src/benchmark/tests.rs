// r[verify octet.benchmark_evidence.verification]
use super::test_support::*;
use super::*;

const PROFILE_WORKLOAD_COUNT: usize = 4;
const PROFILE_CORPUS_COUNT: usize = 2;

#[test]
fn profile_registry_identity_workloads_corpora_and_unknown_selection_are_checked() {
    let profile_bytes = include_bytes!("../../../standards/benchmark/profiles.json");
    assert_eq!(prefixed_identity(profile_bytes), PROFILE_IDENTITY);
    let profile = selected_profile(PROFILE_ID).expect("profile");
    assert_eq!(profile.workloads.len(), PROFILE_WORKLOAD_COUNT);
    assert_eq!(profile.signal_corpus.len(), PROFILE_CORPUS_COUNT);
    assert!(selected_profile("ambient-benchmark").is_err());
}

#[test]
fn starter_check_remains_independent_and_benchmark_selection_is_explicit() {
    let args = ["cargo", "octet", "check"].into_iter().map(std::ffi::OsString::from);
    assert!(matches!(crate::cli::parse(args), Ok(crate::cli::CommandArgs::Check(_))));
    assert_eq!(
        crate::evidence_rails::EvidenceRailKind::parse("benchmark-evidence"),
        Ok(crate::evidence_rails::EvidenceRailKind::BenchmarkEvidence)
    );
}

#[test]
fn stable_benchmark_receipt_binds_performance_resources_signals_and_claims() {
    let mut input = valid_input();
    let first = validation::evaluate(&input, &selected_profile(PROFILE_ID).expect("profile"));
    let first_details = first.details.expect("valid first evaluation");
    input.public_claims.push(PublicClaim {
        metric: "wall-duration".to_owned(),
        value: "stable".to_owned(),
        projection_identity: first_details.projection_identity,
        corpus: "synthetic-octet-benchmark-cases".to_owned(),
        environment: "checked profile environment".to_owned(),
        sample_range: "five baseline and five candidate samples".to_owned(),
        interpretation_boundary: "median wall time for one no-change cargo metadata workload".to_owned(),
    });
    let evaluation = validation::evaluate(&input, &selected_profile(PROFILE_ID).expect("profile"));
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("details");
    assert_eq!(details.projection.performance.result, ComparisonResult::Stable);
    assert_eq!(details.projection.resources.allocation_events, ALLOCATION_EVENTS);
    assert_eq!(details.projection.resources.allocated_bytes, ALLOCATED_BYTES);
    assert_eq!(details.projection.resources.known_copy_bytes, KNOWN_COPY_BYTES);
    assert_eq!(details.projection.resources.unknown_copy_length_count, 1);
    assert_eq!(details.projection.signals.true_positive, EXPECTED_SIGNAL_CLASS_COUNT);
    assert_eq!(details.projection.signals.false_positive, EXPECTED_SIGNAL_CLASS_COUNT);
    assert_eq!(details.projection.signals.false_negative, EXPECTED_SIGNAL_CLASS_COUNT);
    assert_eq!(details.projection.signals.expected_suppression, EXPECTED_SIGNAL_CLASS_COUNT);
    assert_eq!(details.projection.signals.unexpected_suppression, EXPECTED_SIGNAL_CLASS_COUNT);
    assert_eq!(details.projection.signals.inconclusive, EXPECTED_SIGNAL_CLASS_COUNT);
    assert!(validate(&details).is_empty());
    let rendered = request::render(&input).expect("render");
    let artifacts = request::artifact_payloads(&details, &rendered).expect("artifacts");
    assert_eq!(artifacts.len(), 7);
}

#[test]
fn profiled_evidence_rail_receipt_binds_benchmark_details_and_policy() {
    let input = valid_input();
    let rendered = request::render(&input).expect("render benchmark input");
    let baseline_identity = request::baseline_identity(&rendered).expect("baseline identity");
    let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
        rail: crate::evidence_rails::EvidenceRailKind::BenchmarkEvidence,
        mode: "import",
        import_text: &rendered,
        workspace_scope: ".",
        config_path: Some("standards/benchmark/profiles.json"),
        baseline_identity: Some(baseline_identity),
        lockfile_identity: None,
        locked: false,
        advisory_updated_by_prior_step: false,
        profile: Some(PROFILE_ID),
        requested_command: None,
        parent_artifacts: Vec::new(),
        tool_exit_code: None,
    });
    receipt.config_identity = Some(PROFILE_IDENTITY.to_owned());
    assert!(receipt.benchmark_evidence.is_some());
    assert_eq!(receipt.command_argv, input.baseline.command_argv);
    assert!(crate::evidence_rails::validate_receipt(&receipt).is_empty());
}

#[test]
fn comparison_reports_improvement_regression_incomparability_and_uncertainty() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let input = valid_input();
    let mut improved = input.candidate.clone();
    set_all_durations(&mut improved, IMPROVED_DURATION_NS);
    assert_eq!(
        comparison::compare(&input.baseline, &improved, profile.comparison).result,
        ComparisonResult::Improved
    );
    let mut regressed = input.candidate.clone();
    set_all_durations(&mut regressed, REGRESSED_DURATION_NS);
    assert_eq!(
        comparison::compare(&input.baseline, &regressed, profile.comparison).result,
        ComparisonResult::Regressed
    );
    let mut incompatible = input.candidate.clone();
    incompatible.environment.host_arch = "different-architecture".to_owned();
    assert_eq!(
        comparison::compare(&input.baseline, &incompatible, profile.comparison).result,
        ComparisonResult::Incomparable
    );
    let mut noisy = input.candidate.clone();
    noisy.samples[0].wall_duration_ns = NOISY_DURATION_NS;
    assert_eq!(
        comparison::compare(&input.baseline, &noisy, profile.comparison).result,
        ComparisonResult::Inconclusive
    );
    let mut failed = input.candidate;
    failed.samples[0].outcome = "timed_out".to_owned();
    assert_eq!(
        comparison::compare(&input.baseline, &failed, profile.comparison).result,
        ComparisonResult::Inconclusive
    );
}

#[test]
fn malformed_samples_resource_claims_signal_oracles_and_public_claims_fail_closed() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let mut sample_failure = valid_input();
    sample_failure.candidate.samples.pop();
    let diagnostics = validation::evaluate(&sample_failure, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("sample count")));

    let mut hidden_allocation = valid_input();
    hidden_allocation.allocations[0].zero_allocation_claim = true;
    let diagnostics = validation::evaluate(&hidden_allocation, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("contradicts")));

    let mut instrumentation_drift = valid_input();
    let mut drifted_allocation = instrumentation_drift.allocations[0].clone();
    drifted_allocation.interval_id = "second-interval".to_owned();
    drifted_allocation.instrumentation_identity = b3(b"different-instrumentation");
    instrumentation_drift.allocations.push(drifted_allocation);
    let mut drifted_copy = instrumentation_drift.copies[0].clone();
    drifted_copy.site_id = b3(b"different-copy-site");
    drifted_copy.compiler_identity = b3(b"different-compiler");
    instrumentation_drift.copies.push(drifted_copy);
    let diagnostics = validation::evaluate(&instrumentation_drift, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("instrumentation drifted")));
    assert!(diagnostics.iter().any(|value| value.contains("copy cohort drifted")));

    let mut unsupported_observation = valid_input();
    unsupported_observation.candidate.samples[0].cpu_duration_ns = Some(BASE_DURATION_NS);
    let diagnostics = validation::evaluate(&unsupported_observation, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("invalid sample")));

    let mut duplicate_oracle = valid_input();
    duplicate_oracle.oracle.push(duplicate_oracle.oracle[0].clone());
    let diagnostics = validation::evaluate(&duplicate_oracle, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("duplicates case")));

    let mut copy_overclaim = valid_input();
    let first = validation::evaluate(&copy_overclaim, &profile).details.expect("valid projection");
    copy_overclaim.public_claims.push(PublicClaim {
        metric: "copy-bytes".to_owned(),
        value: KNOWN_COPY_BYTES.to_string(),
        projection_identity: first.projection_identity,
        corpus: "synthetic-octet-benchmark-cases".to_owned(),
        environment: "selected environment".to_owned(),
        sample_range: "selected samples".to_owned(),
        interpretation_boundary: "known static sites only".to_owned(),
    });
    let diagnostics = validation::evaluate(&copy_overclaim, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("public claim")));

    let mut stale_claim = valid_input();
    stale_claim.public_claims.push(PublicClaim {
        metric: "wall-duration".to_owned(),
        value: "faster".to_owned(),
        projection_identity: b3(b"stale"),
        corpus: "selected corpus".to_owned(),
        environment: "selected environment".to_owned(),
        sample_range: "selected samples".to_owned(),
        interpretation_boundary: "selected boundary".to_owned(),
    });
    let diagnostics = validation::evaluate(&stale_claim, &profile).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("public claim")));
}

#[test]
fn capability_rooted_resource_inputs_preserve_constant_unknown_and_allocation_observations() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let directory = tempfile::tempdir().expect("temporary directory");
    let llvm = "call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 32, i1 false)\n\
                call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 %length, i1 false)\n";
    std::fs::write(directory.path().join("scan.ll"), llvm).expect("write LLVM fixture");
    let observations = shell::scan_llvm_file(
        directory.path(),
        std::path::Path::new("scan.ll"),
        &profile,
        "workload",
        &b3(COMPILER_IDENTITY_INPUT),
        "x86_64-unknown-linux-gnu",
        "release",
    )
    .expect("copy census");
    assert_eq!(observations.len(), 2);
    assert_eq!(observations[0].constant_length_bytes, Some(KNOWN_COPY_BYTES));
    assert_eq!(observations[1].constant_length_bytes, None);

    let allocations = vec![valid_input().allocations[0].clone()];
    let allocations_json = serde_json::to_vec(&allocations).expect("serialize allocation fixture");
    std::fs::write(directory.path().join("allocations.json"), allocations_json).expect("write allocation fixture");
    let observed_allocations =
        shell::read_allocation_file(directory.path(), std::path::Path::new("allocations.json"), &profile)
            .expect("allocation observations");
    assert_eq!(observed_allocations, allocations);
}

#[test]
fn bounded_shell_uses_fixed_profile_command_and_clear_environment() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let directory = tempfile::tempdir().expect("temporary directory");
    let baseline = run_set(&profile, BASE_DURATION_NS);
    std::fs::create_dir_all(directory.path().join(".octet")).expect("create benchmark fixture root");
    let allocations = vec![AllocationObservation {
        workload_id: baseline.workload_id.clone(),
        instrumentation_identity: b3(b"runtime-allocation-instrumentation"),
        interval_id: "runtime-selected-interval".to_owned(),
        scope: "selected runtime fixture".to_owned(),
        allocation_events: 0,
        allocated_bytes: 0,
        zero_allocation_claim: true,
    }];
    let allocation_bytes = serde_json::to_vec(&allocations).expect("serialize runtime allocation fixture");
    std::fs::write(directory.path().join(&profile.resource_observation.allocation_input_path), allocation_bytes)
        .expect("write runtime allocation fixture");
    std::fs::write(
        directory.path().join(&profile.resource_observation.llvm_input_path),
        "call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 32, i1 false)\n",
    )
    .expect("write runtime LLVM fixture");
    let tools = shell::RuntimeTools {
        cargo: "/bin/true".into(),
        path: "/bin".into(),
        library_path: "/lib".into(),
        kernel_identity: "kernel-fixture".to_owned(),
        cpu_identity: "cpu-fixture".to_owned(),
        governor: "performance".to_owned(),
        load_class: "isolated".to_owned(),
    };
    let observation = shell::run_with(directory.path(), &profile, baseline, tools).expect("bounded benchmark run");
    assert_eq!(observation.input.candidate.samples.len(), SAMPLE_COUNT);
    assert!(observation.input.candidate.samples.iter().all(|sample| sample.outcome == "succeeded"));
    assert!(resources::is_b3(&observation.input.bounded_log_identity));
}

#[test]
#[ignore = "writes the checked benchmark evidence fixture"]
fn regenerate_benchmark_evidence_fixture() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evidence-rails/benchmark-input.json");
    let bytes = request::payload(&valid_input(), "benchmark fixture").expect("render benchmark fixture");
    std::fs::write(&path, bytes).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}
