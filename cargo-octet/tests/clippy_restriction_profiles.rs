use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

const FIXTURE_MANIFEST: &str = r#"[package]
name = "clippy-restriction-fixture"
version = "0.1.0"
edition = "2024"

[workspace]
"#;
const CLEAN_SOURCE: &str = "pub fn increment(value: u32) -> u32 { value + 1 }\n";
const VIOLATING_SOURCE: &str = "pub fn inspect(value: u32) -> u32 { dbg!(value) }\n";
const PROFILE_ID: &str = "production-safety";
const RECEIPT_NAME: &str = "evidence-rail-receipt.json";
const SUCCESS_EXIT: i32 = 0;
const RAIL_FAILURE_EXIT: i32 = 1;

struct ProfileRun {
    output: Output,
    receipt: serde_json::Value,
    artifact_dir: PathBuf,
    _temp: tempfile::TempDir,
}

#[test]
fn clean_workspace_produces_policy_bound_passing_receipt() {
    let run = run_profiled_fixture(CLEAN_SOURCE);

    assert_eq!(run.output.status.code(), Some(SUCCESS_EXIT), "stderr: {}", stderr(&run.output));
    assert_eq!(run.receipt["status"], "passed");
    assert_eq!(run.receipt["profile"], PROFILE_ID);
    assert_eq!(run.receipt["clippy_restrictions"]["profile"]["profile_id"], PROFILE_ID);
    assert_eq!(run.receipt["clippy_restrictions"]["profile"]["lint_level"], "deny");
    assert_eq!(run.receipt["config_path"], "embedded:standards/clippy-restrictions/generated/profiles.json");
    assert!(
        run.receipt["clippy_restrictions"]["profile"]["lints"]
            .as_array()
            .expect("profile lint array")
            .iter()
            .any(|lint| lint == "dbg_macro")
    );
    assert!(run.receipt["config_identity"].as_str().expect("config identity").starts_with("b3:"));
    assert!(
        run.receipt["clippy_restrictions"]["profile_identity"]
            .as_str()
            .expect("profile identity")
            .starts_with("b3:")
    );
    assert!(run.receipt["diagnostics"].as_array().expect("diagnostics").is_empty());

    let verification = Command::new(cargo_octet())
        .args(["artifact", "verify", "--artifact-dir"])
        .arg(&run.artifact_dir)
        .output()
        .expect("verify profiled Clippy artifact bundle");
    assert_eq!(
        verification.status.code(),
        Some(SUCCESS_EXIT),
        "artifact verification stderr: {}",
        stderr(&verification)
    );
}

#[test]
fn violating_workspace_preserves_lint_and_nonzero_tool_failure() {
    let run = run_profiled_fixture(VIOLATING_SOURCE);

    assert_eq!(run.output.status.code(), Some(RAIL_FAILURE_EXIT), "stderr: {}", stderr(&run.output));
    assert_eq!(run.receipt["status"], "failed");
    assert_ne!(run.receipt["tool_exit_code"].as_i64().expect("tool exit code"), i64::from(SUCCESS_EXIT));
    assert!(
        run.receipt["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .any(|finding| finding["family"] == "dbg_macro")
    );
    assert!(
        run.receipt["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .all(|diagnostic| diagnostic.as_str().is_none_or(|text| !text.contains("malformed Clippy JSON")))
    );
}

fn run_profiled_fixture(source: &str) -> ProfileRun {
    let temp = tempfile::tempdir().expect("temporary fixture root");
    let workspace = temp.path().join("workspace");
    let source_dir = workspace.join("src");
    let artifact_dir = temp.path().join("artifacts");
    let target_dir = temp.path().join("target");
    fs::create_dir_all(&source_dir).expect("create fixture source directory");
    fs::write(workspace.join("Cargo.toml"), FIXTURE_MANIFEST).expect("write fixture manifest");
    fs::write(source_dir.join("lib.rs"), source).expect("write fixture source");

    let output = Command::new(cargo_octet())
        .args([
            "evidence",
            "collect",
            "--rail",
            "clippy-co-reporting",
            "--run",
            "--profile",
            PROFILE_ID,
            "--workspace-root",
        ])
        .arg(&workspace)
        .arg("--artifact-dir")
        .arg(&artifact_dir)
        .args(["--output-format", "json"])
        .env("CARGO_TARGET_DIR", target_dir)
        .output()
        .expect("run profiled Clippy evidence rail");
    let receipt_text = fs::read_to_string(artifact_dir.join(RECEIPT_NAME)).unwrap_or_else(|error| {
        panic!("read Clippy receipt after status {:?}: {error}; stderr: {}", output.status.code(), stderr(&output))
    });
    let receipt = serde_json::from_str(&receipt_text).expect("parse Clippy receipt");
    ProfileRun {
        output,
        receipt,
        artifact_dir,
        _temp: temp,
    }
}

fn cargo_octet() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-octet")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
