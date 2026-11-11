use std::{error::Error, fs, path::Path};

use cairn_core::validate_state_machine_receipt_value;
use serde_json::{json, Value};

use crate::support::{print_json, read_option, safe_relative_path, with_receipt_hash};

const STATE_MACHINE_RECEIPT_MAX_BYTES: usize = 1_048_576;

pub(crate) fn run_state_machine_receipt(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("state-machine-receipt requires a subcommand: validate".into());
    };
    match subcommand {
        "validate" => run_validate(args.into_iter().skip(1).collect()),
        unknown => Err(format!("unknown state-machine-receipt subcommand: {unknown}").into()),
    }
}

fn run_validate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.lifecycle_orchestration.state_machine_receipts.cli]
    let root = read_option(args.clone(), "--root", ".");
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err("state-machine-receipt validate requires --input <receipt.json>".into());
    }
    let value = read_bounded_json(&input)?;
    let report = validate_state_machine_receipt_value(&value);
    let receipt = with_receipt_hash(json!({
        "command": "state-machine-receipt validate",
        "root": ".",
        "input": safe_relative_path(&root, &input),
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "state_machine_receipt_hash": report.receipt_hash,
        "checked_properties": report.checked_properties,
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
    if metadata.len() > STATE_MACHINE_RECEIPT_MAX_BYTES as u64 {
        return Err("state-machine receipt exceeds the reviewed byte bound".into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
