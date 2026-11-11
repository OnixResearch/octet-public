// Tests for bounded local producer execution. The pinned Semgrep and CodeQL executables are absent
// from this environment, so every process observation here comes from the clearly labeled
// `tests/fixtures/evidence-rails/sarif-producer-fixture.sh` test-only executable. These tests prove
// Octet's binding, scope, and failure classification; they do not prove real scanner behavior.
use std::path::Path;
use std::path::PathBuf;

use super::*;

const SEMGREP_VERSION: &str = "1.180.0";
const CODEQL_VERSION: &str = "2.27.2";

const SEMGREP_CLEAN_REPORT: &str = r#"{
  "version": "2.1.0",
  "runs": [{
    "tool": {"driver": {"name": "Semgrep", "semanticVersion": "1.180.0", "rules": [
      {"id": "octet.rust.unsafe-block", "defaultConfiguration": {"level": "warning"}}
    ]}},
    "results": []
  }]
}"#;

const SEMGREP_VIOLATION_REPORT: &str = r#"{
  "version": "2.1.0",
  "runs": [{
    "tool": {"driver": {"name": "Semgrep", "semanticVersion": "1.180.0", "rules": [
      {"id": "octet.rust.expect-without-rationale", "defaultConfiguration": {"level": "error"}}
    ]}},
    "results": [{
      "ruleId": "octet.rust.expect-without-rationale",
      "level": "error",
      "message": {"text": "expect call aborts without a declared rationale"},
      "locations": [{"physicalLocation": {"artifactLocation": {"uri": "src/lib.rs"}, "region": {"startLine": 3, "startColumn": 5}}}]
    }]
  }]
}"#;

const CODEQL_CLEAN_REPORT: &str = r#"{
  "version": "2.1.0",
  "runs": [{
    "tool": {
      "driver": {"name": "CodeQL", "semanticVersion": "2.27.2", "rules": [
        {"id": "rust/demo-let-binding", "defaultConfiguration": {"level": "warning"}}
      ]},
      "extensions": [{"name": "codeql/rust-queries", "semanticVersion": "0.1.46"}]
    },
    "results": []
  }]
}"#;

fn fixture_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/evidence-rails/sarif-producer-fixture.sh")
}

fn fixture_executable(_directory: &Path, _name: &str) -> PathBuf {
    // Execute the checked-in fixture directly. Copying it into a per-test directory and executing
    // the copy races with parallel rewrites and can surface ETXTBSY; the fixture is stateless and
    // selected entirely by environment variables, so one stable executable is sufficient.
    fixture_source()
}

fn workspace() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("create producer workspace");
    std::fs::create_dir_all(directory.path().join("src")).expect("create source directory");
    std::fs::write(directory.path().join("src/lib.rs"), "pub fn f() {}\n").expect("write source file");
    directory
}

fn semgrep_profile(workspace: &Path) -> ProducerProfile {
    let mut profile = selected_profile("semgrep-exec-local").expect("checked Semgrep producer profile");
    let rules = b"rules:\n  - id: octet.rust.unsafe-block\n";
    std::fs::write(workspace.join("producer-rules.yml"), rules).expect("write test rule bundle");
    profile.rules.path = "producer-rules.yml".to_owned();
    profile.rules.identity = prefixed_identity(rules);
    profile.scope.paths = vec!["src".to_owned()];
    profile
}

fn codeql_profile(workspace: &Path) -> ProducerProfile {
    let mut profile = selected_profile("codeql-exec-local").expect("checked CodeQL producer profile");
    profile.scope.paths = vec!["src".to_owned()];
    let _ = workspace;
    profile
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Behavior {
    Run,
    Missing,
    TimedOut,
    Overflow,
}

struct FixtureExecutor {
    program: PathBuf,
    behavior: Behavior,
    environment: Vec<(String, String)>,
}

impl FixtureExecutor {
    fn new(program: PathBuf, behavior: Behavior) -> Self {
        Self {
            program,
            behavior,
            environment: Vec::new(),
        }
    }

    fn with_environment(mut self, environment: &[(&str, &str)]) -> Self {
        self.environment = environment.iter().map(|(name, value)| ((*name).to_owned(), (*value).to_owned())).collect();
        self
    }
}

impl Executor for FixtureExecutor {
    fn resolve(&self, name: &str, _current_dir: &Path) -> Result<ExecutableObservation, StepFailure> {
        if self.behavior == Behavior::Missing {
            return Err(StepFailure::new(
                FailureKind::NotFound,
                format!("test-only fixture marks `{name}` as not installed"),
            ));
        }
        let bytes =
            std::fs::read(&self.program).map_err(|error| StepFailure::new(FailureKind::Spawn, error.to_string()))?;
        Ok(ExecutableObservation {
            resolved_path: self.program.display().to_string(),
            identity: prefixed_identity(&bytes),
        })
    }

    fn run(&self, argv: &[String], current_dir: &Path, _limits: &ProducerLimits) -> Result<StepOutput, StepFailure> {
        match self.behavior {
            Behavior::TimedOut => {
                return Err(StepFailure::new(FailureKind::TimedOut, "test-only fixture reports a wall-clock timeout"));
            }
            Behavior::Overflow => {
                return Err(StepFailure::new(
                    FailureKind::OutputLimitExceeded,
                    "test-only fixture reports a per-stream output overflow",
                ));
            }
            Behavior::Missing | Behavior::Run => {}
        }
        let mut command = std::process::Command::new(&self.program);
        command.args(&argv[1..]).current_dir(current_dir);
        for (name, value) in &self.environment {
            command.env(name, value);
        }
        let output = command.output().map_err(|error| StepFailure::new(FailureKind::Spawn, error.to_string()))?;
        Ok(StepOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

fn run_semgrep(
    directory: &Path,
    profile: &ProducerProfile,
    report: &str,
    mode: &str,
    stage_name: &str,
    extra_environment: &[(&str, &str)],
) -> (ProducerExecution, String) {
    let program = fixture_executable(directory, "semgrep");
    let mut executor = FixtureExecutor::new(program, Behavior::Run).with_environment(&[
        ("SARIF_FIXTURE_VERSION", SEMGREP_VERSION),
        ("SARIF_FIXTURE_MODE", mode),
        ("SARIF_FIXTURE_REPORT", report),
    ]);
    executor
        .environment
        .extend(extra_environment.iter().map(|(name, value)| ((*name).to_owned(), (*value).to_owned())));
    let stage = directory.join(stage_name);
    execute(
        profile,
        &ExecuteRequest {
            workspace_root: directory,
            stage_dir: &stage,
        },
        &executor,
    )
    .expect("producer execution shell")
}

fn run_codeql(directory: &Path, profile: &ProducerProfile, mode: &str) -> (ProducerExecution, String) {
    let program = fixture_executable(directory, "codeql");
    let executor = FixtureExecutor::new(program, Behavior::Run).with_environment(&[
        ("SARIF_FIXTURE_VERSION", CODEQL_VERSION),
        ("SARIF_FIXTURE_MODE", mode),
        ("SARIF_FIXTURE_REPORT", CODEQL_CLEAN_REPORT),
    ]);
    let stage = directory.join("stage-codeql");
    execute(
        profile,
        &ExecuteRequest {
            workspace_root: directory,
            stage_dir: &stage,
        },
        &executor,
    )
    .expect("producer execution shell")
}

#[test]
fn successful_clean_execution_binds_tool_rules_scope_and_report() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-clean", &[]);

    assert_eq!(execution.outcome, OUTCOME_COMPLETED);
    assert_eq!(execution.evidence_origin, "octet_executed");
    let executable = execution.executable.as_ref().expect("executable binding");
    assert_eq!(executable.declared_version, SEMGREP_VERSION);
    assert_eq!(executable.observed_version, SEMGREP_VERSION);
    assert_eq!(execution.rules.identity, profile.rules.identity);
    assert_eq!(execution.scope.selected_paths, vec!["src".to_owned()]);
    assert_eq!(execution.source_files.len(), 1);
    assert_eq!(
        execution.source_snapshot_identity,
        source_snapshot_identity(&execution.source_files).expect("snapshot identity")
    );
    assert_eq!(execution.steps.last().expect("analyze step").exit_code, Some(0));
    assert!(validate_execution(&execution).is_empty(), "{:?}", validate_execution(&execution));

    let evaluation = evaluate_executed(&report, &execution);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("clean executed details");
    assert!(details.findings.is_empty());
    assert_eq!(details.raw_report_identity, execution.report.as_ref().expect("report receipt").identity);
    assert_eq!(
        execution_identity(&execution).expect("execution identity"),
        prefixed_identity(&execution_bytes(&execution).expect("execution bytes"))
    );
}

#[test]
fn violation_findings_keep_external_rule_severity_and_scope() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_VIOLATION_REPORT, "violation", "stage-violation", &[]);
    assert_eq!(execution.outcome, OUTCOME_COMPLETED);

    let details = evaluate_executed(&report, &execution).details.expect("violation details");
    assert_eq!(details.findings.len(), 1);
    let finding = &details.findings[0];
    assert_eq!(finding.source_driver, "Semgrep");
    assert_eq!(finding.source_rule_id, "octet.rust.expect-without-rationale");
    assert_eq!(finding.source_level, "error");
    assert_eq!(finding.gate_severity, "error");
    assert_eq!(finding.file, "src/lib.rs");
    assert_eq!((finding.line, finding.column), (3, 5));
}

#[test]
fn missing_and_mis_versioned_tools_never_produce_clean_evidence() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());

    let program = fixture_executable(directory.path(), "semgrep");
    let stage = directory.path().join("stage-missing");
    let (missing, report) = execute(
        &profile,
        &ExecuteRequest {
            workspace_root: directory.path(),
            stage_dir: &stage,
        },
        &FixtureExecutor::new(program.clone(), Behavior::Missing),
    )
    .expect("missing-tool execution shell");
    assert_eq!(missing.outcome, OUTCOME_UNSUPPORTED);
    assert!(report.is_empty());
    assert!(evaluate_executed(&report, &missing).details.is_none());

    let (wrong, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-wrong-version", &[(
            "SARIF_FIXTURE_VERSION",
            "9.9.9",
        )]);
    assert_eq!(wrong.outcome, OUTCOME_TOOL_ERROR);
    assert!(wrong.diagnostics.iter().any(|diagnostic| diagnostic.contains("pinned")));
    assert!(evaluate_executed(&report, &wrong).details.is_none());

    let _ = program;
}

#[test]
fn changed_rule_pack_fails_before_scan() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    std::fs::write(directory.path().join("producer-rules.yml"), b"rules:\n  - id: octet.rust.changed\n")
        .expect("replace rule bundle");
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-clean", &[]);
    assert_eq!(execution.outcome, OUTCOME_TOOL_ERROR);
    assert!(
        execution
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("no longer matches its checked identity"))
    );
    assert!(report.is_empty());
}

#[test]
fn source_mutation_during_scan_fails_closed() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let source = directory.path().join("src/lib.rs");
    let source_text = source.display().to_string();
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-mutated", &[(
            "SARIF_FIXTURE_MUTATE",
            source_text.as_str(),
        )]);
    assert_eq!(execution.outcome, OUTCOME_FAILED);
    assert!(execution.diagnostics.iter().any(|diagnostic| diagnostic.contains("source changed")));
    assert!(evaluate_executed(&report, &execution).details.is_none());
}

#[test]
fn stale_or_mismatched_codeql_database_fails() {
    let directory = workspace();
    let profile = codeql_profile(directory.path());
    let stage = directory.path().join("stage-codeql");
    let database = stage.join(DATABASE_DIR_NAME);
    std::fs::create_dir_all(&database).expect("create stale database directory");
    let binding = DatabaseBinding {
        source_snapshot_identity: "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        language: "rust".to_owned(),
        build_mode: "none".to_owned(),
        query_packs: profile.query_packs.iter().map(query_pack_receipt).collect(),
        executable_identity: "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
    };
    let program = fixture_executable(directory.path(), "codeql");
    let mut bindings = vec![
        binding.clone(),
        DatabaseBinding {
            query_packs: Vec::new(),
            ..binding.clone()
        },
        DatabaseBinding {
            executable_identity: "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_owned(),
            ..binding
        },
    ];
    for (index, binding) in bindings.drain(..).enumerate() {
        let database = directory.path().join(format!("stage-codeql-{index}")).join(DATABASE_DIR_NAME);
        std::fs::create_dir_all(&database).expect("create stale database directory");
        std::fs::write(
            database.join(DATABASE_BINDING_FILE_NAME),
            serde_json::to_vec(&binding).expect("serialize stale database binding"),
        )
        .expect("write stale database binding");
        let stage = database.parent().expect("database stage").to_path_buf();
        let executor = FixtureExecutor::new(program.clone(), Behavior::Run).with_environment(&[
            ("SARIF_FIXTURE_VERSION", CODEQL_VERSION),
            ("SARIF_FIXTURE_MODE", "clean"),
            ("SARIF_FIXTURE_REPORT", CODEQL_CLEAN_REPORT),
        ]);
        let (execution, report) = execute(
            &profile,
            &ExecuteRequest {
                workspace_root: directory.path(),
                stage_dir: &stage,
            },
            &executor,
        )
        .expect("stale database execution shell");
        assert_eq!(execution.outcome, OUTCOME_FAILED);
        assert!(
            execution.diagnostics.iter().any(|diagnostic| diagnostic.contains("stale")),
            "{:?}",
            execution.diagnostics
        );
        assert!(report.is_empty());
    }
    let _ = stage;
}

#[test]
fn failed_scanner_leaves_no_reusable_old_or_empty_report() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let stage = directory.path().join("stage-failed");
    std::fs::create_dir_all(&stage).expect("create stale stage");
    std::fs::write(stage.join(REPORT_FILE_NAME), "{\"stale\":true}").expect("seed old report");

    let (failed, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "fail-no-report", "stage-failed", &[]);
    assert_eq!(failed.outcome, OUTCOME_TOOL_ERROR);
    assert!(report.is_empty());
    assert!(failed.report.is_none());
    assert!(!stage.join(REPORT_FILE_NAME).exists(), "old report was not removed before the scan");

    let (reported, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "fail-with-report", "stage-failed-report", &[]);
    assert_eq!(reported.outcome, OUTCOME_TOOL_ERROR);
    assert!(reported.report.is_none(), "a report from a failed scan is not evidence");
    assert!(report.is_empty());

    let (empty, report) = run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "empty", "stage-empty", &[]);
    assert_eq!(empty.outcome, OUTCOME_INCOMPLETE, "{:?}", empty.diagnostics);
    assert!(empty.report.is_none());
    assert!(report.is_empty());
}

#[test]
fn retained_real_semgrep_reports_import_with_expected_findings() {
    let clean = crate::external::sarif::evaluate(
        include_str!("../../../../standards/sarif/fixtures/semgrep-clean/report.sarif"),
        "semgrep-local",
    );
    assert!(clean.diagnostics.is_empty(), "{:?}", clean.diagnostics);
    let clean = clean.details.expect("clean Semgrep details");
    assert!(clean.findings.is_empty());

    let violation = crate::external::sarif::evaluate(
        include_str!("../../../../standards/sarif/fixtures/semgrep-violation/report.sarif"),
        "semgrep-local",
    );
    assert!(violation.diagnostics.is_empty(), "{:?}", violation.diagnostics);
    let violation = violation.details.expect("violating Semgrep details");
    assert_eq!(violation.findings.len(), 1);
    assert_eq!(violation.findings[0].source_rule_id, "standards.sarif.octet.rust.expect-without-rationale");
    assert_eq!(violation.findings[0].file, "src/lib.rs");
    assert_eq!(violation.findings[0].source_level, "error");
    assert_eq!(violation.source_driver, "Semgrep");
    assert_eq!(violation.supported_version, "2.1.0");
}

#[test]
fn retained_real_codeql_reports_import_with_expected_findings() {
    let clean = crate::external::sarif::evaluate(
        include_str!("../../../../standards/sarif/fixtures/codeql-clean/report.sarif"),
        "codeql-local",
    );
    assert!(clean.diagnostics.is_empty(), "{:?}", clean.diagnostics);
    let clean = clean.details.expect("clean CodeQL details");
    assert!(clean.findings.is_empty());

    let violation = crate::external::sarif::evaluate(
        include_str!("../../../../standards/sarif/fixtures/codeql-violation/report.sarif"),
        "codeql-local",
    );
    assert!(violation.diagnostics.is_empty(), "{:?}", violation.diagnostics);
    let violation = violation.details.expect("violating CodeQL details");
    assert_eq!(violation.findings.len(), 1);
    assert_eq!(violation.findings[0].source_rule_id, "rust/non-https-url");
    assert_eq!(violation.findings[0].file, "src/lib.rs");
    assert_eq!(violation.findings[0].source_level, "warning");
    assert_eq!(violation.source_driver, "CodeQL");
    assert_eq!(violation.supported_version, "2.1.0");
}

#[test]
fn timeout_and_output_overflow_are_not_clean() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let program = fixture_executable(directory.path(), "semgrep");
    for (behavior, outcome) in [
        (Behavior::TimedOut, OUTCOME_TIMEOUT),
        (Behavior::Overflow, OUTCOME_INCOMPLETE),
    ] {
        let stage = directory.path().join(format!("stage-{outcome}"));
        let (execution, report) = execute(
            &profile,
            &ExecuteRequest {
                workspace_root: directory.path(),
                stage_dir: &stage,
            },
            &FixtureExecutor::new(program.clone(), behavior),
        )
        .expect("failure execution shell");
        assert_eq!(execution.outcome, outcome);
        assert!(report.is_empty());
    }
}

#[test]
fn changed_exclusions_change_the_bound_snapshot() {
    let directory = workspace();
    std::fs::create_dir_all(directory.path().join("src/tests/fixtures")).expect("create fixture tree");
    std::fs::write(directory.path().join("src/tests/fixtures/ignored.rs"), "pub fn g() {}\n")
        .expect("write excluded source");
    let mut excluded = semgrep_profile(directory.path());
    excluded.scope.exclude_globs = vec!["**/tests/fixtures/**".to_owned()];
    let excluded_snapshot = snapshot_sources(&excluded, directory.path()).expect("excluded snapshot");
    assert!(!excluded_snapshot.files.iter().any(|file| file.path.ends_with("ignored.rs")));

    let mut included = excluded.clone();
    included.scope.exclude_globs = Vec::new();
    let included_snapshot = snapshot_sources(&included, directory.path()).expect("included snapshot");
    assert!(included_snapshot.files.iter().any(|file| file.path.ends_with("ignored.rs")));
    assert_ne!(excluded_snapshot.identity, included_snapshot.identity);
}

#[test]
fn malformed_sarif_fails_closed() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "malformed", "stage-malformed", &[]);
    assert_eq!(execution.outcome, OUTCOME_FAILED);
    assert!(execution.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed")));
    assert!(evaluate_executed(&report, &execution).details.is_none());
}

#[test]
fn unsupported_scope_is_not_clean() {
    let directory = workspace();
    let mut missing_path = semgrep_profile(directory.path());
    missing_path.scope.paths = vec!["does-not-exist".to_owned()];
    let (execution, report) =
        run_semgrep(directory.path(), &missing_path, SEMGREP_CLEAN_REPORT, "clean", "stage-missing-path", &[]);
    assert_eq!(execution.outcome, OUTCOME_UNSUPPORTED);
    assert!(report.is_empty());

    let mut wrong_extension = semgrep_profile(directory.path());
    wrong_extension.scope.extensions = vec![".py".to_owned()];
    let (execution, report) =
        run_semgrep(directory.path(), &wrong_extension, SEMGREP_CLEAN_REPORT, "clean", "stage-wrong-extension", &[]);
    assert_eq!(execution.outcome, OUTCOME_UNSUPPORTED);
    assert!(report.is_empty());
    assert!(execution.diagnostics.iter().any(|diagnostic| diagnostic.contains("not clean")));
}

#[test]
fn report_replacement_after_execution_is_rejected() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let (execution, report) =
        run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-clean", &[]);
    let stage = directory.path().join("stage-replaced");
    std::fs::create_dir_all(&stage).expect("create replacement stage");
    let path = stage.join(REPORT_FILE_NAME);
    std::fs::write(&path, &report).expect("write first report");
    let replaced = read_stable_report_with(&path, profile.limits.max_report_bytes, || {
        std::fs::write(&path, SEMGREP_VIOLATION_REPORT).expect("replace report");
    });
    assert!(replaced.is_err());

    let mut tampered = execution;
    tampered.report.as_mut().expect("report receipt").identity =
        "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_owned();
    let evaluation = evaluate_executed(&report, &tampered);
    assert!(evaluation.details.is_none());
    assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("do not match")));
}

#[test]
fn import_origin_cannot_be_relabeled_as_executed() {
    let directory = workspace();
    let profile = semgrep_profile(directory.path());
    let (execution, _) = run_semgrep(directory.path(), &profile, SEMGREP_CLEAN_REPORT, "clean", "stage-clean", &[]);
    let mut mislabeled = execution;
    mislabeled.evidence_origin = "imported".to_owned();
    let diagnostics = validate_execution(&mislabeled);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("octet_executed")));
}

#[test]
fn codeql_database_creation_and_analysis_are_recorded_separately() {
    let directory = workspace();
    let profile = codeql_profile(directory.path());
    let (execution, report) = run_codeql(directory.path(), &profile, "clean");
    assert_eq!(execution.outcome, OUTCOME_COMPLETED);
    let database = execution.database.as_ref().expect("database receipt");
    assert!(database.created);
    assert_eq!(database.creation.as_ref().expect("creation step").exit_code, Some(0));
    assert_eq!(database.analysis.as_ref().expect("analysis step").exit_code, Some(0));
    assert_eq!(database.binding.source_snapshot_identity, execution.source_snapshot_identity);
    assert_eq!(database.binding.query_packs, profile.query_packs.iter().map(query_pack_receipt).collect::<Vec<_>>());
    assert_eq!(database.binding.executable_identity, execution.executable.as_ref().expect("executable").identity);
    assert!(execution.steps.iter().any(|step| step.step == "analyze"));
    assert!(validate_execution(&execution).is_empty(), "{:?}", validate_execution(&execution));
    let details = evaluate_executed(&report, &execution).details.expect("CodeQL details");
    assert!(details.findings.is_empty());
}

#[test]
fn checked_profile_and_query_pack_identities_are_fresh() {
    assert_eq!(prefixed_identity(PROFILE_JSON.as_bytes()), PROFILE_IDENTITY_VALUE);
    let semgrep = selected_profile("semgrep-exec-local").expect("Semgrep profile");
    assert_eq!(
        semgrep.rules.identity,
        prefixed_identity(include_bytes!("../../../../standards/sarif/semgrep-rust-rules.yml"))
    );
    assert_eq!(
        query_pack_identity("codeql/rust-queries", "0.1.46"),
        "b3:fd1e7991fbb379fa5cfb0180fda5fcfd0db9d35d47ae9fe4238430657945c8d5"
    );
    let codeql = selected_profile("codeql-exec-local").expect("CodeQL profile");
    assert_eq!(codeql.rules.identity, query_pack_set_identity(&codeql.query_packs));
    assert_eq!(intake_profile_id("semgrep-exec-local").expect("intake id"), "semgrep-local");
    assert_eq!(intake_profile_id("codeql-local").expect("intake id"), "codeql-local");
    assert!(glob_matches("**/target/**", "src/target/generated.rs"));
    assert!(glob_matches("**/tests/fixtures/**", "a/b/tests/fixtures/x.rs"));
    assert!(!glob_matches("**/tests/fixtures/**", "a/b/tests/other/x.rs"));
}
