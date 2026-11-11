use std::{error::Error, fs, path::Path};

use cairn_core::{
    canonical_json_hash, release_run_explain, release_run_status, validate_release_run_ledger_value,
};
use serde_json::{json, Value};

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, safe_relative_path,
    with_receipt_hash,
};

const RELEASE_RUN_LEDGER_MAX_BYTES: usize = 2_097_152;

pub(crate) fn run_release_run(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().cloned() else {
        return Err("release-run requires a subcommand: validate, status, or explain".into());
    };
    let remaining = args.into_iter().skip(1).collect::<Vec<_>>();
    match subcommand.as_str() {
        "validate" => run_projection(remaining, Projection::Validate),
        "status" => run_projection(remaining, Projection::Status),
        "explain" => run_projection(remaining, Projection::Explain),
        unknown => Err(format!("unknown release-run subcommand: {unknown}").into()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Projection {
    Validate,
    Status,
    Explain,
}

impl Projection {
    fn command(self) -> &'static str {
        match self {
            Self::Validate => "release-run validate",
            Self::Status => "release-run status",
            Self::Explain => "release-run explain",
        }
    }
}

fn run_projection(args: Vec<String>, projection: Projection) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.release_evidence.durable_run_ledger.shell]
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err(format!("{} requires --input <ledger.json>", projection.command()).into());
    }
    let policy = load_policy_file(policy_path)?;
    let expected_policy_hash = canonical_json_hash(&policy.raw);
    let value = read_bounded_json(&input)?;
    let report = validate_release_run_ledger_value(&value, &expected_policy_hash);
    let projection_value = match projection {
        Projection::Validate => serde_json::to_value(&report)?,
        Projection::Status => serde_json::to_value(release_run_status(&report))?,
        Projection::Explain => serde_json::to_value(release_run_explain(&report))?,
    };
    let receipt = with_receipt_hash(json!({
        "command": projection.command(),
        "root": ".",
        "input": safe_relative_path(&root, &input),
        "policy_hash": expected_policy_hash,
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "run_id": report.run_id,
        "ledger_hash": report.ledger_hash,
        "projection": projection_value,
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
    if metadata.len() > RELEASE_RUN_LEDGER_MAX_BYTES as u64 {
        return Err("release-run ledger exceeds the reviewed byte bound".into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
