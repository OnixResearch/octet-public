use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

const BLAKE3_HEX_LEN: usize = 64;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn temp_project(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let destination =
        env::temp_dir().join(format!("cairn-traceability-{name}-{}", std::process::id()));
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    fs::create_dir_all(&destination)?;
    Ok(destination)
}

fn write_text(path: &Path, text: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)?;
    Ok(())
}

fn ref_marker(verb: &str, requirement_id: &str) -> String {
    format!("// r[{verb} {requirement_id}]\n")
}

fn run_traceability_json(root: &Path, extra_args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(["traceability", "coverage", "--root"]);
    command.arg(root);
    command.arg("--json");
    command.args(extra_args);
    run_json_command(&mut command)
}

fn run_tracey_json(root: &Path) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(["tracey", "coverage", "--root"]);
    command.arg(root);
    command.arg("--json");
    run_json_command(&mut command)
}

fn run_json_command(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_failing_json_command(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        !output.status.success(),
        "expected failure, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn assert_hash_field(value: &Value, field: &str) -> Result<(), Box<dyn Error>> {
    let hash = value[field].as_str().ok_or("hash field missing")?;
    assert_eq!(hash.len(), BLAKE3_HEX_LEN, "{field} should be BLAKE3 hex");
    assert!(hash.chars().all(|character| character.is_ascii_hexdigit()));
    Ok(())
}

fn write_complete_cairn_fixture(root: &Path) -> Result<(), Box<dyn Error>> {
    write_text(
        &root.join(".cairn/specs/demo/spec.md"),
        "r[cairn.demo.alpha] The system MUST keep alpha covered.\n",
    )?;
    write_text(
        &root.join("crates/demo/src/lib.rs"),
        &ref_marker("impl", "cairn.demo.alpha"),
    )?;
    Ok(())
}

#[test]
fn traceability_coverage_passes_with_receipt_fields() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.command]
    // r[verify cairn.supersede.traceability_ownership.command.maintained]
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("complete")?;
    write_complete_cairn_fixture(&root)?;

    let receipt = run_traceability_json(&root, &[])?;

    assert_eq!(receipt["command"], "traceability coverage");
    assert_eq!(receipt["profile_id"], "cairn-default");
    assert_eq!(receipt["requirements"], 1);
    assert_eq!(receipt["referenced"], 1);
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(receipt["missing"].as_array().unwrap().len(), 0);
    assert_eq!(receipt["dangling"].as_array().unwrap().len(), 0);
    assert_hash_field(&receipt, "input_hash")?;
    assert_hash_field(&receipt, "policy_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn traceability_coverage_reports_missing_requirement_refs() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("missing")?;
    write_text(
        &root.join(".cairn/specs/demo/spec.md"),
        "r[cairn.demo.alpha] The system MUST keep alpha covered.\n",
    )?;

    let mut command = cairn();
    command.args(["traceability", "coverage", "--root"]);
    command.arg(&root);
    let receipt = run_failing_json_command(&mut command)?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["missing"], serde_json::json!(["cairn.demo.alpha"]));
    assert_eq!(receipt["dangling"].as_array().unwrap().len(), 0);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn traceability_coverage_reports_dangling_refs() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("dangling")?;
    write_text(
        &root.join(".cairn/specs/demo/spec.md"),
        "r[cairn.demo.alpha] The system MUST keep alpha covered.\n",
    )?;
    write_text(
        &root.join("crates/demo/src/lib.rs"),
        &format!(
            "{}{}",
            ref_marker("impl", "cairn.demo.alpha"),
            ref_marker("related", "cairn.demo.beta")
        ),
    )?;

    let mut command = cairn();
    command.args(["traceability", "coverage", "--root"]);
    command.arg(&root);
    let receipt = run_failing_json_command(&mut command)?;

    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["missing"].as_array().unwrap().len(), 0);
    assert_eq!(receipt["dangling"], serde_json::json!(["cairn.demo.beta"]));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn tracey_alias_uses_equivalent_repo_owned_checker() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.command]
    // r[verify cairn.supersede.traceability_ownership.command.alias]
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("alias")?;
    write_complete_cairn_fixture(&root)?;

    let maintained = run_traceability_json(&root, &[])?;
    let alias = run_tracey_json(&root)?;

    for field in ["valid", "requirements", "referenced", "missing", "dangling"] {
        assert_eq!(
            maintained[field], alias[field],
            "field {field} should match"
        );
    }
    assert_eq!(alias["command"], "traceability coverage");

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn trellis_style_profile_scans_docs_src_and_tests() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.configured_profiles]
    // r[verify cairn.supersede.traceability_ownership.configured_profiles.trellis]
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("trellis")?;
    write_text(
        &root.join("docs/requirements.md"),
        "r[trellis.demo.alpha] The system MUST keep alpha covered.\n",
    )?;
    write_text(
        &root.join("src/lib.rs"),
        &ref_marker("impl", "trellis.demo.alpha"),
    )?;
    write_text(
        &root.join("tests/trace.rs"),
        &ref_marker("verify", "trellis.demo.alpha"),
    )?;

    let receipt = run_traceability_json(&root, &["--profile", "trellis-rust"])?;

    assert_eq!(receipt["profile_id"], "trellis-rust");
    assert_eq!(receipt["requirements"], 1);
    assert_eq!(receipt["referenced"], 1);
    assert_eq!(receipt["valid"], true);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn malformed_profile_policy_fails_closed_before_scan() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.configured_profiles]
    // r[verify cairn.supersede.traceability_ownership.configured_profiles.malformed]
    // r[verify cairn.supersede.traceability_ownership.validation]
    // r[verify cairn.supersede.traceability_ownership.validation.fixtures]
    let root = temp_project("malformed")?;
    write_complete_cairn_fixture(&root)?;
    let policy_path = root.join("bad-policy.json");
    let mut policy: Value = serde_json::from_str(&fs::read_to_string(
        repo_root().join("cairn-policy/generated/cairn-policy.json"),
    )?)?;
    policy["traceability_policy"]["profiles"][0]["evidence_sources"][0]["source_kind"] =
        Value::String("ambient_source".to_string());
    fs::write(&policy_path, serde_json::to_string_pretty(&policy)? + "\n")?;

    let output = cairn()
        .args(["traceability", "coverage", "--root"])
        .arg(&root)
        .args(["--policy"])
        .arg(&policy_path)
        .output()?;

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("traceability_policy.profiles.sources.source_kind"),
        "{stderr}"
    );

    fs::remove_dir_all(root)?;
    Ok(())
}
