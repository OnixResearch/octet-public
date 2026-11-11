#![allow(dead_code)]

// r[impl cli_maintainability.lifecycle_tests.focused_modules.narrow_bridges]
use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

pub(crate) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(crate) fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

pub(crate) fn ironspec_alias() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ironspec"));
    command.current_dir(repo_root());
    command
}

pub(crate) fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    run_json_command(&mut command)
}

pub(crate) fn run_json_command(command: &mut Command) -> Result<Value, Box<dyn Error>> {
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

pub(crate) fn copy_dir(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    let mut dirs = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((current_source, current_destination)) = dirs.pop() {
        fs::create_dir_all(&current_destination)?;
        for entry in fs::read_dir(current_source)? {
            let entry = entry?;
            let source_path = entry.path();
            let destination_path = current_destination.join(entry.file_name());
            if entry.file_type()?.is_dir() {
                dirs.push((source_path, destination_path));
            } else {
                fs::copy(&source_path, &destination_path)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn temp_project(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let destination = env::temp_dir().join(format!("cairn-cli-{name}-{}", std::process::id()));
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    fs::create_dir_all(&destination)?;
    Ok(destination)
}

pub(crate) fn temp_fixture(name: &str, fixture: &str) -> Result<PathBuf, Box<dyn Error>> {
    let destination = temp_project(name)?;
    fs::remove_dir_all(&destination)?;
    copy_dir(&repo_root().join(fixture), &destination)?;
    Ok(destination)
}

pub(crate) fn golden(name: &str) -> Result<Value, Box<dyn Error>> {
    let path = repo_root().join("crates/cairn-cli/tests/golden").join(name);
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn assert_matches_golden(args: &[&str], name: &str) -> Result<(), Box<dyn Error>> {
    assert_eq!(run_json(args)?, golden(name)?);
    Ok(())
}

pub(crate) fn assert_hash_field(value: &Value, field: &str) -> Result<(), Box<dyn Error>> {
    let hash = value[field].as_str().ok_or("hash field missing")?;
    assert_eq!(hash.len(), 64, "{field} should be a BLAKE3 hex hash");
    assert!(
        hash.chars().all(|character| character.is_ascii_hexdigit()),
        "{field} should be hex: {hash}"
    );
    Ok(())
}

pub(crate) fn assert_reproducible_receipt(
    args: &[&str],
    hash_fields: &[&str],
) -> Result<Value, Box<dyn Error>> {
    let first = run_json(args)?;
    let second = run_json(args)?;
    assert_eq!(
        first, second,
        "receipt output should be reproducible for {args:?}"
    );
    for field in hash_fields {
        assert_hash_field(&first, field)?;
    }
    Ok(first)
}

pub(crate) fn run_json_strings(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    run_json_command(&mut command)
}
