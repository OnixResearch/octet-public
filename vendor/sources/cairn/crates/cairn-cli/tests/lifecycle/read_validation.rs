use std::{error::Error, fs};

use super::support::{assert_matches_golden, run_json, temp_fixture, temp_project};

#[test]
fn status_matches_canonical_golden() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.rename_cairn] r[verify cairn.supersede.rename_cairn.packaging]
    // r[verify cairn.supersede.repo_native_labels] r[verify cairn.supersede.repo_native_labels.check]
    assert_matches_golden(&["status", "--root", "fixtures/basic"], "status-basic.json")
}

#[test]
fn validate_matches_canonical_golden() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.boundary.pure_core]
    // r[verify cairn.supersede.tracey_coverage] r[verify cairn.supersede.tracey_coverage.config]
    // r[verify cairn.supersede.tracey_coverage.check] r[verify cairn.supersede.tracey_coverage.boundary]
    assert_matches_golden(
        &["validate", "--root", "fixtures/basic"],
        "validate-basic.json",
    )
}

#[test]
fn doctor_reports_healthy_fixture() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.doctor]
    // r[verify cairn.supersede.repo_native_only] r[verify cairn.supersede.repo_native_only.doctor]
    let value = run_json(&["doctor", "--root", "fixtures/basic"])?;
    assert_eq!(value["valid"], true);
    assert_eq!(value["layout"], "cairn");
    assert_eq!(value["spec_count"], 1);
    assert_eq!(value["change_count"], 1);
    assert_eq!(value["validation_issue_count"], 0);
    let checks = value["checks"].as_array().ok_or("checks")?;
    assert!(checks
        .iter()
        .any(|check| check["name"] == "policy" && check["status"] == "ok"));
    Ok(())
}

#[test]
fn doctor_reports_missing_policy_without_process_failure() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.doctor]
    let value = run_json(&[
        "doctor",
        "--root",
        "fixtures/basic",
        "--policy",
        "fixtures/basic/missing-policy.json",
    ])?;
    assert_eq!(value["valid"], false);
    let checks = value["checks"].as_array().ok_or("checks")?;
    assert!(checks
        .iter()
        .any(|check| check["name"] == "policy" && check["status"] == "fail"));
    Ok(())
}

#[test]
fn read_outputs_match_canonical_goldens() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.read_parity] r[verify cairn.supersede.read_parity.change_list]
    // r[verify cairn.supersede.read_parity.change_show] r[verify cairn.supersede.read_parity.spec_list]
    // r[verify cairn.supersede.read_parity.spec_show] r[verify cairn.supersede.boundary]
    // r[verify cairn.supersede.boundary.read_models]
    assert_matches_golden(
        &["change", "list", "--root", "fixtures/basic"],
        "change-list-basic.json",
    )?;
    assert_matches_golden(
        &["change", "show", "demo", "--root", "fixtures/basic"],
        "change-show-basic-demo.json",
    )?;
    assert_matches_golden(
        &["spec", "list", "--root", "fixtures/basic"],
        "spec-list-basic.json",
    )?;
    assert_matches_golden(
        &["spec", "show", "demo", "--root", "fixtures/basic"],
        "spec-show-basic-demo.json",
    )?;
    Ok(())
}

#[test]
fn legacy_openspec_is_not_runtime_fallback() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.native_layout.native_only]
    let fixture = temp_fixture("legacy-no-fallback", "fixtures/basic")?;
    fs::remove_dir_all(fixture.join(".cairn"))?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;

    let status = run_json(&["status", "--root", root])?;
    assert_eq!(status["layout"], "cairn");
    assert!(status["specs"].as_array().ok_or("specs")?.is_empty());
    assert!(status["changes"].as_array().ok_or("changes")?.is_empty());

    let doctor = run_json(&["doctor", "--root", root])?;
    assert_eq!(doctor["layout"], "cairn");
    assert_eq!(doctor["valid"], false);
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn status_reports_fixture_change_counts() -> Result<(), Box<dyn Error>> {
    let value = run_json(&["status", "--root", "fixtures/basic"])?;
    assert_eq!(value["root"], "fixtures/basic");
    assert_eq!(value["specs"].as_array().ok_or("specs array")?.len(), 1);
    let changes = value["changes"].as_array().ok_or("changes array")?;
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0]["name"], "demo");
    assert_eq!(changes[0]["has_proposal"], true);
    assert_eq!(changes[0]["has_design"], true);
    assert_eq!(changes[0]["has_tasks"], true);
    assert_eq!(changes[0]["task_counts"]["total"], 2);
    assert_eq!(changes[0]["task_counts"]["unmarked"], 0);
    Ok(())
}

#[test]
fn validate_accepts_fixture_repo() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.validate_ears] r[verify cairn.supersede.validate_ears.accept]
    // r[verify cairn.supersede.validate_ears.boundary]
    let value = run_json(&["validate", "--root", "fixtures/basic"])?;
    assert_eq!(value["valid"], true);
    assert_eq!(value["changes"], 1);
    assert_eq!(value["policy"], "cairn-default");
    assert!(value["issues"].as_array().ok_or("issues array")?.is_empty());
    assert!(value["spec_issues"]
        .as_array()
        .ok_or("spec issues array")?
        .is_empty());
    assert_eq!(value["specs_validated"], 2);
    assert_eq!(
        value["spec_substance"]
            .as_array()
            .ok_or("spec substance")?
            .len(),
        2
    );
    let task_inventory = value["substance"]
        .as_array()
        .ok_or("substance")?
        .iter()
        .find(|inventory| inventory["kind"] == "tasks")
        .ok_or("task inventory")?;
    assert_eq!(task_inventory["substantive_tasks"], 2);
    assert_eq!(task_inventory["task_todo"], 2);
    Ok(())
}

#[test]
fn validate_rejects_weak_ears_style_specs() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.validate_ears.weak_requirement]
    // r[verify cairn.supersede.validate_ears.incomplete_scenario]
    // r[verify cairn.lifecycle_validation.spec_substance.borrowed_strength]
    // r[verify cairn.lifecycle_validation.spec_substance.identity]
    let root = temp_project("weak-ears")?;
    let spec_dir = root.join(".cairn/specs/weak");
    fs::create_dir_all(&spec_dir)?;
    fs::write(
        spec_dir.join("spec.md"),
        "# Weak Spec\n\n## Requirements\n\n### Requirement: Weak prose\nThis feature works.\n\n#### Scenario: Missing when\n- GIVEN a precondition\n- THEN an outcome occurs\n",
    )?;
    let root_text = root.to_str().ok_or("utf8 temp root")?;
    let value = run_json(&["validate", "--root", root_text])?;
    assert_eq!(value["valid"], false);
    let issues = value["spec_issues"].as_array().ok_or("spec issues")?;
    assert_eq!(issues.len(), 4);
    for expected in [
        "Requirement: Weak prose missing requirement strength",
        "Scenario: Missing when missing WHEN clause",
        "lifecycle_validation.spec_substance.identity",
        "lifecycle_validation.spec_substance.minimum_requirements",
    ] {
        assert!(
            issues
                .iter()
                .any(|issue| issue.as_str().unwrap_or_default().contains(expected)),
            "missing {expected}: {issues:?}"
        );
    }
    fs::remove_dir_all(root)?;
    Ok(())
}
