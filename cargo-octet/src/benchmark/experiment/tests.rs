use std::path::PathBuf;

use super::evaluate::evaluate;
use super::evaluate::verify;
use super::execution;
use super::execution::ExperimentMode;
use super::fixtures;
use super::model::ARTIFACT_NAME;
use super::model::ARTIFACT_SCHEMA;
use super::model::AdoptionPhase;
use super::model::ApplicabilityStatus;
use super::model::Availability;
use super::model::Category;
use super::model::CompileStatus;
use super::model::ControlOutcome;
use super::model::ExpectedOutcome;
use super::model::ExperimentArtifact;
use super::model::ExperimentObservations;
use super::model::OBSERVATIONS_NAME;
use super::model::ObservationStatus;
use super::model::PROFILE_ID;
use super::model::REPORT_NAME;
use super::profile::embedded_profile_identity;
use super::profile::profile_bytes_identity;
use super::profile::selected_profile;
use super::report::render_report;
use crate::benchmark::prefixed_identity;
use crate::benchmark::pretty_bytes;

const ARCHITECTURE_CASE_COUNT: usize = 14;
const ASSURANCE_CASE_COUNT: usize = 5;
const CORPUS_CASE_COUNT: usize = 19;
const TASK_COUNT: usize = 11;
const TOOL_COUNT: usize = 9;
const APPLICABILITY_COUNT: usize = 99;
const CONTROL_COUNT: usize = 8;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().expect("repository root").to_path_buf()
}

fn corpus_root() -> PathBuf {
    repository_root().join("standards/benchmark/corpus")
}

fn read_retained_observations() -> ExperimentObservations {
    let path = repository_root().join("standards/benchmark").join(OBSERVATIONS_NAME);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn read_checked_in_artifact() -> ExperimentArtifact {
    let path = repository_root().join("standards/benchmark").join(ARTIFACT_NAME);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

fn evaluated_checked_in() -> ExperimentArtifact {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let observations = read_retained_observations();
    let evaluation = evaluate(PROFILE_ID, &observations);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let artifact = evaluation.artifact.expect("artifact");
    assert_eq!(artifact.profile, profile);
    artifact
}

#[test]
fn profile_identity_matrix_and_corpus_sources_are_frozen() {
    assert_eq!(embedded_profile_identity(), profile_bytes_identity());
    assert_eq!(profile_bytes_identity(), super::PROFILE_IDENTITY);
    let profile = selected_profile(PROFILE_ID).expect("profile");
    assert!(selected_profile("ambient-experiment").is_err());
    assert_eq!(profile.corpus.len(), CORPUS_CASE_COUNT);
    assert_eq!(profile.tasks.len(), TASK_COUNT);
    assert_eq!(profile.tools.len(), TOOL_COUNT);
    assert_eq!(profile.applicability.len(), APPLICABILITY_COUNT);
    assert!(profile.declared_before_measurement);
    assert!(profile.coverage_boundary.bounded);
    assert!(!profile.coverage_boundary.complete_rust_coverage);
    let architecture = profile.corpus.iter().filter(|case| case.category == Category::Architecture).count();
    let assurance = profile.corpus.iter().filter(|case| case.category == Category::Assurance).count();
    assert_eq!(architecture, ARCHITECTURE_CASE_COUNT);
    assert_eq!(assurance, ASSURANCE_CASE_COUNT);
    let mut supported = 0;
    let mut partial = 0;
    let mut unsupported = 0;
    for entry in &profile.applicability {
        match entry.status {
            ApplicabilityStatus::Supported => supported += 1,
            ApplicabilityStatus::Partial => partial += 1,
            ApplicabilityStatus::Unsupported => {
                unsupported += 1;
                assert!(entry.case_ids.is_empty());
            }
        }
    }
    assert!(supported > 0 && partial > 0 && unsupported > 0);
    for case in &profile.corpus {
        if case.scope.empty {
            assert_eq!(case.expected, ExpectedOutcome::EmptyScope);
            assert_eq!(case.source_identity, prefixed_identity(b""));
            continue;
        }
        let path = repository_root().join(&case.source_path);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        assert_eq!(prefixed_identity(&bytes), case.source_identity, "source identity drift for {}", case.source_path);
        assert!(!case.rationale.is_empty());
        assert!(!case.reviewer_role.is_empty());
    }
    for tool in &profile.tools {
        for (path, identity) in tool.config_path.iter().zip(tool.config_identity.iter()) {
            let bytes = std::fs::read(repository_root().join(path))
                .unwrap_or_else(|error| panic!("read config {path}: {error}"));
            assert_eq!(prefixed_identity(&bytes), *identity, "config identity drift for {path}");
        }
    }
}

#[test]
fn checked_in_observations_artifact_and_report_verify() {
    let observations = read_retained_observations();
    let artifact = evaluated_checked_in();
    let checked_in = read_checked_in_artifact();
    assert_eq!(checked_in.schema_version, ARTIFACT_SCHEMA);
    assert_eq!(artifact, checked_in);
    assert!(verify(PROFILE_ID, &checked_in).is_empty(), "{:?}", verify(PROFILE_ID, &checked_in));
    let report = render_report(&checked_in);
    for needle in [
        "Cross-tool benchmark experiment report",
        "### Unsupported applicability cells",
        "## Coverage boundary and non-claims",
        "## Negative controls",
        "Precision and recall use only the declared comparable subset.",
        "standards/benchmark/corpus/crates/app/src/violating.rs#dependency-boundary-violating",
    ] {
        assert!(report.contains(needle), "report missing `{needle}`");
    }
    let report_path = repository_root().join("standards/benchmark").join(REPORT_NAME);
    let checked_in_report = std::fs::read_to_string(&report_path).expect("checked-in report");
    assert_eq!(render_report(&checked_in), checked_in_report);
    let observation_bytes = pretty_bytes(&observations, "observations").expect("observations bytes");
    assert_eq!(artifact.observations_identity, prefixed_identity(&observation_bytes));
    let artifact_bytes = pretty_bytes(&checked_in, "artifact").expect("artifact bytes");
    assert_eq!(
        artifact.derived_identity,
        prefixed_identity(&pretty_bytes(&artifact.derived, "derived").expect("derived bytes"))
    );
    assert!(artifact_bytes.len() > artifact.observations_identity.len());
}

#[test]
fn architecture_and_assurance_metrics_stay_separate_and_do_not_upgrade_unsupported_scope() {
    let artifact = evaluated_checked_in();
    assert!(artifact.derived.architecture.iter().all(|metrics| metrics.category == Category::Architecture));
    assert!(artifact.derived.assurance.iter().all(|metrics| metrics.category == Category::Assurance));
    let octet_dependency = artifact
        .derived
        .architecture
        .iter()
        .find(|metrics| metrics.tool_id == "octet" && metrics.task_id == "dependency-boundary")
        .expect("octet dependency metrics");
    assert_eq!(octet_dependency.true_positive, 1);
    assert_eq!(octet_dependency.true_negative, 1);
    assert_eq!(octet_dependency.precision_basis_points, Some(10_000));
    assert_eq!(octet_dependency.recall_basis_points, Some(10_000));
    let octet_empty = artifact
        .derived
        .architecture
        .iter()
        .find(|metrics| metrics.tool_id == "octet" && metrics.task_id == "empty-selected-scope")
        .expect("octet empty-scope metrics");
    assert_eq!(octet_empty.empty_scope_cases, 1);
    assert_eq!(octet_empty.true_negative, 0);
    assert_eq!(octet_empty.comparable_cases, 0);
    for tool in ["archunit-rust", "semgrep", "codeql"] {
        let metrics = artifact
            .derived
            .architecture
            .iter()
            .find(|metrics| metrics.tool_id == tool && metrics.applicability_status != ApplicabilityStatus::Unsupported)
            .expect("unavailable tool metrics");
        assert_eq!(metrics.availability, Availability::Unavailable);
        assert_eq!(metrics.comparable_cases, 0);
        assert!(metrics.unsupported_cases > 0);
        assert_eq!(metrics.true_negative, 0);
        assert_eq!(metrics.precision_basis_points, None);
        assert_eq!(metrics.recall_basis_points, None);
    }
    let assurance: Vec<_> = artifact.derived.assurance.iter().filter(|metrics| metrics.tool_id == "octet").collect();
    assert!(!assurance.is_empty());
    let true_positive: u64 = assurance.iter().map(|metrics| metrics.true_positive).sum();
    let true_negative: u64 = assurance.iter().map(|metrics| metrics.true_negative).sum();
    assert_eq!(true_positive, 4);
    assert_eq!(true_negative, 1);
    assert!(artifact.derived.provenance.executed_locally.iter().all(|tool| tool == "clippy"));
    assert!(artifact.derived.provenance.unavailable.iter().any(|tool| tool == "semgrep"));
}

#[test]
fn negative_controls_are_contained_and_tampering_is_rejected() {
    let artifact = evaluated_checked_in();
    assert_eq!(artifact.controls.len(), CONTROL_COUNT);
    for control in &artifact.controls {
        assert_eq!(control.outcome, ControlOutcome::Contained, "{control:?}");
        assert!(!control.evidence.is_empty(), "{control:?}");
    }

    let profile = selected_profile(PROFILE_ID).expect("profile");
    let observations = read_retained_observations();

    let mut stale = observations.clone();
    stale.observations[0].source_identity = prefixed_identity(b"renamed");
    let diagnostics = evaluate(PROFILE_ID, &stale).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("stale oracle")), "{diagnostics:?}");

    let mut dropped = observations.clone();
    if let Some(index) = dropped
        .observations
        .iter()
        .position(|observation| observation.status == ObservationStatus::TimedOut)
    {
        dropped.observations.remove(index);
    } else {
        let index = dropped
            .observations
            .iter()
            .position(|observation| observation.case_id.is_some())
            .expect("case observation");
        dropped.observations.remove(index);
    }
    let diagnostics = evaluate(PROFILE_ID, &dropped).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("missing")), "{diagnostics:?}");

    let mut noncompiling = observations.clone();
    let violating = noncompiling
        .observations
        .iter_mut()
        .find(|observation| {
            observation.findings.iter().any(|finding| !finding.suppressed)
                && observation.status == ObservationStatus::Completed
        })
        .expect("violating observation");
    violating.compile_status = CompileStatus::NotCompiled;
    let diagnostics = evaluate(PROFILE_ID, &noncompiling).diagnostics;
    assert!(diagnostics.iter().any(|value| value.contains("noncompiling")), "{diagnostics:?}");

    let mut changed_scope = artifact.clone();
    changed_scope.observations[0].scope.selected_sources.push("hidden/other.rs".to_owned());
    let diagnostics = verify(PROFILE_ID, &changed_scope);
    assert!(diagnostics.iter().any(|value| value.contains("scope identity")), "{diagnostics:?}");

    let mut changed_environment = artifact.clone();
    changed_environment.observations[0].environment.rust_toolchain = "nightly-unreviewed".to_owned();
    let diagnostics = verify(PROFILE_ID, &changed_environment);
    assert!(diagnostics.iter().any(|value| value.contains("does not reproduce")), "{diagnostics:?}");

    let mut stale_derived = artifact.clone();
    stale_derived.derived.architecture[0].true_positive += 1;
    let diagnostics = verify(PROFILE_ID, &stale_derived);
    assert!(diagnostics.iter().any(|value| value.contains("does not reproduce")), "{diagnostics:?}");

    let mut complete = artifact.clone();
    complete.profile.coverage_boundary.complete_rust_coverage = true;
    let diagnostics = verify(PROFILE_ID, &complete);
    assert!(diagnostics.iter().any(|value| value.contains("complete Rust coverage")), "{diagnostics:?}");

    let mut unsupported = observations.clone();
    let clean = profile
        .corpus
        .iter()
        .find(|case| case.expected == ExpectedOutcome::Clean && case.category == Category::Architecture)
        .expect("clean case");
    let observation = unsupported
        .observations
        .iter_mut()
        .find(|observation| observation.case_id.as_deref() == Some(clean.case_id.as_str()))
        .expect("clean observation");
    observation.status = ObservationStatus::Unsupported;
    observation.unavailable_reason = Some("negative-control unsupported scope".to_owned());
    let evaluation = evaluate(PROFILE_ID, &unsupported);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
}

#[test]
fn execute_mode_marks_missing_executables_unsupported_and_keeps_runs() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let run = execution::run(ExperimentMode::Execute, PROFILE_ID, None, &corpus_root());
    let Ok(run) = run else {
        panic!("execute mode failed: {:?}", run.err());
    };
    assert_eq!(run.artifact.profile.profile_id, PROFILE_ID);
    assert!(!run.artifact.derived.architecture.is_empty());
    assert!(!run.artifact.derived.assurance.is_empty());
    assert!(run.report.contains("## Measurement provenance"));
    let unavailable: Vec<&str> = run
        .artifact
        .tool_availability
        .iter()
        .filter(|record| record.status == Availability::Unavailable)
        .map(|record| record.tool_id.as_str())
        .collect();
    if !unavailable.is_empty() {
        for tool in unavailable {
            let observations = run
                .artifact
                .observations
                .iter()
                .filter(|observation| observation.tool_id == tool)
                .collect::<Vec<_>>();
            assert!(
                observations.iter().all(|observation| observation.status == ObservationStatus::Unavailable),
                "unavailable tool {tool} produced a comparable observation"
            );
        }
    }
    for metrics in run.artifact.derived.architecture.iter().chain(run.artifact.derived.assurance.iter()) {
        if metrics.availability == Availability::Unavailable {
            assert_eq!(metrics.comparable_cases, 0);
            assert_eq!(metrics.true_negative, 0);
        }
    }
    assert!(!profile.adoption.clean_consumer.selection_id.is_empty());
}

#[test]
fn local_clippy_probe_and_clean_consumer_measurement_when_available() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let clippy = profile.tools.iter().find(|tool| tool.tool_id == "clippy").expect("clippy pin");
    let probe = execution::probe_tool(clippy);
    if probe.status == Availability::Unavailable {
        assert!(probe.reason.as_deref().is_some_and(|reason| !reason.is_empty()));
        eprintln!("clippy probe unavailable in this environment: {:?}", probe.reason);
        return;
    }
    assert!(probe.version_output.contains("clippy"));
    let outcome = execution::run_command(&corpus_root(), &[
        "cargo".to_owned(),
        "check".to_owned(),
        "--workspace".to_owned(),
        "--all-targets".to_owned(),
        "--offline".to_owned(),
    ])
    .expect("local cargo check");
    assert!(outcome.succeeded, "{outcome:?}");
    assert!(outcome.duration_ns > 0);
    let (initial, incremental) = execution::measure_target(
        &profile,
        clippy,
        &corpus_root(),
        &profile.adoption.clean_consumer.selection_id,
        &clippy.run_argv,
        &clippy.run_argv,
        true,
        true,
    );
    if initial.measured {
        assert!(initial.wall_duration_ns.is_some_and(|value| value > 0));
    }
    if incremental.measured {
        assert!(incremental.wall_duration_ns.is_some_and(|value| value > 0));
    }
    assert_eq!(initial.phase, AdoptionPhase::Initial);
    assert_eq!(incremental.phase, AdoptionPhase::Incremental);
}

#[test]
fn repository_assurance_baseline_rejects_invalid_fixtures_when_available() {
    let root = repository_root();
    let script = "standards/benchmark/assurance/repository-baseline.sh";
    if !root.join(script).is_file() {
        eprintln!("repository baseline script unavailable");
        return;
    }
    if execution::run_command(&root, &["jq".to_owned(), "--version".to_owned()]).is_err() {
        eprintln!("jq unavailable for the repository baseline");
        return;
    }
    let cases = [
        ("stale-evidence.json", false),
        ("changed-policy.json", false),
        ("missing-execution.json", false),
        ("invalid-proof-linkage.json", false),
        ("valid-evidence-control.json", true),
    ];
    for (name, expected) in cases {
        let fixture = format!("standards/benchmark/assurance/{name}");
        let outcome = execution::run_command(&root, &["bash".to_owned(), script.to_owned(), fixture.clone()]);
        let Ok(outcome) = outcome else {
            eprintln!("repository baseline cannot run: {outcome:?}");
            return;
        };
        assert_eq!(outcome.succeeded, expected, "{fixture}: {outcome:?}");
    }
}

#[test]
#[ignore = "writes the checked experiment observations, artifact, and report"]
fn regenerate_experiment_artifacts() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let mut observations = fixtures::recorded_observations(&profile);
    let clippy = profile.tools.iter().find(|tool| tool.tool_id == "clippy").expect("clippy pin");
    let clean = profile.adoption.clean_consumer.selection_id.clone();
    let (initial, incremental) = execution::measure_target(
        &profile,
        clippy,
        &corpus_root(),
        &clean,
        &clippy.run_argv,
        &clippy.run_argv,
        true,
        !clippy.run_argv.is_empty(),
    );
    if initial.measured || incremental.measured {
        for observation in &mut observations.adoption {
            if observation.tool_id == "clippy" && observation.target_selection == clean {
                match observation.phase {
                    AdoptionPhase::Initial => *observation = initial.clone(),
                    AdoptionPhase::Incremental => *observation = incremental.clone(),
                }
            }
        }
        let probe = execution::probe_tool(clippy);
        if probe.status != Availability::Unavailable
            && let Some(record) = observations.tool_availability.iter_mut().find(|record| record.tool_id == "clippy")
        {
            record.status = Availability::Local;
            record.version_output = probe.version_output;
            record.version_identity = probe.version_identity;
            record.reason = None;
        }
    }
    let evaluation = evaluate(PROFILE_ID, &observations);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let artifact = evaluation.artifact.expect("artifact");
    let report = render_report(&artifact);
    let directory = repository_root().join("standards/benchmark");
    std::fs::write(
        directory.join(OBSERVATIONS_NAME),
        pretty_bytes(&observations, "experiment observations").expect("observations bytes"),
    )
    .expect("write observations");
    std::fs::write(
        directory.join(ARTIFACT_NAME),
        pretty_bytes(&artifact, "experiment artifact").expect("artifact bytes"),
    )
    .expect("write artifact");
    std::fs::write(directory.join(REPORT_NAME), report.as_bytes()).expect("write report");
    eprintln!("wrote {} / {} / {}", OBSERVATIONS_NAME, ARTIFACT_NAME, REPORT_NAME);
}
