use std::{env, error::Error, path::Path};

mod adr;
mod agent_guidance_shell;
mod agent_receipt;
mod archive_compaction;
mod authenticated_inputs;
mod cross_repo_dependencies;
mod determinism;
mod developer_experience;
mod developer_loop;
mod evidence;
mod evidence_manifest;
mod evidence_registry;
mod function_address;
mod generated_agent_workflow_shell;
mod global_registry;
mod hash_algorithm_policy;
mod lifecycle;
mod lifecycle_data;
mod lifecycle_store;
mod lifecycle_store_shell;
mod machine;
mod mcp;
mod migration;
mod native;
mod nickel_export_boundary;
mod oxide_lifecycle_patterns;
mod plan_receipts;
mod policy;
mod policy_exception;
mod preserves_corpus;
mod probe;
mod radicle_change;
mod receipt_preserves_diff;
mod receipt_verify_content_hash;
mod release_decision;
mod release_evidence;
mod release_readiness;
mod release_run;
mod rename_classification;
mod requirement_id_registry;
mod rfd;
mod runtime_evidence;
mod semantic_review;
mod stack_provenance;
mod state_machine_receipt;
mod steel_orchestration;
mod support;
mod test_harness;
mod toolchain_profile_matrix;
mod tracey;
mod view;
mod vo;
mod workflow;
mod worksets;
mod workspace_contract_matrix;

// r[impl cli_maintainability.main_compat_release_followup.focused_modules]
// r[impl cli_maintainability.main_compat_release_followup.focused_modules.visible_boundaries]
// r[impl cli_maintainability.main_compat_release_followup.focused_modules.narrow_bridges]
mod collaboration;
mod config_export;
mod content_addressed_evidence;

use adr::run_adr;
use collaboration::run_collaboration;
use determinism::run_determinism;
use developer_loop::run_developer_loop;
use evidence::run_evidence;
use evidence_manifest::run_evidence_manifest;
use generated_agent_workflow_shell::run_agent;
use hash_algorithm_policy::run_hash_policy;
use lifecycle_store_shell::{prepare_invocation, run_context};
use machine::{
    run_machine, MachineRunError, MACHINE_BOOTSTRAP_POLICY_STATUS, MACHINE_HOST_OUTPUT_STATUS,
};
use mcp::run_mcp;
use native::{run_change, run_doctor, run_init, run_spec, run_status, run_validate};
use policy::run_policy;
use policy_exception::run_policy_exception;
use radicle_change::run_cob;
use receipt_preserves_diff::run_receipt_preserves_diff;
use receipt_verify_content_hash::run_receipt_verify_content_hash;
use release_decision::run_release;
use requirement_id_registry::run_requirement_registry;
use rfd::run_rfd;
use runtime_evidence::run_runtime_evidence;
use semantic_review::run_review;
use state_machine_receipt::run_state_machine_receipt;
use support::{print_usage, read_option, reject_legacy_only_lifecycle_root, usage_text};
use toolchain_profile_matrix::run_toolchain_profile;
use tracey::{run_traceability, run_tracey};
use view::run_view;
use vo::run_vo;
use workflow::run_workflow;
use workspace_contract_matrix::run_workspace_contract;

fn main() {
    let invoked_as = env::args().next().unwrap_or_default();
    let all_args = env::args().skip(1).collect::<Vec<_>>();
    if all_args.first().map(String::as_str) == Some("--machine") {
        finish_machine(run_machine(all_args.into_iter().skip(1).collect()));
        return;
    }
    if invoked_as.ends_with("ironspec") {
        eprintln!("warning: `ironspec` is a compatibility alias; use canonical `cairn`");
    }
    if let Err(err) = run(all_args) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn finish_machine(result: Result<u8, MachineRunError>) {
    let status = match result {
        Ok(status) => status,
        Err(MachineRunError::PolicyUnavailable) => {
            eprintln!("error: cairn.cli.policy_error");
            MACHINE_BOOTSTRAP_POLICY_STATUS
        }
        Err(MachineRunError::HostOutputFailure) => MACHINE_HOST_OUTPUT_STATUS,
    };
    if status != 0 {
        std::process::exit(i32::from(status));
    }
}

// r[impl cairn.lifecycle_directory.legacy_diagnostic.normal_command]
fn command_requires_native_lifecycle(command: &str) -> bool {
    matches!(
        command,
        "init"
            | "change"
            | "compact"
            | "template"
            | "scaffold"
            | "discover-specs"
            | "spec"
            | "status"
            | "context"
            | "doctor"
            | "validate"
            | "view"
            | "workflow"
            | "release-readiness"
            | "lifecycle"
            | "traceability"
            | "tracey"
            | "gate"
            | "gate-context"
            | "sync"
            | "archive"
    )
}

fn run(all_args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = all_args.first() else {
        print_usage();
        return Ok(());
    };
    let prepared = prepare_invocation(command, all_args[1..].to_vec())?;
    if command_requires_native_lifecycle(command) {
        let selected_root = read_option(prepared.args.clone(), "--root", ".");
        reject_legacy_only_lifecycle_root(Path::new(&selected_root))?;
    }
    let root_local = prepared.root_local;
    let mut args = prepared.args.into_iter();

    match command.as_str() {
        "--help" | "-h" | "help" => print_usage(),
        "init" => run_init(args.collect())?,
        "change" => run_change(args.collect())?,
        "compact" => archive_compaction::run_compact(args.collect())?,
        "template" => developer_experience::run_template(args.collect())?,
        "scaffold" => developer_experience::run_scaffold(args.collect())?,
        "discover-specs" => developer_experience::run_discover_specs(args.collect())?,
        "spec" => run_spec(
            args.collect(),
            root_local
                .as_ref()
                .ok_or("spec root resolution is unavailable")?,
        )?,
        "status" => run_status(
            args.collect(),
            root_local
                .as_ref()
                .ok_or("status root resolution is unavailable")?,
        )?,
        "store" => lifecycle_store::run_store(args.collect())?,
        "context" => run_context(
            args.collect(),
            root_local
                .as_ref()
                .ok_or("context root resolution is unavailable")?,
        )?,
        "config" => config_export::run_config(args.collect())?,
        "doctor" => run_doctor(
            args.collect(),
            root_local
                .as_ref()
                .ok_or("doctor root resolution is unavailable")?,
        )?,
        "validate" => run_validate(
            args.collect(),
            root_local
                .as_ref()
                .ok_or("validate root resolution is unavailable")?,
        )?,
        "view" => run_view(args.collect())?,
        "workflow" => run_workflow(args.collect(), root_local.as_ref())?,
        "workset" => worksets::run_workset(args.collect())?,
        "release-readiness" => release_readiness::run_release_readiness(args.collect())?,
        "release-decision" => run_release(args.collect())?,
        "release-evidence" => release_evidence::run_release_evidence(args.collect())?,
        "release-run" => release_run::run_release_run(args.collect())?,
        "lifecycle" => lifecycle_data::run_lifecycle_data(args.collect())?,
        "lifecycle-patterns" => {
            oxide_lifecycle_patterns::run_oxide_lifecycle_patterns(args.collect())?
        }
        "determinism" => run_determinism(args.collect())?,
        "developer-loop" => run_developer_loop(args.collect())?,
        "agent" => run_agent(args.collect())?,
        "adr" => run_adr(args.collect())?,
        "agent-receipt" => agent_receipt::run_agent_receipt(args.collect())?,
        "evidence" => run_evidence(args.collect())?,
        "evidence-manifest" => run_evidence_manifest(args.collect())?,
        "hash-policy" => run_hash_policy(args.collect())?,
        "registry" => global_registry::run_global_registry(args.collect())?,
        "requirement-registry" => run_requirement_registry(args.collect())?,
        "policy" => run_policy(args.collect())?,
        "policy-exception" => run_policy_exception(args.collect())?,
        "preserves-corpus" => preserves_corpus::run_preserves_corpus(args.collect())?,
        "probe" => probe::run_probe(args.collect())?,
        "runtime-evidence" => run_runtime_evidence(args.collect())?,
        "rfd" => run_rfd(args.collect())?,
        "verification-obligation" => run_vo(args.collect())?,
        "collaboration" => run_collaboration(args.collect())?,
        "cob" => run_cob(args.collect())?,
        "radicle" => {
            return Err("the Radicle runtime command was removed; use `cairn cob` with an explicit Git repository".into());
        }
        "state-machine-receipt" => run_state_machine_receipt(args.collect())?,
        "traceability" => run_traceability(args.collect())?,
        "tracey" => run_tracey(args.collect())?,
        "review" => run_review(args.collect())?,
        "steel-orchestration" => steel_orchestration::run_steel_orchestration(args.collect())?,
        "test-harness" => test_harness::run_test_harness(args.collect())?,
        "toolchain-profile" => run_toolchain_profile(args.collect())?,
        "workspace-contract" => run_workspace_contract(args.collect())?,
        "rename-classification" => {
            rename_classification::run_rename_classification(args.collect())?
        }
        "migrate" => migration::run_migrate(args.collect())?,
        "mcp" => run_mcp(args.collect())?,
        "gate" => lifecycle::run_gate(args.collect())?,
        "gate-context" => {
            let Some(change) = args.next() else {
                return Err("gate-context requires a change name".into());
            };
            lifecycle::run_change_context(args.collect(), &change)?;
        }
        "sync" => {
            let Some(change) = args.next() else {
                return Err("sync requires a change name".into());
            };
            lifecycle::run_sync(args.collect(), &change)?;
        }
        "archive" => {
            let Some(change) = args.next() else {
                return Err("archive requires a change name".into());
            };
            lifecycle::run_archive(args.collect(), &change)?;
        }
        "receipt-preserves-diff" => run_receipt_preserves_diff(args.collect())?,
        "receipt-verify-content-hash" => run_receipt_verify_content_hash(args.collect())?,
        unknown => return Err(format!("unknown command: {unknown}\n\n{}", usage_text()).into()),
    }
    Ok(())
}
