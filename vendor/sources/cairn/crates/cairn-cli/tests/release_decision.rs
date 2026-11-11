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
    let dir = std::env::temp_dir().join(format!("cairn-rd-{name}-{}", std::process::id()));
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
fn release_decision_create_status_and_approve() -> Result<(), Box<dyn Error>> {
    let dir = test_dir("approve")?;
    let store = dir.join("store");
    fs::create_dir_all(&store)?;
    let store_s = store.to_string_lossy().to_string();
    let base = base_arg();
    let policy = policy_arg();

    let mut create = cairn();
    create.args([
        "release-decision",
        "create",
        "--store",
        store_s.as_str(),
        "--decision-id",
        "rd-1",
        "--base-revision",
        base.as_str(),
        "--source-revision",
        "src-1",
        "--artifact-manifest",
        "manifest-1",
        "--artifact-identity",
        "artifact-1",
        "--target-label",
        "x86_64-linux",
        "--release-label",
        "stable",
        "--proposers",
        "proposer",
        "--reviewers",
        "reviewer",
        "--approvers",
        "approver",
        "--withdrawers",
        "withdrawer",
        "--invalidators",
        "invalidator",
        "--policy",
        policy.as_str(),
        "--execute",
    ]);
    run_json(&mut create)?;

    let mut status = cairn();
    status.args([
        "release-decision",
        "status",
        "--store",
        store_s.as_str(),
        "--decision-id",
        "rd-1",
    ]);
    let status = run_json(&mut status)?;
    assert_eq!(status["candidate_root"].as_str().map(|s| s.len()), Some(64));
    assert_eq!(status["lifecycle"], "draft");

    let mut vote = cairn();
    vote.args([
        "release-decision",
        "vote",
        "--store",
        store_s.as_str(),
        "--decision-id",
        "rd-1",
        "--actor",
        "approver",
        "--kind",
        "approve",
        "--execute",
    ]);
    let vote = run_json(&mut vote)?;
    assert_eq!(vote["verdict"], "approved");

    let mut reject = cairn();
    reject.args([
        "release-decision",
        "vote",
        "--store",
        store_s.as_str(),
        "--decision-id",
        "rd-1",
        "--actor",
        "reviewer",
        "--kind",
        "reject",
        "--execute",
    ]);
    let out = reject.output()?;
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("conflict")
            || String::from_utf8_lossy(&out.stderr).contains("stale")
    );
    Ok(())
}
