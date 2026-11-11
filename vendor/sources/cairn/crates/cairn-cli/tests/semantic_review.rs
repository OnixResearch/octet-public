use std::{env, error::Error, fs, path::PathBuf, process::Command};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn temp_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = env::temp_dir().join(format!(
        "cairn-semantic-review-{name}-{}",
        std::process::id()
    ));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn hash() -> String {
    "a".repeat(64)
}

#[test]
fn semantic_review_validate_accepts_valid_receipt() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.semantic_review_receipts.cli]
    // r[verify cairn.semantic_review.semantic_review_receipts.cli.cli.valid]
    // r[verify cairn.semantic_review.semantic_review_boundary.fcis.fcis.cli_shell]
    let dir = temp_dir("valid")?;
    let receipt = dir.join("review.json");
    fs::write(
        &receipt,
        format!(
            r#"{{
  "schema_version": "cairn.semantic-review.review-receipt.v1",
  "review_kind": "code",
  "decision": "approved",
  "reviewer": {{"source": "manual", "id": "reviewer"}},
  "reviewed_inputs": [{{"path": "src/lib.rs", "hash": "{}"}}],
  "findings": []
}}"#,
            hash()
        ),
    )?;
    let output = cairn()
        .args([
            "review",
            "validate",
            receipt.to_str().unwrap(),
            "--root",
            dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["valid"], true);
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn semantic_review_validate_rejects_uncategorized_revision() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.semantic_review_receipts.cli.cli.invalid]
    // r[verify cairn.semantic_review.review_rejection_taxonomy.finding_validation]
    // r[verify cairn.semantic_review.review_rejection_taxonomy.finding_validation.finding_validation.required]
    let dir = temp_dir("invalid")?;
    let receipt = dir.join("review.json");
    fs::write(
        &receipt,
        format!(
            r#"{{
  "schema_version": "cairn.semantic-review.review-receipt.v1",
  "review_kind": "design",
  "decision": "needs_revision",
  "reviewer": {{"source": "agentic", "id": "model-a"}},
  "reviewed_inputs": [{{"path": "design.md", "hash": "{}"}}],
  "findings": []
}}"#,
            hash()
        ),
    )?;
    let output = cairn()
        .args([
            "review",
            "validate",
            receipt.to_str().unwrap(),
            "--root",
            dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(!output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["valid"], false);
    assert_eq!(stdout["issues"][0]["code"], "missing_findings");
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn semantic_review_ledger_reports_counts_and_small_sample_warning() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.review_ledger_analytics.aggregate]
    // r[verify cairn.semantic_review.review_ledger_analytics.aggregate.aggregate.counts]
    // r[verify cairn.semantic_review.review_ledger_analytics.aggregate.aggregate.hashes]
    // r[verify cairn.semantic_review.review_ledger_analytics.input_validation.input_validation.sample_warning]
    let dir = temp_dir("ledger")?;
    fs::write(
        dir.join("review.json"),
        format!(
            r#"{{
  "schema_version": "cairn.semantic-review.review-receipt.v1",
  "review_kind": "plan",
  "decision": "needs_revision",
  "reviewer": {{"source": "manual", "id": "reviewer"}},
  "reviewed_inputs": [{{"path": "proposal.md", "hash": "{}"}}],
  "findings": [{{"id": "F1", "category": "omission", "severity": "high"}}]
}}"#,
            hash()
        ),
    )?;
    let output = cairn()
        .args([
            "review",
            "ledger",
            "--include",
            dir.to_str().unwrap(),
            "--root",
            dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["valid"], true);
    assert_eq!(stdout["analytics"]["counts"]["by_category"]["omission"], 1);
    assert_eq!(stdout["analytics"]["warnings"][0]["code"], "small_sample");
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn semantic_review_ledger_rejects_invalid_input_receipt() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.review_ledger_analytics.input_validation]
    // r[verify cairn.semantic_review.review_ledger_analytics.input_validation.input_validation.invalid_receipt]
    let dir = temp_dir("ledger-invalid")?;
    fs::write(
        dir.join("bad.json"),
        r#"{
  "schema_version": "cairn.semantic-review.review-receipt.v1",
  "review_kind": "plan",
  "decision": "needs_revision",
  "reviewer": {"source": "manual", "id": "reviewer"},
  "reviewed_inputs": [],
  "findings": []
}"#,
    )?;
    let output = cairn()
        .args([
            "review",
            "ledger",
            "--include",
            dir.to_str().unwrap(),
            "--root",
            dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(!output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["valid"], false);
    assert_eq!(
        stdout["analytics"]["invalid_inputs"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn revision_handoff_validate_requires_addressed_evidence() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.revision_handoff_receipts.validation]
    // r[verify cairn.semantic_review.revision_handoff_receipts.validation.validation.addressed_evidence]
    // r[verify cairn.semantic_review.revision_handoff_receipts.validation.validation.new_findings]
    let dir = temp_dir("revision")?;
    let receipt = dir.join("revision.json");
    fs::write(
        &receipt,
        format!(
            r#"{{
  "schema_version": "cairn.semantic-review.revision-handoff.v1",
  "prior_review_receipt_hash": "{}",
  "revision_artifacts": [{{"path": "src/lib.rs", "hash": "{}"}}],
  "finding_dispositions": [
    {{"finding_id": "F1", "disposition": "addressed"}},
    {{"finding_id": "F2", "disposition": "new"}}
  ]
}}"#,
            hash(),
            hash()
        ),
    )?;
    let output = cairn()
        .args([
            "review",
            "revision",
            "validate",
            receipt.to_str().unwrap(),
            "--root",
            dir.to_str().unwrap(),
        ])
        .output()?;
    assert!(!output.status.success());
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let issue_codes = stdout["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(issue_codes.contains(&"missing_addressed_evidence"));
    assert!(issue_codes.contains(&"missing_new_finding_marker"));
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn review_profiles_exposes_policy_owned_vocabularies() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.review_gate_policy_profiles.profiles]
    // r[verify cairn.semantic_review.review_gate_policy_profiles.profiles.profiles.kinds]
    // r[verify cairn.semantic_review.review_gate_policy_profiles.profiles.profiles.criteria]
    // r[verify cairn.semantic_review.review_gate_policy_profiles.sources]
    // r[verify cairn.semantic_review.review_gate_policy_profiles.sources.sources.allowed]
    // r[verify cairn.semantic_review.review_gate_policy_profiles.sources.sources.no_secret]
    let output = cairn().args(["review", "profiles"]).output()?;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["reviewer_sources"][0], "manual");
    assert!(stdout.to_string().contains("required_criteria"));
    assert!(!stdout.to_string().contains("secret"));
    Ok(())
}

#[test]
fn review_metrics_regression_accepts_checked_in_positive_fixture() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.review_metrics_regression_rail.downstream_category_ownership]
    // r[verify cairn.semantic_review.review_metrics_regression_rail.registry_binds_surfaces]
    let fixture = metrics_regression_fixture("positive");
    let output = run_metrics_regression_command(&fixture)?.output()?;
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["valid"], true);
    assert_eq!(stdout["verdict"], "PASS");
    assert_eq!(stdout["reports"].as_array().unwrap().len(), 4);
    assert!(stdout["receipt_hash"].as_str().unwrap().len() >= 64);
    Ok(())
}

#[test]
fn review_metrics_regression_rejects_checked_in_negative_fixtures() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.review_metrics_regression_rail.fixture_pair_enforcement]
    // r[verify cairn.semantic_review.review_metrics_regression_rail.guidance_task_sync]
    // r[verify cairn.semantic_review.review_metrics_regression_rail.secret_free_evidence]
    for (fixture, report, expected_codes) in [
        (
            "negative/registry-drift",
            "registry",
            &["missing_registry_binding"][..],
        ),
        (
            "negative/fixture-pair-drift",
            "fixtures",
            &["negative_fixture_missing_diagnostic"][..],
        ),
        (
            "negative/guidance-task-drift",
            "guidance",
            &[
                "guidance_missing_diagnostic",
                "task_missing_requirement_ref",
                "spec_missing_requirement_ref",
            ][..],
        ),
        (
            "negative/unsafe-evidence",
            "evidence",
            &["unsafe_evidence_field", "unsafe_evidence_text"][..],
        ),
    ] {
        let fixture = metrics_regression_fixture(fixture);
        let output = run_metrics_regression_command(&fixture)?.output()?;
        assert!(!output.status.success());
        let stdout: serde_json::Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(stdout["valid"], false);
        assert_report_contains_issue_codes(&stdout, report, expected_codes);
    }
    Ok(())
}

struct MetricsRegressionFixture {
    root: PathBuf,
    registry: PathBuf,
    fixtures: PathBuf,
    guidance: PathBuf,
    evidence: PathBuf,
}

fn metrics_regression_fixture(name: &str) -> MetricsRegressionFixture {
    let root = repo_root().join("fixtures/review-metrics-regression");
    let dir = root.join(name);
    MetricsRegressionFixture {
        root,
        registry: dir.join("registry.json"),
        fixtures: dir.join("fixtures.json"),
        guidance: dir.join("guidance.json"),
        evidence: dir.join("evidence.json"),
    }
}

fn run_metrics_regression_command(
    paths: &MetricsRegressionFixture,
) -> Result<Command, Box<dyn Error>> {
    let mut command = cairn();
    command.args([
        "review",
        "metrics-regression",
        "--registry",
        paths.registry.to_str().unwrap(),
        "--fixtures",
        paths.fixtures.to_str().unwrap(),
        "--guidance",
        paths.guidance.to_str().unwrap(),
        "--evidence",
        paths.evidence.to_str().unwrap(),
        "--root",
        paths.root.to_str().unwrap(),
    ]);
    Ok(command)
}

fn assert_report_contains_issue_codes(
    receipt: &serde_json::Value,
    report_name: &str,
    expected_codes: &[&str],
) {
    let report = receipt["reports"]
        .as_array()
        .unwrap()
        .iter()
        .find(|report| report["name"] == report_name)
        .unwrap();
    let issue_codes = report["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    for expected_code in expected_codes {
        assert!(
            issue_codes.contains(expected_code),
            "missing {expected_code} in {issue_codes:?}"
        );
    }
}
