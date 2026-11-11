// r[impl release_evidence.automated_current_head_index.cli_contract]
// r[impl release_evidence.automated_current_head_index.cli_contract.inputs_outputs]
// r[impl release_evidence.automated_current_head_index.composes_existing_rails]
// r[impl release_evidence.automated_current_head_index.composes_existing_rails.authoritative_validators]
// r[impl release_evidence.automated_current_head_index.generated_index]
// r[impl release_evidence.automated_current_head_index.generated_index.payload_head]
// r[impl release_evidence.automated_current_head_index.generated_index.required_hashes]
// r[impl release_evidence.automated_current_head_index.fail_closed]
// r[impl release_evidence.automated_current_head_index.fail_closed.dirty_or_mismatched]
// r[impl release_evidence.automated_current_head_index.operator_docs]
// r[impl release_evidence.automated_current_head_index.operator_docs.non_claims]
// r[impl release_evidence.cli_source_split.focused_modules]
// r[impl release_evidence.cli_source_split.focused_modules.visible_boundaries]
// r[impl release_evidence.cli_source_split.narrow_bridges]
// r[impl release_evidence.cli_source_split.narrow_bridges.shared_helpers]
// r[impl release_evidence.cli_source_split.behavior_preserved]
use std::error::Error;

mod check_index;
mod common;
mod index;
mod publish;
mod retention;
mod rotate;

pub(crate) use check_index::release_evidence_check_index_receipt_for_path;

#[cfg(test)]
mod tests;

pub(crate) fn run_release_evidence(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err(
            "release-evidence requires a subcommand: index|check-index|publish|retention-plan|rotate".into(),
        );
    };
    match subcommand.as_str() {
        "index" => index::run_release_evidence_index(iter.collect()),
        "check-index" => check_index::run_release_evidence_check_index(iter.collect()),
        "publish" => publish::run_release_evidence_publish(iter.collect()),
        "retention-plan" => retention::run_release_evidence_retention_plan(iter.collect()),
        "rotate" => rotate::run_release_evidence_rotate(iter.collect()),
        unknown => Err(format!("unknown release-evidence subcommand: {unknown}").into()),
    }
}
