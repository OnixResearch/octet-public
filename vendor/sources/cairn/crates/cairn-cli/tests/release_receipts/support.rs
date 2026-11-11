use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(repo_root());
    command
}

pub(super) fn run_json(args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    run_json_command(&mut command)
}

pub(super) fn run_json_command(command: &mut Command) -> Result<Value, Box<dyn Error>> {
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

pub(super) fn run_json_failure_command(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub(super) fn copy_dir(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
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

pub(super) fn temp_project(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let destination = env::temp_dir().join(format!("cairn-cli-{name}-{}", std::process::id()));
    if destination.exists() {
        fs::remove_dir_all(&destination)?;
    }
    fs::create_dir_all(&destination)?;
    Ok(destination)
}

pub(super) fn temp_fixture(name: &str, fixture: &str) -> Result<PathBuf, Box<dyn Error>> {
    let destination = temp_project(name)?;
    fs::remove_dir_all(&destination)?;
    copy_dir(&repo_root().join(fixture), &destination)?;
    Ok(destination)
}

pub(super) fn assert_hash_field(value: &Value, field: &str) -> Result<(), Box<dyn Error>> {
    let hash = value[field].as_str().ok_or("hash field missing")?;
    assert_eq!(hash.len(), 64, "{field} should be a BLAKE3 hex hash");
    assert!(
        hash.chars().all(|character| character.is_ascii_hexdigit()),
        "{field} should be hex: {hash}"
    );
    Ok(())
}

pub(super) fn assert_object_keys(value: &Value, expected: &[&str]) -> Result<(), Box<dyn Error>> {
    let object = value.as_object().ok_or("json object")?;
    let mut actual = object.keys().map(String::as_str).collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected, "receipt object keys changed");
    Ok(())
}

pub(super) fn policy_json() -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_slice(&fs::read(
        repo_root().join("cairn-policy/generated/cairn-policy.json"),
    )?)?)
}

pub(super) fn receipt_schema(command: &str) -> Result<Value, Box<dyn Error>> {
    policy_json()?["receipt_schemas"]
        .as_array()
        .ok_or("receipt_schemas array")?
        .iter()
        .find(|schema| schema["command"] == command)
        .cloned()
        .ok_or_else(|| format!("missing receipt schema for {command}").into())
}

pub(super) fn policy_replay_group_cases(group_id: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut cases = policy_json()?["determinism_replay_policy"]["groups"]
        .as_array()
        .ok_or("policy replay groups")?
        .iter()
        .find(|group| group["id"] == group_id)
        .ok_or_else(|| format!("missing replay group {group_id}"))?["cases"]
        .as_array()
        .ok_or("group cases")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "case id string".into())
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    cases.sort();
    cases.dedup();
    Ok(cases)
}

pub(super) fn schema_strings(schema: &Value, field: &str) -> Result<Vec<String>, Box<dyn Error>> {
    schema[field]
        .as_array()
        .ok_or_else(|| format!("{field} array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{field} string").into())
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()
}

pub(super) fn assert_schema_keys(
    value: &Value,
    schema: &Value,
    field: &str,
) -> Result<(), Box<dyn Error>> {
    let expected = schema_strings(schema, field)?;
    assert_object_keys(
        value,
        &expected.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

pub(super) fn assert_first_action_matches_schema(
    value: &Value,
    schema: &Value,
) -> Result<(), Box<dyn Error>> {
    let actions = value["actions"].as_array().ok_or("actions array")?;
    let action = actions.first().ok_or("at least one action")?;
    assert_schema_keys(action, schema, "action_keys")
}

pub(super) fn assert_reproducible_receipt(
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

pub(super) fn run_json_strings(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    run_json_command(&mut command)
}

pub(super) fn run_json_failure_strings(args: &[String]) -> Result<Value, Box<dyn Error>> {
    let mut command = cairn();
    command.args(args);
    run_json_failure_command(&mut command)
}

pub(super) fn assert_hash_fields_equal(
    first: &Value,
    second: &Value,
    fields: &[&str],
) -> Result<(), Box<dyn Error>> {
    for field in fields {
        assert_hash_field(first, field)?;
        assert_hash_field(second, field)?;
        assert_eq!(
            first[field], second[field],
            "portable hash field {field} changed across roots"
        );
    }
    Ok(())
}
