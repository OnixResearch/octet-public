use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{canonical_json_hash, Policy};
use serde_json::{json, Value};

use crate::agent_receipt::is_blake3_hex_text;
use crate::support::{
    default_policy_path, load_policy_file, path_string, portable_json_hash, portable_text,
    print_json, read_option, with_portable_content_hash_receipt, with_portable_receipt_hash,
};

mod coverage;
mod replay;
mod verify;

pub(crate) use coverage::{determinism_coverage_audit_receipt, determinism_surfaces_from_policy};
pub(crate) use replay::{
    determinism_replay_receipt, replay_cases_from_policy, replay_groups_from_policy, ReplayCase,
    ReplayGroup,
};
pub(crate) use verify::determinism_verify_replay_receipt;

pub(crate) fn run_determinism(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("determinism requires a subcommand: replay".into());
    };
    match subcommand.as_str() {
        "coverage-audit" => coverage::run_determinism_coverage_audit(iter.collect()),
        "replay" => replay::run_determinism_replay(iter.collect()),
        "verify-receipt" => verify::run_determinism_verify_receipt(iter.collect()),
        unknown => Err(format!("unknown determinism subcommand: {unknown}").into()),
    }
}
