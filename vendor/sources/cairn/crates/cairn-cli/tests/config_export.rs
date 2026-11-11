use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;

const STORE_SOURCE_PATH: &str = ".cairn-store/store.ncl";
const STORE_PROJECTION_PATH: &str = ".cairn-store/store.json";
const PROJECT_SOURCE_PATH: &str = ".cairn/project.ncl";
const PROJECT_PROJECTION_PATH: &str = ".cairn/project.json";
const STORE_SOURCE: &str = r#"{
  schema = "cairn.lifecycle-store-source.v1",
  store_id = "shared-planning",
  remote_hint = "https://example.invalid/shared-planning.git",
  non_claims = [
    "store identity proves only the observed checked projection and native lifecycle identity",
    "store identity does not prove remote freshness, ownership, trust, or VCS synchronization",
    "store identity does not prove requirement truth, implementation, lifecycle readiness, or release eligibility",
  ],
}
"#;
const PROJECT_SOURCE: &str = r#"{
  schema = "cairn.lifecycle-store-project-source.v1",
  lifecycle_store = {
    store_id = "shared-planning",
    remote_hint = null,
  },
  references = [
    {
      store_id = "reference-store",
      remote_hint = "https://example.invalid/reference-store.git",
    },
  ],
  non_claims = [
    "a project pointer selects one declared store but does not grant lifecycle authority",
    "a direct reference provides read-only discovery and does not satisfy dependencies, gates, evidence, or requirements",
    "reference indexing does not prove truth, freshness, ownership, implementation, or satisfaction",
    "project configuration does not authorize transitive reference traversal or transport",
    "project facts and advisory guidance do not override generated policy or command authorization",
    "project context checks do not prove truth, secrecy, semantic consistency, agent compliance, or release eligibility",
  ],
}
"#;

fn test_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = env::temp_dir().join(format!("cairn-config-export-{name}-{}", std::process::id()));
    remove_tree(&root)?;
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn remove_tree(path: &Path) -> Result<(), Box<dyn Error>> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn config_output(root: &Path, kind: &str, flags: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_cairn"))
        .args(["config", "export", "--kind", kind, "--root"])
        .arg(root)
        .args(flags)
        .output()?)
}

fn run_config(root: &Path, kind: &str, flags: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = config_output(root, kind, flags)?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn write_source(root: &Path, relative: &str, source: &str) -> Result<(), Box<dyn Error>> {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().ok_or("source path requires a parent")?)?;
    fs::write(path, source)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.identity.valid]
fn store_export_is_preview_first_and_deterministic() -> Result<(), Box<dyn Error>> {
    let root = test_root("store-smoke")?;
    write_source(&root, STORE_SOURCE_PATH, STORE_SOURCE)?;

    let preview = run_config(&root, "store", &[])?;
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!root.join(STORE_PROJECTION_PATH).exists());

    let executed = run_config(&root, "store", &["--execute"])?;
    assert_eq!(executed["mutated"], true);
    let first = fs::read(root.join(STORE_PROJECTION_PATH))?;
    let projection: Value = serde_json::from_slice(&first)?;
    assert_eq!(projection["schema"], "cairn.lifecycle-store-identity.v1");
    assert_eq!(projection["store_id"], "shared-planning");
    assert_eq!(projection["source_blake3"].as_str().map(str::len), Some(64));

    let repeated = run_config(&root, "store", &["--execute"])?;
    assert_eq!(repeated["mutated"], false);
    assert_eq!(fs::read(root.join(STORE_PROJECTION_PATH))?, first);

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.project_config.pointer]
fn project_export_is_preview_first_and_checkable() -> Result<(), Box<dyn Error>> {
    let root = test_root("project-positive")?;
    write_source(&root, PROJECT_SOURCE_PATH, PROJECT_SOURCE)?;

    let preview = run_config(&root, "project", &[])?;
    assert_eq!(preview["dry_run"], true);
    assert!(!root.join(PROJECT_PROJECTION_PATH).exists());

    run_config(&root, "project", &["--execute"])?;
    let projection_bytes = fs::read(root.join(PROJECT_PROJECTION_PATH))?;
    let projection: Value = serde_json::from_slice(&projection_bytes)?;
    assert_eq!(projection["schema"], "cairn.lifecycle-store-project.v1");
    assert_eq!(projection["lifecycle_store"]["store_id"], "shared-planning");
    assert_eq!(projection["references"][0]["store_id"], "reference-store");
    assert_eq!(projection["source_blake3"].as_str().map(str::len), Some(64));

    let checked = run_config(&root, "project", &["--check"])?;
    assert_eq!(checked["current"], true);
    assert_eq!(
        fs::read(root.join(PROJECT_PROJECTION_PATH))?,
        projection_bytes
    );

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.identity.invalid]
fn malformed_store_and_missing_non_claims_do_not_mutate_projection() -> Result<(), Box<dyn Error>> {
    let root = test_root("store-negative")?;
    write_source(&root, STORE_SOURCE_PATH, "{ malformed =\n")?;
    let projection = root.join(STORE_PROJECTION_PATH);
    fs::write(&projection, b"preserve-store-projection\n")?;

    let malformed = config_output(&root, "store", &["--execute"])?;
    assert!(!malformed.status.success());
    assert_eq!(fs::read(&projection)?, b"preserve-store-projection\n");

    let missing_non_claims = STORE_SOURCE.replace(
        "    \"store identity does not prove requirement truth, implementation, lifecycle readiness, or release eligibility\",\n",
        "",
    );
    write_source(&root, STORE_SOURCE_PATH, &missing_non_claims)?;
    let missing = config_output(&root, "store", &["--execute"])?;
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr)
        .contains("required lifecycle-store identity non-claim is missing"));
    assert_eq!(fs::read(&projection)?, b"preserve-store-projection\n");

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.project_config.invalid]
fn unsafe_project_remote_and_stale_check_do_not_mutate_projection() -> Result<(), Box<dyn Error>> {
    let root = test_root("project-negative")?;
    write_source(&root, PROJECT_SOURCE_PATH, PROJECT_SOURCE)?;
    run_config(&root, "project", &["--execute"])?;
    let projection_path = root.join(PROJECT_PROJECTION_PATH);
    let original = fs::read(&projection_path)?;

    let changed = PROJECT_SOURCE.replace("reference-store.git", "reference store.git");
    write_source(&root, PROJECT_SOURCE_PATH, &changed)?;
    let unsafe_remote = config_output(&root, "project", &["--execute"])?;
    assert!(!unsafe_remote.status.success());
    assert_eq!(fs::read(&projection_path)?, original);

    write_source(
        &root,
        PROJECT_SOURCE_PATH,
        &(PROJECT_SOURCE.to_string() + "\n"),
    )?;
    let stale = config_output(&root, "project", &["--check"])?;
    assert!(!stale.status.success());
    let receipt: Value = serde_json::from_slice(&stale.stdout)?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["current"], false);
    assert_eq!(fs::read(&projection_path)?, original);

    remove_tree(&root)?;
    Ok(())
}

#[test]
fn wrong_kind_fails_without_creating_a_projection() -> Result<(), Box<dyn Error>> {
    let root = test_root("wrong-kind")?;
    write_source(&root, STORE_SOURCE_PATH, STORE_SOURCE)?;

    let output = config_output(&root, "wrong", &["--execute"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must be store or project"));
    assert!(!root.join(STORE_PROJECTION_PATH).exists());
    assert!(!root.join(PROJECT_PROJECTION_PATH).exists());

    remove_tree(&root)?;
    Ok(())
}
