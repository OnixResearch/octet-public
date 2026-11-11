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

fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "command failed: {}\nstdout: {}\nstderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn test_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = std::env::temp_dir().join(format!("cairn-adr-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn base_arg() -> String {
    format!("git_oid:{}{}", "b".repeat(40), "c".repeat(24))
}

#[test]
fn adr_import_accept_and_immutability() -> Result<(), Box<dyn Error>> {
    let dir = test_dir("accept")?;
    let store = dir.join("store");
    fs::create_dir_all(&store)?;
    let decision = dir.join("decision.md");
    fs::write(&decision, "# Decision: adopt X\n")?;
    let decision_s = decision.to_string_lossy().to_string();
    let store_s = store.to_string_lossy().to_string();
    let policy = worktree_root().join("cairn-policy/generated/cairn-policy.json");
    let policy = policy.to_string_lossy().to_string();
    let base = base_arg();

    let mut import = vec![
        "adr".to_string(),
        "import".to_string(),
        "--store".to_string(),
        store_s.clone(),
        "--document-id".to_string(),
        "adr-1".to_string(),
        "--base-revision".to_string(),
        base,
        "--authors".to_string(),
        "author".to_string(),
        "--reviewers".to_string(),
        "reviewer".to_string(),
        "--approvers".to_string(),
        "approver".to_string(),
        "--publishers".to_string(),
        "publisher".to_string(),
        "--decision".to_string(),
        decision_s,
        "--policy".to_string(),
        policy,
    ];
    let refs = import.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs)?;
    // execute import
    import.push("--execute".to_string());
    let refs = import.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&refs)?;

    let status = run_json(&[
        "adr",
        "status",
        "--store",
        &store_s,
        "--document-id",
        "adr-1",
    ])?;
    assert_eq!(status["kind"], "adr");
    assert_eq!(status["conflicted"], false);

    // approver accepts (preview then execute)
    let preview = run_json(&[
        "adr",
        "accept",
        "--store",
        &store_s,
        "--document-id",
        "adr-1",
        "--actor",
        "approver",
    ])?;
    assert_eq!(preview["dry_run"], true);
    let accept = run_json(&[
        "adr",
        "accept",
        "--store",
        &store_s,
        "--document-id",
        "adr-1",
        "--actor",
        "approver",
        "--execute",
    ])?;
    assert_eq!(accept["executed"], true);

    // status shows accepted
    let status = run_json(&[
        "adr",
        "status",
        "--store",
        &store_s,
        "--document-id",
        "adr-1",
    ])?;
    assert_eq!(status["lifecycle"]["status"], "accepted");

    // post-acceptance edit by author is rejected (decision frozen)
    let edit = cairn()
        .args([
            "adr",
            "update",
            "--store",
            &store_s,
            "--document-id",
            "adr-1",
            "--actor",
            "author",
            "--text",
            "# Changed decision\n",
            "--execute",
        ])
        .output()?;
    assert!(!edit.status.success());
    assert!(
        String::from_utf8_lossy(&edit.stderr).contains("immutable"),
        "expected immutable error, got {}",
        String::from_utf8_lossy(&edit.stderr)
    );
    Ok(())
}
