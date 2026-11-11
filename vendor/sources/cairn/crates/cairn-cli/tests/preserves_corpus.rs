use std::{env, error::Error, fs, path::PathBuf, process::Command};

use serde_json::{json, Value};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn temp_file(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = env::temp_dir().join(format!(
        "cairn-preserves-corpus-{name}-{}.json",
        std::process::id()
    ));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(path)
}

fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_json_failure(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn checked_in_corpus_text() -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(
        repo_root().join("fixtures/preserves-canonicalization/corpus.json"),
    )?)
}

#[test]
// r[verify cairn.determinism.preserves_corpus_explain_cli.command]
// r[verify cairn.determinism.preserves_corpus_explain_cli.read_only]
// r[verify cairn.determinism.preserves_corpus_explain_cli.tests]
// r[verify cairn.determinism.preserves_corpus_explain_cli.validation]
fn preserves_corpus_explain_reports_cases_without_values() -> Result<(), Box<dyn Error>> {
    let path = temp_file("explain-clean")?;
    fs::write(&path, checked_in_corpus_text()?)?;
    let before = fs::read_to_string(&path)?;

    let receipt = run_json(&[
        "preserves-corpus",
        "explain",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    assert_eq!(receipt["command"], "preserves-corpus explain");
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["mutated"], false);
    assert_eq!(receipt["case_count"], 7);
    assert_eq!(receipt["cases"][0]["id"], "null_value");
    assert_eq!(receipt["cases"][0]["description"], "JSON null primitive");
    assert_eq!(receipt["cases"][0]["value_redacted"], true);
    assert!(receipt["cases"][0].get("value").is_none());
    assert_eq!(receipt["cases"][0]["expected_hash_present"], true);
    assert_eq!(receipt["cases"][0]["expected_hash_shape"], "hex64");
    assert_eq!(fs::read_to_string(&path)?, before);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_explain_cli.tests]
fn preserves_corpus_explain_rejects_malformed_cases_without_mutating() -> Result<(), Box<dyn Error>>
{
    let path = temp_file("explain-malformed")?;
    let text = serde_json::to_string_pretty(&json!([
        { "id": "missing_value", "description": "bad", "expected_hash": "abc" }
    ]))?;
    fs::write(&path, &text)?;

    let receipt = run_json_failure(&[
        "preserves-corpus",
        "explain",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    let issue_codes: Vec<_> = receipt["issues"]
        .as_array()
        .ok_or("issues array")?
        .iter()
        .filter_map(|issue| issue["code"].as_str())
        .collect();
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["mutated"], false);
    assert!(issue_codes.contains(&"preserves_corpus.case_missing_value"));
    assert!(issue_codes.contains(&"preserves_corpus.expected_hash_malformed"));
    assert_eq!(fs::read_to_string(&path)?, text);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_explain_cli.tests]
fn preserves_corpus_explain_reports_duplicate_case_ids() -> Result<(), Box<dyn Error>> {
    let path = temp_file("explain-duplicate")?;
    let text = serde_json::to_string_pretty(&json!([
        {
            "id": "duplicate",
            "description": "first",
            "value": null,
            "expected_hash": "4111d2d01cd6f342ffbf86955fa72cd2af2e4cbb38d4db96bb3c28f77eb9661a"
        },
        {
            "id": "duplicate",
            "description": "second",
            "value": true,
            "expected_hash": "51d26a6b9bfaec0d66accd6f887d2dd4933b4f0a8f5a2bbba55ea4d60949be22"
        }
    ]))?;
    fs::write(&path, &text)?;

    let receipt = run_json_failure(&[
        "preserves-corpus",
        "explain",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["case_count"], 2);
    assert_eq!(
        receipt["issues"][0]["code"],
        "preserves_corpus.duplicate_case_id"
    );
    assert_eq!(receipt["issues"][0]["case_id"], "duplicate");
    assert_eq!(fs::read_to_string(&path)?, text);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_update_rail.validation]
// r[verify cairn.determinism.preserves_workflow_test_split.corpus_focus]
// r[verify cairn.determinism.preserves_corpus_ci_guard]
// r[verify cairn.determinism.preserves_corpus_ci_guard.clean_check]
// r[verify cairn.determinism.preserves_corpus_ci_guard.validation]
fn preserves_corpus_update_reports_clean_checked_in_corpus() -> Result<(), Box<dyn Error>> {
    let path = temp_file("clean")?;
    fs::write(&path, checked_in_corpus_text()?)?;
    let receipt = run_json(&[
        "preserves-corpus",
        "update",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    assert_eq!(receipt["command"], "preserves-corpus update");
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["execute"], false);
    assert_eq!(receipt["mutated"], false);
    assert_eq!(receipt["case_count"], 7);
    assert_eq!(receipt["stale_count"], 0);
    assert_eq!(receipt["issues"].as_array().ok_or("issues array")?.len(), 0);
    assert_eq!(fs::read_to_string(&path)?, checked_in_corpus_text()?);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_update_rail.validation]
// r[verify cairn.determinism.preserves_corpus_ci_guard.stale_detection]
fn preserves_corpus_update_dry_run_detects_stale_without_mutating() -> Result<(), Box<dyn Error>> {
    let path = temp_file("stale-dry-run")?;
    let first_hash = "4111d2d01cd6f342ffbf86955fa72cd2af2e4cbb38d4db96bb3c28f77eb9661a";
    let stale_text = checked_in_corpus_text()?.replacen(
        first_hash,
        "0000000000000000000000000000000000000000000000000000000000000000",
        1,
    );
    fs::write(&path, &stale_text)?;

    let receipt = run_json_failure(&[
        "preserves-corpus",
        "update",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["execute"], false);
    assert_eq!(receipt["mutated"], false);
    assert_eq!(receipt["stale_count"], 1);
    assert_eq!(receipt["cases"][0]["id"], "null_value");
    assert_eq!(receipt["cases"][0]["stale"], true);
    assert_eq!(fs::read_to_string(&path)?, stale_text);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_update_rail.validation]
fn preserves_corpus_update_execute_repairs_stale_hashes() -> Result<(), Box<dyn Error>> {
    let path = temp_file("execute")?;
    let first_hash = "4111d2d01cd6f342ffbf86955fa72cd2af2e4cbb38d4db96bb3c28f77eb9661a";
    let stale_text = checked_in_corpus_text()?.replacen(
        first_hash,
        "0000000000000000000000000000000000000000000000000000000000000000",
        1,
    );
    fs::write(&path, stale_text)?;

    let receipt = run_json(&[
        "preserves-corpus",
        "update",
        "--path",
        path.to_str().ok_or("temp path")?,
        "--execute",
    ])?;

    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["execute"], true);
    assert_eq!(receipt["mutated"], true);
    assert_eq!(receipt["stale_count"], 1);

    let clean = run_json(&[
        "preserves-corpus",
        "update",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;
    assert_eq!(clean["valid"], true);
    assert_eq!(clean["stale_count"], 0);
    assert_eq!(fs::read_to_string(&path)?, checked_in_corpus_text()?);
    fs::remove_file(path)?;
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_corpus_update_rail.validation]
fn preserves_corpus_update_rejects_malformed_cases() -> Result<(), Box<dyn Error>> {
    let path = temp_file("malformed")?;
    fs::write(
        &path,
        serde_json::to_string_pretty(&json!([{ "id": "missing_value", "expected_hash": "abc" }]))?,
    )?;

    let receipt = run_json_failure(&[
        "preserves-corpus",
        "update",
        "--path",
        path.to_str().ok_or("temp path")?,
    ])?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(
        receipt["issues"][0]["code"],
        "preserves_corpus.case_missing_value"
    );
    assert_eq!(receipt["mutated"], false);
    fs::remove_file(path)?;
    Ok(())
}
