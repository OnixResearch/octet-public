use std::{error::Error, fs, io::Write, path::Path};

use cairn_core::{
    canonical_json_hash, validate_circuit_primitive_receipt, validate_downstream_pilot_receipt,
    validate_export_bundle_receipt, validate_function_object_receipt,
    validate_name_pointer_receipt,
};
use serde_json::{json, Value};

const FUNCTION_OBJECT_MAX_BYTES: u64 = 64 * 1024;
const NAME_POINTER_MAX_BYTES: u64 = 64 * 1024;
const EXPORT_BUNDLE_MAX_BYTES: u64 = 128 * 1024;
const DOWNSTREAM_PILOT_MAX_BYTES: u64 = 64 * 1024;
const CIRCUIT_PRIMITIVE_MAX_BYTES: u64 = 128 * 1024;

pub(crate) fn run_evidence_circuit_primitive(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence circuit-primitive requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err(
                    "evidence circuit-primitive validate requires a JSON receipt path".into(),
                );
            };
            let receipt = read_json_artifact(Path::new(&path), CIRCUIT_PRIMITIVE_MAX_BYTES)?;
            let report = validate_circuit_primitive_receipt(&receipt);
            let issues = report
                .issues
                .into_iter()
                .map(|issue| json!({"code": issue.code, "message": issue.message}))
                .collect::<Vec<_>>();
            let receipt_hash = canonical_json_hash(&json!({
                "schema": receipt.get("schema"),
                "computed_graph_hash": report.computed_graph_hash.clone(),
                "valid": report.valid,
                "issues": issues.clone(),
            }));
            let output = json!({
                "command": "evidence circuit-primitive validate",
                "path": path,
                "schema": receipt.get("schema").and_then(Value::as_str).unwrap_or(""),
                "valid": report.valid,
                "verdict": report.verdict,
                "issues": issues,
                "computed_graph_hash": report.computed_graph_hash.clone(),
                "receipt_hash": receipt_hash,
            });
            let valid = output["valid"].as_bool().unwrap_or(false);
            print_json(output)?;
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        unknown => Err(format!("unknown evidence circuit-primitive subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_evidence_downstream_pilot(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence downstream-pilot requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err(
                    "evidence downstream-pilot validate requires a JSON receipt path".into(),
                );
            };
            let receipt = read_json_artifact(Path::new(&path), DOWNSTREAM_PILOT_MAX_BYTES)?;
            let report = validate_downstream_pilot_receipt(&receipt);
            let issues = report
                .issues
                .into_iter()
                .map(|issue| json!({"code": issue.code, "message": issue.message}))
                .collect::<Vec<_>>();
            let receipt_hash = canonical_json_hash(&json!({
                "schema": receipt.get("schema"),
                "computed_pilot_hash": report.computed_pilot_hash.clone(),
                "valid": report.valid,
                "issues": issues.clone(),
            }));
            let output = json!({
                "command": "evidence downstream-pilot validate",
                "path": path,
                "schema": receipt.get("schema").and_then(Value::as_str).unwrap_or(""),
                "valid": report.valid,
                "verdict": report.verdict,
                "issues": issues,
                "computed_pilot_hash": report.computed_pilot_hash.clone(),
                "receipt_hash": receipt_hash,
            });
            let valid = output["valid"].as_bool().unwrap_or(false);
            print_json(output)?;
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        unknown => Err(format!("unknown evidence downstream-pilot subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_evidence_export_bundle_receipt(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence export-bundle requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err("evidence export-bundle validate requires a JSON receipt path".into());
            };
            let receipt = read_json_artifact(Path::new(&path), EXPORT_BUNDLE_MAX_BYTES)?;
            let report = validate_export_bundle_receipt(&receipt);
            let issues = report
                .issues
                .into_iter()
                .map(|issue| json!({"code": issue.code, "message": issue.message}))
                .collect::<Vec<_>>();
            let receipt_hash = canonical_json_hash(&json!({
                "schema": receipt.get("schema"),
                "computed_bundle_hash": report.computed_bundle_hash.clone(),
                "valid": report.valid,
                "issues": issues.clone(),
            }));
            let output = json!({
                "command": "evidence export-bundle validate",
                "path": path,
                "schema": receipt.get("schema").and_then(Value::as_str).unwrap_or(""),
                "valid": report.valid,
                "verdict": report.verdict,
                "issues": issues,
                "computed_bundle_hash": report.computed_bundle_hash.clone(),
                "receipt_hash": receipt_hash,
            });
            let valid = output["valid"].as_bool().unwrap_or(false);
            print_json(output)?;
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        unknown => Err(format!("unknown evidence export-bundle subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_evidence_name_pointer(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence name-pointer requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err("evidence name-pointer validate requires a JSON receipt path".into());
            };
            let receipt = read_json_artifact(Path::new(&path), NAME_POINTER_MAX_BYTES)?;
            let report = validate_name_pointer_receipt(&receipt);
            let issues = report
                .issues
                .into_iter()
                .map(|issue| json!({"code": issue.code, "message": issue.message}))
                .collect::<Vec<_>>();
            let receipt_hash = canonical_json_hash(&json!({
                "schema": receipt.get("schema"),
                "computed_pointer_hash": report.computed_pointer_hash.clone(),
                "valid": report.valid,
                "issues": issues.clone(),
            }));
            let output = json!({
                "command": "evidence name-pointer validate",
                "path": path,
                "schema": receipt.get("schema").and_then(Value::as_str).unwrap_or(""),
                "valid": report.valid,
                "verdict": report.verdict,
                "issues": issues,
                "computed_pointer_hash": report.computed_pointer_hash.clone(),
                "receipt_hash": receipt_hash,
            });
            let valid = output["valid"].as_bool().unwrap_or(false);
            print_json(output)?;
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        unknown => Err(format!("unknown evidence name-pointer subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_evidence_function_object(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence function-object requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err(
                    "evidence function-object validate requires a JSON receipt path".into(),
                );
            };
            let receipt = read_json_artifact(Path::new(&path), FUNCTION_OBJECT_MAX_BYTES)?;
            let report = validate_function_object_receipt(&receipt);
            let issues = report
                .issues
                .into_iter()
                .map(|issue| json!({"code": issue.code, "message": issue.message}))
                .collect::<Vec<_>>();
            let receipt_hash = canonical_json_hash(&json!({
                "schema": receipt.get("schema"),
                "computed_object_hash": report.computed_object_hash.clone(),
                "valid": report.valid,
                "issues": issues.clone(),
            }));
            let output = json!({
                "command": "evidence function-object validate",
                "path": path,
                "schema": receipt.get("schema").and_then(Value::as_str).unwrap_or(""),
                "valid": report.valid,
                "verdict": report.verdict,
                "issues": issues,
                "computed_object_hash": report.computed_object_hash.clone(),
                "receipt_hash": receipt_hash,
            });
            let valid = output["valid"].as_bool().unwrap_or(false);
            print_json(output)?;
            if !valid {
                std::process::exit(1);
            }
            Ok(())
        }
        unknown => Err(format!("unknown evidence function-object subcommand: {unknown}").into()),
    }
}

fn read_json_artifact(path: &Path, max_bytes: u64) -> Result<Value, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > max_bytes {
        return Err(format!("artifact too large: {} bytes", metadata.len()).into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn print_json(value: Value) -> Result<(), Box<dyn Error>> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, &value)?;
    writeln!(handle)?;
    Ok(())
}
