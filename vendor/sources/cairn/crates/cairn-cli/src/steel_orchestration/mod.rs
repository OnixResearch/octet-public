use std::{error::Error, path::PathBuf};

mod evaluation;
mod profiles;
mod receipts;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, read_string_option,
};
use receipts::{default_gate_receipt, orchestration_receipt};

// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split]
// r[impl cairn.lifecycle_orchestration.steel_lifecycle_orchestration.boundary_split.dispatch]
pub(crate) fn run_steel_orchestration(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(steel_usage().into());
    };

    match command {
        "receipt" => run_receipt(&args[1..]),
        "default-gate" => run_default_gate(&args[1..]),
        "--help" | "-h" | "help" => {
            println!("{}", steel_usage());
            Ok(())
        }
        unknown => Err(format!(
            "unknown steel-orchestration command: {unknown}\n\n{}",
            steel_usage()
        )
        .into()),
    }
}

fn run_default_gate(args: &[String]) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let policy_path = read_option(
        args.to_vec(),
        "--policy",
        &root.join(default_policy_path()).to_string_lossy(),
    );
    let policy = load_policy_file(policy_path)?;
    let requested_profile_id = read_string_option(args, "--profile");
    let receipt = default_gate_receipt(
        args,
        &policy.steel_orchestration_policy.profiles,
        requested_profile_id.as_deref(),
        policy.steel_orchestration_policy.enabled,
    );
    print_json(receipt)
}

fn run_receipt(args: &[String]) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let policy_path = read_option(
        args.to_vec(),
        "--policy",
        &root.join(default_policy_path()).to_string_lossy(),
    );
    let policy = load_policy_file(policy_path)?;
    let requested_profile_id = read_string_option(args, "--profile");
    let requested_script_hash = read_string_option(args, "--script-hash");
    let script_path = read_string_option(args, "--script").map(PathBuf::from);
    let receipt = orchestration_receipt(
        &policy.steel_orchestration_policy.profiles,
        requested_profile_id.as_deref(),
        requested_script_hash.as_deref(),
        script_path.as_ref(),
        policy.steel_orchestration_policy.enabled,
    );
    print_json(receipt)
}

fn steel_usage() -> &'static str {
    "usage: cairn steel-orchestration <receipt|default-gate> [--root PATH] [--policy PATH] [--profile ID] [--script PATH] [--script-hash HASH] [--equivalence-receipt-hash HASH] [--fallback-receipt-hash HASH] [--ci-reproducibility-receipt-hash HASH] [--diagnostics-receipt-hash HASH]"
}
