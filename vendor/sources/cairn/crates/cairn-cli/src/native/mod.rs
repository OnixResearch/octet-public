// r[impl cairn.supersede.rust_nickel_only_gates.gates]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.focused_modules]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.behavior_preserved]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.native_lifecycle_source_submodule_split.behavior_preserved.repository_checks]

use std::error::Error;

mod authoring;
mod json;
mod read;
mod validate;

pub(crate) use authoring::{create_change_with_profile, write_new_file};
pub(crate) use json::{
    change_dependency_edge_json, change_dependency_graph_json, change_dependency_issue_json,
    change_dependency_node_json, change_detail_json, change_status_json, doctor_report_json,
    repo_status_json, spec_detail_json,
};
pub(crate) use validate::{
    cairn_validation_receipt, cairn_validation_receipt_with_cross_repo_evidence,
};

pub(crate) fn run_status(
    args: Vec<String>,
    invocation: &crate::lifecycle_store_shell::RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    read::run_status(args, invocation)
}

pub(crate) fn run_doctor(
    args: Vec<String>,
    invocation: &crate::lifecycle_store_shell::RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    read::run_doctor(args, invocation)
}

pub(crate) fn run_change(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err(
            "change requires a subcommand: create, depend, graph, list, next, or show".into(),
        );
    };
    match subcommand.as_str() {
        "create" => {
            let Some(change) = iter.next() else {
                return Err("change create requires a change name".into());
            };
            authoring::run_change_create(iter.collect(), &change)
        }
        "depend" => authoring::run_change_depend(iter.collect()),
        "list" => read::run_change_list(iter.collect()),
        "graph" => read::run_change_graph(iter.collect()),
        "next" => read::run_change_next(iter.collect()),
        "show" => {
            let Some(change) = iter.next() else {
                return Err("change show requires a change name".into());
            };
            read::run_change_show(iter.collect(), &change)
        }
        unknown => Err(format!("unknown change subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_spec(
    args: Vec<String>,
    invocation: &crate::lifecycle_store_shell::RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("spec requires a subcommand: list or show".into());
    };
    match subcommand.as_str() {
        "list" => read::run_spec_list(iter.collect(), invocation),
        "show" => {
            let Some(spec) = iter.next() else {
                return Err("spec show requires a spec id".into());
            };
            read::run_spec_show(iter.collect(), &spec, invocation)
        }
        unknown => Err(format!("unknown spec subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_validate(
    args: Vec<String>,
    invocation: &crate::lifecycle_store_shell::RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    validate::run_validate(args, invocation)
}

pub(crate) fn run_init(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    authoring::run_init(args)
}
