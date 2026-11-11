use std::{error::Error, fs, path::PathBuf};

use cairn_core::{
    canonical_json_hash, hash_algorithm_policy_issues_json, validate_hash_algorithm_policy_value,
};
use serde_json::{json, Value};

use crate::support::{print_json, read_option, with_receipt_hash};

pub(crate) fn run_hash_policy(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("hash-policy requires a subcommand: verify".into());
    };
    match subcommand {
        "verify" => run_hash_policy_verify(args.into_iter().skip(1).collect()),
        other => Err(format!("unknown hash-policy subcommand: {other}").into()),
    }
}

fn run_hash_policy_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.hash_algorithm_policy.validation]
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err("hash-policy verify requires --input <manifest.json>".into());
    }
    let value = read_hash_policy_input(&input)?;
    let report = validate_hash_algorithm_policy_value(&value);
    let receipt = json!({
        "command": "hash-policy verify",
        "schema": "cairn.hash-policy.verify-receipt.v1",
        "input": input.to_string_lossy(),
        "input_hash": canonical_json_hash(&value),
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "field_count": report.field_count,
        "accepted_exception_count": report.accepted_exception_count,
        "boundary": report.boundary,
        "issues": hash_algorithm_policy_issues_json(&report),
    });
    print_json(with_receipt_hash(receipt))?;
    if !report.valid {
        std::process::exit(1);
    }
    Ok(())
}

fn read_hash_policy_input(path: &PathBuf) -> Result<Value, Box<dyn Error>> {
    let text = fs::read_to_string(path).map_err(|source| {
        format!(
            "failed to read hash policy input {}: {source}",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|source| {
        format!(
            "failed to parse hash policy input {}: {source}",
            path.display()
        )
        .into()
    })
}
