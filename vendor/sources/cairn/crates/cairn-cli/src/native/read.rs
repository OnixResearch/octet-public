use std::error::Error;

use cairn_core::{
    archive_dir, build_change_dependency_graph, build_change_detail, build_change_list,
    build_doctor_report, build_spec_detail, build_spec_list, changes_dir,
    filter_changes_by_metadata, layout_root, lifecycle_substance_issues, next_ready_change,
    render_change_dependency_graph_dot, render_change_dependency_graph_mermaid, specs_dir,
    validate_changes_with_archives, validate_spec_sources_with_policy, ChangeDependencyNextReady,
    DoctorSnapshot, LifecycleLayout,
};
use serde_json::json;

use crate::{
    lifecycle_store_shell::{
        attach_resolution, context_result_json, resolution_json, RootLocalInvocation,
    },
    support::{
        collect_archived_change_ids, collect_spec_sources, collect_validation_spec_sources,
        default_policy_path, detect_layout, find_change, find_spec_source, load_policy_file,
        path_string, print_json, read_gate_snapshot, read_option, read_string_option, scan_repo,
        with_portable_receipt_hash,
    },
};

use super::json::{
    change_dependency_graph_json, change_detail_json, change_list_json, doctor_report_json,
    repo_status_json, spec_detail_json, spec_list_json,
};

pub(crate) fn run_status(
    args: Vec<String>,
    invocation: &RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let status = scan_repo(root, selected.layout)?;
    print_json(attach_resolution(
        repo_status_json(&status, selected.layout),
        invocation,
    ))
}

pub(crate) fn run_doctor(
    args: Vec<String>,
    invocation: &RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args, "--policy", default_policy_path());
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let policy_result = load_policy_file(policy_path.clone());
    let (policy_name, policy_error, validation_issue_count) = match policy_result {
        Ok(policy) => {
            let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
            let change_issue_count =
                validate_changes_with_archives(&status.changes, &archived_changes, &policy).len();
            let spec_issue_count = validate_spec_sources_with_policy(
                &collect_validation_spec_sources(&root, selected.layout)?,
                &policy.artifact_substance_policy,
            )
            .len();
            let mut substance_issue_count = 0;
            for change in &status.changes {
                let snapshot = read_gate_snapshot(&root, selected.layout, change.clone())?;
                substance_issue_count += lifecycle_substance_issues(&snapshot, &policy).len();
            }
            (
                Some(policy.name),
                None,
                change_issue_count + spec_issue_count + substance_issue_count,
            )
        }
        Err(error) => (None, Some(error.to_string()), 0),
    };
    let snapshot = DoctorSnapshot {
        root: path_string(&root),
        selected_layout: selected.layout,
        native_layout_present: root.join(layout_root(LifecycleLayout::Native)).exists(),
        legacy_layout_present: root
            .join(layout_root(LifecycleLayout::LegacyOpenSpec))
            .exists(),
        specs_dir_present: root.join(specs_dir(selected.layout)).exists(),
        changes_dir_present: root.join(changes_dir(selected.layout)).exists(),
        archive_dir_present: root.join(archive_dir(selected.layout)).exists(),
        policy_path: path_string(policy_path),
        policy_name,
        policy_error,
        spec_count: status.specs.len(),
        change_count: status.changes.len(),
        validation_issue_count,
    };
    let report = build_doctor_report(snapshot);
    let context = invocation.context_result()?;
    let doctor_valid = report.valid && context.valid;
    let mut value = doctor_report_json(&report);
    if let Some(object) = value.as_object_mut() {
        object.insert("valid".to_string(), json!(doctor_valid));
        object.insert(
            "resolution".to_string(),
            resolution_json(&invocation.resolution),
        );
        object.insert(
            "lifecycle_store_context".to_string(),
            context_result_json(&context),
        );
    }
    print_json(value)
}

pub(super) fn run_change_list(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let group = read_string_option(&args, "--group");
    let target_after = read_string_option(&args, "--target-after");
    let target_before = read_string_option(&args, "--target-before");
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let changes = filter_changes_by_metadata(
        status.changes,
        group.as_deref(),
        target_after.as_deref(),
        target_before.as_deref(),
    );
    let list = build_change_list(changes);
    print_json(change_list_json(&path_string(root), selected.layout, &list))
}

pub(super) fn run_change_show(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, change)?;
    let detail = build_change_detail(change_status);
    print_json(change_detail_json(
        &path_string(root),
        selected.layout,
        &detail,
    ))
}

pub(super) fn run_change_graph(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.change_dependency_graph.drain_order]
    // r[impl cairn.change_dependency_graph.render_formats]
    let root = read_option(args.clone(), "--root", ".");
    let format = read_string_option(&args, "--format").unwrap_or_else(|| "json".to_string());
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let graph = build_change_dependency_graph(&status.changes, &archived_changes);
    match format.as_str() {
        "json" => print_json(change_dependency_graph_json(
            &path_string(root),
            selected.layout,
            &archived_changes,
            &graph,
        )),
        "mermaid" => {
            print!("{}", render_change_dependency_graph_mermaid(&graph));
            Ok(())
        }
        "dot" => {
            print!("{}", render_change_dependency_graph_dot(&graph));
            Ok(())
        }
        unknown => Err(format!("unknown change graph format: {unknown}").into()),
    }
}

pub(super) fn run_change_next(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.change_dependency_graph.next_ready]
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let graph = build_change_dependency_graph(&status.changes, &archived_changes);
    let summary = next_ready_change(&graph);
    let receipt = change_next_receipt(&path_string(&root), selected.layout, &summary, &graph);
    print_json(with_portable_receipt_hash(&root, receipt))
}

fn change_next_receipt(
    root: &str,
    layout: LifecycleLayout,
    summary: &ChangeDependencyNextReady,
    graph: &cairn_core::ChangeDependencyGraph,
) -> serde_json::Value {
    json!({
        "command": "change next",
        "root": root,
        "layout": cairn_core::layout_name(layout),
        "valid": graph.valid,
        "ready_change": summary.ready,
        "blocked": summary.blocked.iter().map(|blocked| {
            json!({
                "change": blocked.change,
                "blockers": blocked.blockers,
            })
        }).collect::<Vec<_>>(),
        "issues": graph.issues.iter().map(super::json::change_dependency_issue_json).collect::<Vec<_>>(),
        "drain_order": graph.drain_order,
    })
}

pub(super) fn run_spec_list(
    args: Vec<String>,
    invocation: &RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let specs = collect_spec_sources(&root.join(specs_dir(selected.layout)))?;
    let list = build_spec_list(specs);
    print_json(attach_resolution(
        spec_list_json(&path_string(root), selected.layout, &list),
        invocation,
    ))
}

pub(super) fn run_spec_show(
    args: Vec<String>,
    spec: &str,
    invocation: &RootLocalInvocation,
) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let source = find_spec_source(&root, selected.layout, spec)?;
    let detail = build_spec_detail(source);
    print_json(attach_resolution(
        spec_detail_json(&path_string(root), selected.layout, &detail),
        invocation,
    ))
}
