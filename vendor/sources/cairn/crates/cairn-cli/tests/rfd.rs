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
    let dir = std::env::temp_dir().join(format!("cairn-rfd-{name}-{}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

const AUTHOR: &str = "author";
const PARTICIPANT: &str = "participant";
const REVIEWER: &str = "reviewer";
const PUBLISHER: &str = "publisher";

fn policy_arg() -> String {
    worktree_root()
        .join("cairn-policy/generated/cairn-policy.json")
        .to_string_lossy()
        .to_string()
}

fn import_identity() -> Vec<String> {
    let base = format!("git_oid:{}{}", "b".repeat(40), "c".repeat(24));
    vec![
        "--base-revision".to_string(),
        base,
        "--authors".to_string(),
        AUTHOR.to_string(),
        "--participants".to_string(),
        PARTICIPANT.to_string(),
        "--reviewers".to_string(),
        REVIEWER.to_string(),
        "--approvers".to_string(),
        "approver".to_string(),
        "--publishers".to_string(),
        PUBLISHER.to_string(),
        "--policy".to_string(),
        policy_arg(),
    ]
}

#[test]
fn rfd_import_status_update_publish_are_preview_first() -> Result<(), Box<dyn Error>> {
    let dir = test_dir("preview")?;
    let store = dir.join("store");
    fs::create_dir_all(&store)?;
    let spec = dir.join("spec.md");
    fs::write(&spec, "# Initial RFD\n")?;
    let spec_s = spec.to_string_lossy().to_string();
    let store_s = store.to_string_lossy().to_string();

    let mut import_args = vec![
        "rfd".to_string(),
        "import".to_string(),
        "--store".to_string(),
        store_s.clone(),
        "--document-id".to_string(),
        "rfd-1".to_string(),
        "--spec".to_string(),
        spec_s.clone(),
    ];
    import_args.extend(import_identity());
    let arg_refs = import_args.iter().map(String::as_str).collect::<Vec<_>>();
    let imported = run_json(&arg_refs)?;
    assert_eq!(imported["dry_run"], true);
    assert_eq!(imported["executed"], false);
    assert!(!store.join("rfd-1.portable.json").exists());

    // execute import
    import_args.push("--execute".to_string());
    let arg_refs = import_args.iter().map(String::as_str).collect::<Vec<_>>();
    let imported = run_json(&arg_refs)?;
    assert_eq!(imported["executed"], true);
    assert!(store.join("rfd-1.portable.json").exists());

    // status
    let status = run_json(&[
        "rfd",
        "status",
        "--store",
        &store_s,
        "--document-id",
        "rfd-1",
    ])?;
    assert_eq!(status["kind"], "rfd");
    assert_eq!(status["conflicted"], false);

    // single artifact update (author)
    let update = run_json(&[
        "rfd",
        "update",
        "--store",
        &store_s,
        "--document-id",
        "rfd-1",
        "--actor",
        AUTHOR,
        "--kind",
        "artifact",
        "--text",
        "# Updated RFD\n",
        "--execute",
    ])?;
    assert_eq!(update["executed"], true);

    // publish plan succeeds conflict-free
    let out_dir = dir.join("publish");
    let out_s = out_dir.to_string_lossy().to_string();
    let publish = run_json(&[
        "rfd",
        "publish",
        "--store",
        &store_s,
        "--document-id",
        "rfd-1",
        "--out",
        &out_s,
    ])?;
    assert_eq!(publish["files"].as_array().map(Vec::len), Some(1));
    Ok(())
}

#[test]
fn rfd_number_plan_binds_registry_and_publisher() -> Result<(), Box<dyn Error>> {
    let dir = test_dir("number")?;
    let store = dir.join("store");
    fs::create_dir_all(&store)?;
    let spec = dir.join("spec.md");
    fs::write(&spec, "# RFD\n")?;
    let spec_s = spec.to_string_lossy().to_string();
    let store_s = store.to_string_lossy().to_string();
    let mut import_args = vec![
        "rfd".to_string(),
        "import".to_string(),
        "--store".to_string(),
        store_s.clone(),
        "--document-id".to_string(),
        "rfd-2".to_string(),
        "--spec".to_string(),
        spec_s,
        "--execute".to_string(),
    ];
    import_args.extend(import_identity());
    let arg_refs = import_args.iter().map(String::as_str).collect::<Vec<_>>();
    run_json(&arg_refs)?;
    let registry = "c".repeat(64);
    let number = run_json(&[
        "rfd",
        "number",
        "--store",
        &store_s,
        "--document-id",
        "rfd-2",
        "--number",
        "42",
        "--registry",
        &registry,
        "--publisher",
        PUBLISHER,
    ])?;
    assert_eq!(number["publication"]["number"], 42);
    assert_eq!(number["publication"]["publisher"], PUBLISHER);
    Ok(())
}
