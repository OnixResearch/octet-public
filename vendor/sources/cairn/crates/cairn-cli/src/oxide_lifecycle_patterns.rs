use std::{error::Error, fs, path::Path};

use cairn_core::validate_oxide_lifecycle_patterns_value;
use serde_json::{json, Value};

use crate::support::{print_json, read_option, safe_relative_path, with_receipt_hash};

const OXIDE_PATTERN_INPUT_MAX_BYTES: usize = 2_097_152;

pub(crate) fn run_oxide_lifecycle_patterns(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("lifecycle-patterns requires a subcommand: validate".into());
    };
    match subcommand {
        "validate" => run_validate(args.into_iter().skip(1).collect()),
        unknown => Err(format!("unknown lifecycle-patterns subcommand: {unknown}").into()),
    }
}

fn run_validate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err("lifecycle-patterns validate requires --input <bundle.json>".into());
    }
    let value = read_bounded_json(&input)?;
    let report = validate_oxide_lifecycle_patterns_value(&value);
    let receipt = with_receipt_hash(json!({
        "command": "lifecycle-patterns validate",
        "root": ".",
        "input": safe_relative_path(&root, &input),
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "input_hash": report.input_hash,
        "reference_count": report.reference_count,
        "saga_step_count": report.saga_step_count,
        "document_plan_count": report.document_plan_count,
        "golden_snapshot_count": report.golden_snapshot_count,
        "bundle_member_count": report.bundle_member_count,
        "issues": report.issues,
        "boundary": report.boundary,
    }));
    let valid = receipt.get("valid").and_then(Value::as_bool) == Some(true);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

fn read_bounded_json(path: &Path) -> Result<Value, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > OXIDE_PATTERN_INPUT_MAX_BYTES as u64 {
        return Err("lifecycle pattern bundle exceeds the reviewed byte bound".into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
