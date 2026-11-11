use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::SystemTime,
};

use serde_json::Value;

const BLAKE3_HEX_LENGTH: usize = 64;
const REPOSITORY_ID: &str = "OnixResearch/example";
const OTHER_REPOSITORY_ID: &str = "OnixResearch/other";
const REVISION_KIND: &str = "opaque";
const REVISION_VALUE: &str = "reviewed-fixture-revision";
const CURRENT_FIXTURE: &str = "fixtures/content-bound-requirement-registry/current";
const DUPLICATE_FIXTURE: &str = "fixtures/content-bound-requirement-registry/duplicate";
const CURRENT_REQUIREMENT_ID: &str = "example.registry.current";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

fn temp_test_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let directory = env::temp_dir().join(format!(
        "cairn-content-bound-registry-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&directory)?;
    Ok(directory)
}

fn run_json_success(args: &[String]) -> Result<Value, Box<dyn Error>> {
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

fn run_json_failure(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn export_args(root: &Path, output: &Path) -> Vec<String> {
    vec![
        "requirement-registry".to_string(),
        "export".to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--repository-id".to_string(),
        REPOSITORY_ID.to_string(),
        "--revision-kind".to_string(),
        REVISION_KIND.to_string(),
        "--revision".to_string(),
        REVISION_VALUE.to_string(),
        "--out".to_string(),
        output.display().to_string(),
    ]
}

fn verify_args(root: &Path, registry: &Path, repository_id: &str) -> Vec<String> {
    vec![
        "requirement-registry".to_string(),
        "verify".to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--registry".to_string(),
        registry.display().to_string(),
        "--repository-id".to_string(),
        repository_id.to_string(),
    ]
}

fn issue_codes(receipt: &Value) -> Vec<&str> {
    receipt["issues"]
        .as_array()
        .expect("issues array")
        .iter()
        .map(|issue| issue["code"].as_str().expect("issue code"))
        .collect()
}

fn assert_hash(value: &Value) {
    let hash = value.as_str().expect("hash string");
    assert_eq!(hash.len(), BLAKE3_HEX_LENGTH);
    assert!(hash
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')));
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((source_directory, destination_directory)) = pending.pop() {
        for entry in fs::read_dir(&source_directory)? {
            let entry = entry?;
            let source_path = entry.path();
            let destination_path = destination_directory.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                fs::create_dir_all(&destination_path)?;
                pending.push((source_path, destination_path));
            } else {
                fs::copy(source_path, destination_path)?;
            }
        }
    }
    Ok(())
}

#[test]
fn content_bound_registry_exports_deterministically_and_verifies_current_root(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.requirement_registry.content_bound_export]
    // r[verify cairn.requirement_registry.current_root_verification]
    let temporary = temp_test_dir("current")?;
    let first_path = temporary.join("first.json");
    let second_path = temporary.join("second.json");
    let fixture = repo_root().join(CURRENT_FIXTURE);

    let first_receipt = run_json_success(&export_args(&fixture, &first_path))?;
    let second_receipt = run_json_success(&export_args(&fixture, &second_path))?;
    assert_eq!(first_receipt["valid"], true);
    assert_eq!(second_receipt["valid"], true);
    assert_hash(&first_receipt["registry_blake3"]);
    assert_hash(&first_receipt["registry_receipt_blake3"]);

    let first_registry: Value = serde_json::from_slice(&fs::read(&first_path)?)?;
    let second_registry: Value = serde_json::from_slice(&fs::read(&second_path)?)?;
    assert_eq!(first_registry, second_registry);
    assert_eq!(
        first_registry["rows"][0]["requirement_id"],
        CURRENT_REQUIREMENT_ID
    );
    assert_eq!(
        first_registry["specifications"][0]["path"],
        ".cairn/specs/example/spec.md"
    );
    assert_eq!(
        first_registry["receipt"]["registry_blake3"],
        first_registry["registry_blake3"]
    );

    let verification = run_json_success(&verify_args(&fixture, &first_path, REPOSITORY_ID))?;
    assert_eq!(verification["valid"], true);
    assert_eq!(
        verification["registry_blake3"],
        first_registry["registry_blake3"]
    );
    assert_eq!(verification["revision_kind"], REVISION_KIND);
    assert!(verification["boundary"]
        .as_str()
        .expect("boundary")
        .contains("does not prove revision authenticity"));

    fs::remove_dir_all(temporary)?;
    Ok(())
}

#[test]
fn content_bound_registry_fails_closed_for_duplicate_malformed_stale_and_cross_repo_inputs(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.requirement_registry.current_root_verification]
    // r[verify cairn.requirement_registry.downstream_boundary]
    let temporary = temp_test_dir("negative")?;
    let duplicate_output = temporary.join("duplicate.json");
    let duplicate_fixture = repo_root().join(DUPLICATE_FIXTURE);
    let duplicate = run_json_failure(&export_args(&duplicate_fixture, &duplicate_output))?;
    assert!(
        issue_codes(&duplicate).contains(&"content_requirement_registry.requirement_id_duplicate")
    );
    assert!(!duplicate_output.exists());

    let malformed_output = temporary.join("malformed-revision.json");
    let fixture = repo_root().join(CURRENT_FIXTURE);
    let mut malformed_args = export_args(&fixture, &malformed_output);
    let kind_index = malformed_args
        .iter()
        .position(|argument| argument == REVISION_KIND)
        .expect("revision kind argument");
    malformed_args[kind_index] = "git".to_string();
    let malformed = run_json_failure(&malformed_args)?;
    assert!(issue_codes(&malformed).contains(&"content_requirement_registry.revision_invalid"));
    assert!(!malformed_output.exists());

    let current_root = temporary.join("current-root");
    copy_directory(&fixture, &current_root)?;
    let current_registry = temporary.join("current-registry.json");
    run_json_success(&export_args(&current_root, &current_registry))?;

    let cross_repo = run_json_failure(&verify_args(
        &current_root,
        &current_registry,
        OTHER_REPOSITORY_ID,
    ))?;
    assert!(issue_codes(&cross_repo).contains(&"content_requirement_registry.repository_mismatch"));

    let spec_path = current_root.join(".cairn/specs/example/spec.md");
    let source = fs::read_to_string(&spec_path)?;
    let changed = source.replace(
        "preserve the current requirement body",
        "preserve the changed requirement body",
    );
    assert_ne!(source, changed);
    fs::write(&spec_path, changed)?;
    let stale = run_json_failure(&verify_args(
        &current_root,
        &current_registry,
        REPOSITORY_ID,
    ))?;
    let stale_codes = issue_codes(&stale);
    assert!(stale_codes.contains(&"content_requirement_registry.stale_specification"));
    assert!(stale_codes.contains(&"content_requirement_registry.stale_requirement"));
    let stale_text = serde_json::to_string(&stale)?;
    assert!(!stale_text.contains("changed requirement body"));

    let malformed_registry = temporary.join("malformed-registry.json");
    fs::write(&malformed_registry, "{}\n")?;
    let malformed_registry_receipt = run_json_failure(&verify_args(
        &current_root,
        &malformed_registry,
        REPOSITORY_ID,
    ))?;
    assert!(issue_codes(&malformed_registry_receipt)
        .contains(&"content_requirement_registry.schema_unsupported"));

    fs::remove_dir_all(temporary)?;
    Ok(())
}

#[test]
fn legacy_requirement_registry_verify_remains_compatible() -> Result<(), Box<dyn Error>> {
    let positive = run_json_success(&[
        "requirement-registry".to_string(),
        "verify".to_string(),
        "--input".to_string(),
        "fixtures/requirement-id-registry/positive/active-alias.json".to_string(),
    ])?;
    assert_eq!(positive["valid"], true);
    assert_eq!(
        positive["schema"],
        "cairn.requirement-registry.verify-receipt.v1"
    );

    let negative = run_json_failure(&[
        "requirement-registry".to_string(),
        "verify".to_string(),
        "--input".to_string(),
        "fixtures/requirement-id-registry/negative/unknown-id.json".to_string(),
    ])?;
    assert_eq!(negative["valid"], false);
    assert!(issue_codes(&negative).contains(&"requirement_registry.unknown_requirement"));
    Ok(())
}
