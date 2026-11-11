// r[impl cli_maintainability.agent_receipt_cli_source.focused_modules]
// r[impl cli_maintainability.agent_receipt_cli_source.focused_modules.visible_boundaries]
// r[impl cli_maintainability.agent_receipt_cli_source.focused_modules.narrow_bridges]
// r[impl cli_maintainability.agent_receipt_cli_source.behavior_preserved]
// r[verify cli_maintainability.agent_receipt_cli_source.behavior_preserved.focused_checks]
// r[verify cli_maintainability.agent_receipt_cli_source.behavior_preserved.repository_checks]
use std::{error::Error, path::PathBuf};

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, read_string_option,
};

mod bundle;
mod io;
mod validation;

pub(crate) use bundle::{
    agent_evidence_bundle_verification_receipt, receipt_stated_or_computed_hash,
};
pub(crate) use io::{
    ensure_inline_receipt_is_bounded, read_agent_receipt_file, read_json_artifact,
};
pub(crate) use validation::{
    agent_completion_validation_receipt, agent_preserves_hash_receipt,
    agent_receipt_validation_receipt, is_blake3_hex_text, AgentCompletionValidationInput,
};

pub(crate) const AGENT_RECEIPT_MAX_BYTES: u64 = 64 * 1024;
pub(crate) const REQUIRED_AGENT_REPLAY_GROUP: &str = "agent";

pub(crate) fn run_agent_receipt(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("agent-receipt requires a subcommand: validate".into());
    };
    match subcommand.as_str() {
        "validate" => {
            let Some(path) = iter.next() else {
                return Err("agent-receipt validate requires a JSON receipt path".into());
            };
            let path = PathBuf::from(path);
            let receipt = read_agent_receipt_file(&path)?;
            let policy = load_policy_file(default_policy_path().into())?;
            print_json(agent_receipt_validation_receipt(
                "agent-receipt validate",
                Some(path),
                &receipt,
                false,
                &policy,
            ))
        }
        "complete" => {
            let rest = iter.collect::<Vec<_>>();
            let contract_path = read_string_option(&rest, "--contract")
                .map(PathBuf::from)
                .ok_or("agent-receipt complete requires --contract FILE")?;
            let results_path = read_string_option(&rest, "--results")
                .map(PathBuf::from)
                .ok_or("agent-receipt complete requires --results FILE")?;
            let subject_hash = read_string_option(&rest, "--subject-hash")
                .ok_or("agent-receipt complete requires --subject-hash HASH")?;
            let policy_path = read_string_option(&rest, "--policy")
                .map(PathBuf::from)
                .unwrap_or_else(|| default_policy_path().into());
            let contract = read_json_artifact(&contract_path, AGENT_RECEIPT_MAX_BYTES)?;
            let results = read_json_artifact(&results_path, AGENT_RECEIPT_MAX_BYTES)?;
            let policy = load_policy_file(policy_path)?;
            print_json(agent_completion_validation_receipt(
                AgentCompletionValidationInput {
                    command: "agent-receipt complete",
                    contract_path: Some(contract_path),
                    results_path: Some(results_path),
                    contract: &contract,
                    results: &results,
                    current_subject_hash: &subject_hash,
                    policy: &policy,
                    input_kind: "path",
                },
            ))
        }
        "preserves-hash" => {
            let Some(path) = iter.next() else {
                return Err("agent-receipt preserves-hash requires a JSON receipt path".into());
            };
            let path = PathBuf::from(path);
            let receipt = read_agent_receipt_file(&path)?;
            print_json(agent_preserves_hash_receipt(path, &receipt))
        }
        "verify-bundle" => bundle::run_agent_receipt_verify_bundle(iter.collect()),
        "smoke-mcp" => {
            let rest = iter.collect::<Vec<_>>();
            let root = read_option(rest, "--root", ".");
            print_json(crate::mcp::mcp_agent_smoke_receipt(root)?)
        }
        unknown => Err(format!("unknown agent-receipt subcommand: {unknown}").into()),
    }
}
