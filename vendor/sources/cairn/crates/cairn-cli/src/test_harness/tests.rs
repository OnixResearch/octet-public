use super::{
    artifacts::{artifact_schema, ARTIFACT_SCHEMA},
    cases::case_report,
    model::{Class, Outcome},
    rails::{rail_args, rail_output_report, real_rail_report},
};
use std::fs;

fn root(n: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("cairn-harness-{n}-{}", std::process::id()))
}

#[test]
fn pass_case() {
    let r = root("pass");
    let a = r.join("a.json");
    let report = case_report(&r, "pass", "1", 1, &a).expect("case");
    assert_eq!(report.outcome, Outcome::Passed);
    assert_eq!(
        artifact_schema(&fs::read(&a).expect("artifact")),
        ARTIFACT_SCHEMA
    );
    fs::remove_dir_all(r).ok();
}

#[test]
fn product_class() {
    let r = root("prod");
    let a = r.join("a.json");
    let report = case_report(&r, "product-fail", "1", 1, &a).expect("case");
    assert_eq!(report.class, Class::Product);
    fs::remove_dir_all(r).ok();
}

#[test]
fn corrupt_artifact() {
    let r = root("harness");
    let a = r.join("a.json");
    let report = case_report(&r, "harness-fail", "1", 1, &a).expect("case");
    assert_eq!(report.class, Class::Harness);
    assert_eq!(artifact_schema(&fs::read(&a).expect("artifact")), "invalid");
    fs::remove_dir_all(r).ok();
}

#[test]
fn known_rail_args_are_bounded() {
    assert_eq!(
        rail_args("validate").expect("validate"),
        vec!["validate", "--root", "."]
    );
    assert_eq!(
        rail_args("traceability-coverage").expect("traceability"),
        vec!["traceability", "coverage", "--root", "."]
    );
    assert_eq!(
        rail_args("tracey-coverage").expect("tracey"),
        vec!["traceability", "coverage", "--root", "."]
    );
    assert!(rail_args("arbitrary-shell").is_none());
}

#[test]
fn rail_output_classifies_product_failure() {
    let args = rail_args("validate").expect("validate");
    let report = rail_output_report("validate", &args, Some(2), b"bad", b"err");
    assert_eq!(report.outcome, Outcome::Failed);
    assert_eq!(report.class, Class::Product);
    assert_eq!(report.code, "rail_failed");
}

#[test]
fn unknown_rail_is_harness_failure() {
    let r = root("unknown-rail");
    let report = real_rail_report(&r, "not-a-rail");
    assert_eq!(report.class, Class::Harness);
    assert_eq!(report.code, "unknown_harness_rail");
    fs::remove_dir_all(r).ok();
}

#[test]
fn flake_not_green() {
    let r = root("flake");
    let a = r.join("a.json");
    let report = case_report(&r, "timeout-flake", "1", 2, &a).expect("case");
    assert_eq!(report.outcome, Outcome::Flaky);
    assert_eq!(report.attempts.len(), 3);
    fs::remove_dir_all(r).ok();
}
