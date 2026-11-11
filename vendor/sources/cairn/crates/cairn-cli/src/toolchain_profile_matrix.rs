use std::{error::Error, fs, path::PathBuf};

use cairn_core::{
    canonical_json_hash, toolchain_profile_issues_json, validate_toolchain_profile_matrix_value,
};
use serde_json::{json, Value};

use crate::support::{print_json, read_option, with_receipt_hash};

pub(crate) fn run_toolchain_profile(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("toolchain-profile requires a subcommand: verify".into());
    };
    match subcommand {
        "verify" => run_toolchain_profile_verify(args.into_iter().skip(1).collect()),
        other => Err(format!("unknown toolchain-profile subcommand: {other}").into()),
    }
}

fn run_toolchain_profile_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.toolchain_profile_matrix.validation]
    // r[impl cairn.toolchain_profile_matrix.validation.receipt]
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err("toolchain-profile verify requires --input <matrix.json>".into());
    }
    let value = read_toolchain_profile_input(&input)?;
    let report = validate_toolchain_profile_matrix_value(&value);
    let receipt = json!({
        "command": "toolchain-profile verify",
        "schema": "cairn.toolchain-profile.verify-receipt.v1",
        "input": input.to_string_lossy(),
        "input_hash": canonical_json_hash(&value),
        "matrix_hash": report.matrix_hash,
        "policy_hash": policy_hash(&value),
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "profile_count": report.profile_count,
        "invalid_profile_count": report.invalid_profile_count,
        "boundary": report.boundary,
        "issues": toolchain_profile_issues_json(&report),
    });
    print_json(with_receipt_hash(receipt))?;
    if !report.valid {
        std::process::exit(1);
    }
    Ok(())
}

fn policy_hash(value: &Value) -> String {
    value
        .get("policy_hash")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| canonical_json_hash(&Value::Null))
}

fn read_toolchain_profile_input(path: &PathBuf) -> Result<Value, Box<dyn Error>> {
    let text = fs::read_to_string(path).map_err(|source| {
        format!(
            "failed to read toolchain profile matrix input {}: {source}",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|source| {
        format!(
            "failed to parse toolchain profile matrix input {}: {source}",
            path.display()
        )
        .into()
    })
}
