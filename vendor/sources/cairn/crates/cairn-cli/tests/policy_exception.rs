use std::{error::Error, fs, path::PathBuf, process::Command};

use serde_json::Value;

fn worktree_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(worktree_root())
        .env_remove("CAIRN_GLOBAL_REGISTRY");
    command
}

fn run_json(cmd: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!(
            "command failed: stderr={}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn test_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("cairn-pe-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn base_arg() -> String {
    format!("git_oid:{}{}", "b".repeat(40), "c".repeat(24))
}

fn policy_arg() -> String {
    worktree_root()
        .join("cairn-policy/generated/cairn-policy.json")
        .to_string_lossy()
        .to_string()
}

#[test]
fn policy_exception_request_status_and_validate() -> Result<(), Box<dyn Error>> {
    let dir = test_dir("request")?;
    let store = dir.join("store");
    fs::create_dir_all(&store)?;
    let store_s = store.to_string_lossy().to_string();
    let base = base_arg();
    let policy = policy_arg();

    let mut request = cairn();
    request.args([
        "policy-exception",
        "request",
        "--store",
        &store_s,
        "--exception-id",
        "pe-1",
        "--base-revision",
        &base,
        "--rules",
        "allow-network-egress",
        "--targets",
        "svc:bar",
        "--rationale",
        "temporary migration",
        "--controls",
        "audit-log",
        "--requesters",
        "requester",
        "--reviewers",
        "reviewer",
        "--approvers",
        "approver",
        "--revokers",
        "revoker",
        "--observers",
        "observer",
        "--policy",
        &policy,
        "--execute",
    ]);
    let requested = run_json(&mut request)?;
    assert_eq!(
        requested["request_root"].as_str().map(|s| s.len()),
        Some(64)
    );

    let mut status = cairn();
    status.args([
        "policy-exception",
        "status",
        "--store",
        &store_s,
        "--exception-id",
        "pe-1",
    ]);
    let status = run_json(&mut status)?;
    assert_eq!(status["lifecycle"], "requested");

    // observe validity by the observer
    let mut observe = cairn();
    observe.args([
        "policy-exception",
        "observe-validity",
        "--store",
        &store_s,
        "--exception-id",
        "pe-1",
        "--actor",
        "observer",
        "--observed-epoch",
        "100",
        "--finite-valid-until",
        "1000",
        "--execute",
    ]);
    run_json(&mut observe)?;

    let mut status2 = cairn();
    status2.args([
        "policy-exception",
        "status",
        "--store",
        &store_s,
        "--exception-id",
        "pe-1",
    ]);
    let status2 = run_json(&mut status2)?;
    assert_eq!(status2["applicability"], "not-approved");
    Ok(())
}
