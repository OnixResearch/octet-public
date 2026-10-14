use super::*;

const SOURCE_IDENTITY: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TOOL_FAILURE_EXIT: i32 = 2;
const OWNER_ONLY_PERMISSIONS: u32 = 0o700;

#[test]
fn profile_registry_and_identity_are_fresh() {
    // r[verify octet.mutation_testing.profile]
    assert_eq!(prefixed_identity(PROFILE_JSON.as_bytes()), PROFILE_IDENTITY);
    let profile = selected_profile(PROFILE_ID).expect("mutation profile");
    assert_eq!(profile.provider.version, "27.0.0");
    assert!(profile.command_argv.contains(&"--no-times".to_owned()));
    assert!(profile.command_argv.contains(&"--output".to_owned()));
}

#[test]
fn caught_and_missed_outcomes_preserve_complete_inventory() {
    // r[verify octet.mutation_testing.receipt]
    // r[verify octet.mutation_testing.baseline]
    let caught = evaluate_request(&request("CaughtMutant", true), PROFILE_ID);
    let details = caught.details.expect("caught mutation details");
    assert!(caught.diagnostics.is_empty());
    assert_eq!(details.gate_status, "passed");
    assert_eq!(details.outcome_counts.get("caught"), Some(&1));
    assert_eq!(details.mutants[0].source_identity, SOURCE_IDENTITY);
    assert!(details.mutants[0].diff.is_some());

    let missed = evaluate_request(&request("MissedMutant", true), PROFILE_ID);
    assert_eq!(missed.details.as_ref().expect("missed details").gate_status, "failed");
    assert!(missed.diagnostics.iter().any(|value| value.contains("failed outcome")));
}

#[test]
fn failed_baseline_incomplete_inventory_and_stale_artifacts_fail_closed() {
    // r[verify octet.mutation_testing.verification]
    let failed_baseline = evaluate_request(&request("CaughtMutant", false), PROFILE_ID);
    assert!(failed_baseline.details.is_some());
    assert!(failed_baseline.diagnostics.iter().any(|value| value.contains("baseline")));

    let mut incomplete: serde_json::Value =
        serde_json::from_str(&request("CaughtMutant", true)).expect("parse request");
    incomplete["outcomes_text"] = serde_json::Value::String(outcomes_without_mutant());
    let incomplete = serde_json::to_string(&incomplete).expect("serialize incomplete request");
    assert!(evaluate_request(&incomplete, PROFILE_ID).details.is_none());

    let valid = request("CaughtMutant", true);
    let details = evaluate_request(&valid, PROFILE_ID).details.expect("valid details");
    let mut stale: serde_json::Value = serde_json::from_str(&valid).expect("parse valid request");
    stale["artifacts"]["log/mutant.log"] = serde_json::Value::String("stale".to_owned());
    let stale = serde_json::to_string(&stale).expect("serialize stale request");
    assert!(request::artifact_payloads(&details, &stale).is_err());
}

#[cfg(unix)]
#[test]
fn bounded_native_shell_collects_declared_machine_artifacts() {
    // r[verify octet.mutation_testing.native_artifacts]
    // r[verify octet.mutation_testing.boundary]
    use std::os::unix::fs::PermissionsExt as _;

    let directory = tempfile::tempdir().expect("create mutation shell fixture");
    let script = directory.path().join("cargo-fixture");
    let config = directory.path().join("cargo-config.toml");
    std::fs::write(&config, "[net]\noffline = true\n").expect("write mutation Cargo config");
    let script_text = format!(
        "#!/bin/sh\nprintf '%s' '{}' > .octet/mutants-output/mutants.out/mutants.json\nprintf '%s' '{}' > .octet/mutants-output/mutants.out/outcomes.json\nprintf '%s' 'baseline log' > .octet/mutants-output/mutants.out/log/baseline.log\nprintf '%s' 'mutant log' > .octet/mutants-output/mutants.out/log/mutant.log\nprintf '%s' 'mutant diff' > .octet/mutants-output/mutants.out/diff/mutant.diff\nexit {TOOL_FAILURE_EXIT}\n",
        inventory(),
        outcomes("MissedMutant", true)
    );
    std::fs::write(&script, script_text).expect("write mutation executable fixture");
    let mut permissions = std::fs::metadata(&script).expect("inspect mutation executable").permissions();
    permissions.set_mode(OWNER_ONLY_PERMISSIONS);
    std::fs::set_permissions(&script, permissions).expect("make mutation fixture executable");

    let profile = selected_profile(PROFILE_ID).expect("mutation profile");
    let observation = shell::run_with(directory.path(), &profile, source_manifest(), shell::RuntimeTools {
        cargo: script,
        path: "/bin".into(),
        cargo_config: config,
        library_path: "/lib".into(),
    })
    .expect("run bounded mutation fixture");
    assert_eq!(observation.exit_code, TOOL_FAILURE_EXIT);
    assert_eq!(observation.parts.artifacts.len(), 3);
    let rendered = request::render(observation.parts).expect("render mutation run input");
    assert_eq!(request::evaluate(&rendered, PROFILE_ID).details.expect("run details").gate_status, "failed");
}

fn evaluate_request(text: &str, profile_id: &str) -> Evaluation {
    request::evaluate(text, profile_id)
}

fn request(outcome: &str, baseline_passed: bool) -> String {
    let mut artifacts = std::collections::BTreeMap::new();
    artifacts.insert("log/baseline.log".to_owned(), "baseline log".to_owned());
    artifacts.insert("log/mutant.log".to_owned(), "mutant log".to_owned());
    artifacts.insert("diff/mutant.diff".to_owned(), "mutant diff".to_owned());
    request::render(request::InputParts {
        outcomes_text: outcomes(outcome, baseline_passed),
        inventory_text: inventory(),
        source_manifest_text: source_manifest(),
        artifacts,
        bounded_log_identity: prefixed_identity(b""),
    })
    .expect("render mutation request")
}

fn source_manifest() -> String {
    serde_json::json!({
        "schema_version": "octet-source-manifest/v1",
        "files": [{"path": "src/lib.rs", "blake3": SOURCE_IDENTITY}]
    })
    .to_string()
}

fn inventory() -> String {
    serde_json::json!([{
        "name": "src/lib.rs:1:1: replace value",
        "package": "octet-lint-metadata",
        "file": "src/lib.rs",
        "replacement": "false",
        "genre": "FnValue",
        "span": {"start": {"line": 1, "column": 1}}
    }])
    .to_string()
}

fn outcomes(outcome: &str, baseline_passed: bool) -> String {
    let baseline = if baseline_passed { "Success" } else { "Failure" };
    serde_json::json!({"outcomes": [
        {"scenario": "Baseline", "summary": baseline, "log_path": "log/baseline.log", "diff_path": null},
        {"scenario": {"Mutant": {"name": "src/lib.rs:1:1: replace value"}}, "summary": outcome,
         "log_path": "log/mutant.log", "diff_path": "diff/mutant.diff"}
    ]})
    .to_string()
}

fn outcomes_without_mutant() -> String {
    serde_json::json!({"outcomes": [
        {"scenario": "Baseline", "summary": "Success", "log_path": "log/baseline.log", "diff_path": null}
    ]})
    .to_string()
}
