use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

const DATA_DIRECTORY_OVERRIDE: &str = "CAIRN_DATA_DIR";
const STORE_SOURCE_PATH: &str = ".cairn-store/store.ncl";
const STORE_PROJECTION_PATH: &str = ".cairn-store/store.json";
const REGISTRY_PATH: &str = "cairn/stores/registry.json";
const BLAKE3_HEX_LENGTH: usize = 64;
const STORE_ALPHA: &str = "alpha";
const STORE_BETA: &str = "beta";
const REMOTE_ALPHA: &str = "https://example.invalid/alpha.git";
const REMOTE_ALPHA_MIRROR: &str = "https://mirror.invalid/alpha.git";

fn test_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = env::temp_dir().join(format!(
        "cairn-lifecycle-store-{name}-{}",
        std::process::id()
    ));
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

fn run_store(data_dir: &Path, args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_cairn"))
        .arg("store")
        .args(args)
        .env(DATA_DIRECTORY_OVERRIDE, data_dir)
        .output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn setup_store(
    data_dir: &Path,
    store_id: &str,
    path: &Path,
    remote: Option<&str>,
) -> Result<Value, Box<dyn Error>> {
    let path_text = path.to_string_lossy().into_owned();
    let mut args = vec!["setup", store_id, "--path", path_text.as_str()];
    if let Some(remote) = remote {
        args.extend(["--remote", remote]);
    }
    args.push("--execute");
    run_store(data_dir, &args)
}

fn register_store(
    data_dir: &Path,
    path: &Path,
    store_id: Option<&str>,
    execute: bool,
) -> Result<Value, Box<dyn Error>> {
    let path_text = path.to_string_lossy().into_owned();
    let mut args = vec!["register", path_text.as_str()];
    if let Some(store_id) = store_id {
        args.extend(["--id", store_id]);
    }
    if execute {
        args.push("--execute");
    }
    run_store(data_dir, &args)
}

fn diagnostic_codes(receipt: &Value) -> Vec<&str> {
    receipt["diagnostics"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|diagnostic| diagnostic["code"].as_str())
        .collect()
}

fn copy_identity(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::copy(
        source.join(STORE_SOURCE_PATH),
        destination.join(STORE_SOURCE_PATH),
    )?;
    fs::copy(
        source.join(STORE_PROJECTION_PATH),
        destination.join(STORE_PROJECTION_PATH),
    )?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.commands.setup]
// r[verify cairn.lifecycle_stores.transport_boundary.store]
fn setup_is_preview_first_and_creates_only_native_artifacts() -> Result<(), Box<dyn Error>> {
    let root = test_root("setup-preview")?;
    let data_dir = root.join("data");
    let store = root.join("store");
    let store_text = store.to_string_lossy().into_owned();

    let preview = run_store(
        &data_dir,
        &[
            "setup",
            STORE_ALPHA,
            "--path",
            store_text.as_str(),
            "--remote",
            REMOTE_ALPHA,
        ],
    )?;
    assert_eq!(preview["valid"], true);
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!store.exists());

    let executed = setup_store(&data_dir, STORE_ALPHA, &store, Some(REMOTE_ALPHA))?;
    assert_eq!(executed["valid"], true);
    assert_eq!(executed["dry_run"], false);
    assert_eq!(executed["mutated"], true);
    assert!(store.join(STORE_SOURCE_PATH).is_file());
    assert!(store.join(STORE_PROJECTION_PATH).is_file());
    assert!(store.join(".cairn/specs").is_dir());
    assert!(store.join(".cairn/changes").is_dir());
    assert!(store.join(".cairn/archive").is_dir());
    assert!(!store.join(".git").exists());
    assert!(!store.join(".jj").exists());

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.commands.blocked]
fn setup_blocks_overwrite_and_preserves_existing_content() -> Result<(), Box<dyn Error>> {
    let root = test_root("setup-overwrite")?;
    let data_dir = root.join("data");
    let store = root.join("store");
    fs::create_dir_all(&store)?;
    fs::write(store.join("keep.txt"), b"keep\n")?;

    let receipt = setup_store(&data_dir, STORE_ALPHA, &store, None)?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["blocked"], true);
    assert_eq!(receipt["mutated"], false);
    assert_eq!(fs::read(store.join("keep.txt"))?, b"keep\n");
    assert!(!store.join(".cairn-store").exists());

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.locator.register]
fn register_list_show_doctor_and_unregister_use_only_overridden_state() -> Result<(), Box<dyn Error>>
{
    let root = test_root("locator-positive")?;
    let data_dir = root.join("isolated-data");
    let store = root.join("store");
    setup_store(&data_dir, STORE_ALPHA, &store, Some(REMOTE_ALPHA))?;

    let preview = register_store(&data_dir, &store, Some(STORE_ALPHA), false)?;
    assert_eq!(preview["valid"], true);
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!data_dir.join(REGISTRY_PATH).exists());

    let executed = register_store(&data_dir, &store, Some(STORE_ALPHA), true)?;
    assert_eq!(executed["valid"], true);
    assert_eq!(executed["mutated"], true);
    assert!(data_dir.join(REGISTRY_PATH).is_file());

    let listed = run_store(&data_dir, &["list"])?;
    assert_eq!(listed["valid"], true);
    assert_eq!(listed["stores"].as_array().map(Vec::len), Some(1));
    assert_eq!(listed["stores"][0]["store_id"], STORE_ALPHA);

    let shown = run_store(&data_dir, &["show", STORE_ALPHA])?;
    assert_eq!(shown["valid"], true);
    assert_eq!(shown["projection"]["store_id"], STORE_ALPHA);
    assert_eq!(shown["checks"]["projection_current"], true);

    let doctor = run_store(&data_dir, &["doctor", STORE_ALPHA])?;
    assert_eq!(doctor["valid"], true);
    assert_eq!(doctor["stores"][0]["checks"]["identity_matches"], true);

    let registry_before_preview = fs::read(data_dir.join(REGISTRY_PATH))?;
    let unregister_preview = run_store(&data_dir, &["unregister", STORE_ALPHA])?;
    assert_eq!(unregister_preview["dry_run"], true);
    assert_eq!(unregister_preview["mutated"], false);
    assert_eq!(
        fs::read(data_dir.join(REGISTRY_PATH))?,
        registry_before_preview
    );

    let unregister = run_store(&data_dir, &["unregister", STORE_ALPHA, "--execute"])?;
    assert_eq!(unregister["valid"], true);
    assert_eq!(unregister["mutated"], true);
    let empty = run_store(&data_dir, &["list"])?;
    assert!(empty["stores"].as_array().is_some_and(Vec::is_empty));

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.identity.invalid]
// r[verify cairn.lifecycle_stores.locator.conflict]
fn stale_projection_and_malformed_identity_do_not_create_locator_state(
) -> Result<(), Box<dyn Error>> {
    let root = test_root("identity-negative")?;
    let data_dir = root.join("data");
    let stale_store = root.join("stale-store");
    setup_store(&data_dir, STORE_ALPHA, &stale_store, None)?;
    let mut source = fs::read(stale_store.join(STORE_SOURCE_PATH))?;
    source.push(b'\n');
    fs::write(stale_store.join(STORE_SOURCE_PATH), source)?;

    let stale = register_store(&data_dir, &stale_store, Some(STORE_ALPHA), true)?;
    assert_eq!(stale["valid"], false);
    assert!(diagnostic_codes(&stale).contains(&"lifecycle_store.identity.stale_projection"));
    assert!(!data_dir.join(REGISTRY_PATH).exists());

    let malformed_store = root.join("malformed-store");
    setup_store(&data_dir, STORE_BETA, &malformed_store, None)?;
    fs::write(malformed_store.join(STORE_PROJECTION_PATH), b"{\n")?;
    let malformed = register_store(&data_dir, &malformed_store, Some(STORE_BETA), true)?;
    assert_eq!(malformed["valid"], false);
    assert!(diagnostic_codes(&malformed).contains(&"lifecycle_store.identity.invalid"));
    assert!(!data_dir.join(REGISTRY_PATH).exists());

    remove_tree(&root)?;
    Ok(())
}

#[cfg(unix)]
#[test]
// r[verify cairn.lifecycle_stores.locator.conflict]
fn symlink_checkout_path_is_rejected_without_state_mutation() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let root = test_root("symlink-path")?;
    let data_dir = root.join("data");
    let store = root.join("store");
    let linked = root.join("linked-store");
    setup_store(&data_dir, STORE_ALPHA, &store, None)?;
    symlink(&store, &linked)?;

    let receipt = register_store(&data_dir, &linked, Some(STORE_ALPHA), true)?;
    assert_eq!(receipt["valid"], false);
    assert!(diagnostic_codes(&receipt).contains(&"lifecycle_store.locator.symlink_path"));
    assert!(!data_dir.join(REGISTRY_PATH).exists());

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.locator.conflict]
fn duplicate_ids_identity_drift_and_duplicate_paths_preserve_locator_state(
) -> Result<(), Box<dyn Error>> {
    let root = test_root("locator-conflicts")?;
    let data_dir = root.join("data");
    let alpha = root.join("alpha");
    let alpha_duplicate = root.join("alpha-duplicate");
    let alpha_mirror = root.join("alpha-mirror");
    let beta = root.join("beta");
    setup_store(&data_dir, STORE_ALPHA, &alpha, Some(REMOTE_ALPHA))?;
    setup_store(&data_dir, STORE_ALPHA, &alpha_duplicate, Some(REMOTE_ALPHA))?;
    setup_store(
        &data_dir,
        STORE_ALPHA,
        &alpha_mirror,
        Some(REMOTE_ALPHA_MIRROR),
    )?;
    setup_store(&data_dir, STORE_BETA, &beta, None)?;
    register_store(&data_dir, &alpha, Some(STORE_ALPHA), true)?;
    let registry = data_dir.join(REGISTRY_PATH);
    let original = fs::read(&registry)?;

    let duplicate_id = register_store(&data_dir, &alpha_duplicate, Some(STORE_ALPHA), true)?;
    assert!(diagnostic_codes(&duplicate_id).contains(&"lifecycle_store.locator.duplicate_id"));
    assert_eq!(fs::read(&registry)?, original);

    copy_identity(&alpha_mirror, &alpha)?;
    let drift = register_store(&data_dir, &alpha, Some(STORE_ALPHA), true)?;
    assert!(diagnostic_codes(&drift).contains(&"lifecycle_store.locator.identity_changed"));
    assert_eq!(fs::read(&registry)?, original);

    copy_identity(&beta, &alpha)?;
    let duplicate_path = register_store(&data_dir, &alpha, Some(STORE_BETA), true)?;
    assert!(diagnostic_codes(&duplicate_path).contains(&"lifecycle_store.locator.duplicate_path"));
    assert_eq!(fs::read(&registry)?, original);

    remove_tree(&root)?;
    Ok(())
}

#[test]
// r[verify cairn.lifecycle_stores.functional_boundary.shell]
// r[verify cairn.lifecycle_stores.commands.blocked]
fn stale_locator_state_blocks_mutation_and_preserves_exact_bytes() -> Result<(), Box<dyn Error>> {
    let root = test_root("stale-locator")?;
    let data_dir = root.join("data");
    let store = root.join("store");
    setup_store(&data_dir, STORE_ALPHA, &store, None)?;
    register_store(&data_dir, &store, Some(STORE_ALPHA), true)?;
    let registry = data_dir.join(REGISTRY_PATH);
    let mut stale: Value = serde_json::from_slice(&fs::read(&registry)?)?;
    stale["state_hash"] = Value::String("a".repeat(BLAKE3_HEX_LENGTH));
    let mut stale_bytes = serde_json::to_vec_pretty(&stale)?;
    stale_bytes.push(b'\n');
    fs::write(&registry, &stale_bytes)?;

    let receipt = run_store(&data_dir, &["unregister", STORE_ALPHA, "--execute"])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["mutated"], false);
    assert!(diagnostic_codes(&receipt).contains(&"lifecycle_store.identity.invalid"));
    assert_eq!(fs::read(&registry)?, stale_bytes);

    remove_tree(&root)?;
    Ok(())
}
