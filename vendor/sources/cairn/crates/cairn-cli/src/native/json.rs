use cairn_core::{
    layout_name, ChangeDependencyEdge, ChangeDependencyGraph, ChangeDependencyIssue,
    ChangeDependencyNode, ChangeDetail, ChangeList, ChangeStatus, DoctorCheckStatus, DoctorReport,
    LifecycleLayout, RepoStatus, SpecDetail, SpecList, SpecSummary, TaskCounts,
};
use serde_json::{json, Value};

pub(crate) fn repo_status_json(status: &RepoStatus, layout: LifecycleLayout) -> Value {
    json!({
        "root": status.root,
        "layout": layout_name(layout),
        "specs": status.specs,
        "changes": status.changes.iter().map(change_status_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn doctor_report_json(report: &DoctorReport) -> Value {
    json!({
        "valid": report.valid,
        "root": report.root,
        "layout": report.layout,
        "spec_count": report.spec_count,
        "change_count": report.change_count,
        "validation_issue_count": report.validation_issue_count,
        "checks": report.checks.iter().map(|check| {
            json!({
                "name": check.name,
                "status": doctor_check_status(check.status),
                "message": check.message,
            })
        }).collect::<Vec<_>>(),
    })
}

fn doctor_check_status(status: DoctorCheckStatus) -> &'static str {
    match status {
        DoctorCheckStatus::Ok => "ok",
        DoctorCheckStatus::Warn => "warn",
        DoctorCheckStatus::Fail => "fail",
    }
}

pub(crate) fn change_list_json(root: &str, layout: LifecycleLayout, list: &ChangeList) -> Value {
    json!({
        "root": root,
        "layout": layout_name(layout),
        "changes": list.changes.iter().map(change_status_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn change_detail_json(
    root: &str,
    layout: LifecycleLayout,
    detail: &ChangeDetail,
) -> Value {
    json!({
        "root": root,
        "layout": layout_name(layout),
        "change": change_status_json(&detail.change),
    })
}

pub(crate) fn spec_list_json(root: &str, layout: LifecycleLayout, list: &SpecList) -> Value {
    json!({
        "root": root,
        "layout": layout_name(layout),
        "specs": list.specs.iter().map(spec_summary_json).collect::<Vec<_>>(),
    })
}

pub(crate) fn spec_detail_json(root: &str, layout: LifecycleLayout, detail: &SpecDetail) -> Value {
    json!({
        "root": root,
        "layout": layout_name(layout),
        "spec": {
            "id": detail.id,
            "path": detail.path,
            "title": detail.title,
            "markdown": detail.markdown,
        }
    })
}

fn spec_summary_json(summary: &SpecSummary) -> Value {
    json!({
        "id": summary.id,
        "path": summary.path,
        "title": summary.title,
    })
}

pub(crate) fn change_dependency_graph_json(
    root: &str,
    layout: LifecycleLayout,
    archived_changes: &[String],
    graph: &ChangeDependencyGraph,
) -> Value {
    json!({
        "command": "change graph",
        "root": root,
        "layout": layout_name(layout),
        "valid": graph.valid,
        "active_change_count": graph.nodes.len(),
        "archived_changes": archived_changes,
        "nodes": graph.nodes.iter().map(change_dependency_node_json).collect::<Vec<_>>(),
        "edges": graph.edges.iter().map(change_dependency_edge_json).collect::<Vec<_>>(),
        "issues": graph.issues.iter().map(change_dependency_issue_json).collect::<Vec<_>>(),
        "drain_order": graph.drain_order,
    })
}

pub(crate) fn change_dependency_node_json(node: &ChangeDependencyNode) -> Value {
    json!({
        "name": node.name,
        "dependencies": node.dependencies,
        "satisfied_dependencies": node.satisfied_dependencies,
        "blocked_by": node.blocked_by,
        "blocks": node.blocks,
        "dependency_ready": node.dependency_ready,
    })
}

pub(crate) fn change_dependency_edge_json(edge: &ChangeDependencyEdge) -> Value {
    json!({
        "dependent": edge.dependent,
        "dependency": edge.dependency,
        "satisfied": edge.satisfied,
    })
}

pub(crate) fn change_dependency_issue_json(issue: &ChangeDependencyIssue) -> Value {
    json!({
        "change": issue.change,
        "code": issue.code,
        "dependency": issue.dependency,
        "message": issue.message,
    })
}

pub(crate) fn change_status_json(change: &ChangeStatus) -> Value {
    json!({
        "name": change.name,
        "path": change.path,
        "has_proposal": change.has_proposal,
        "has_design": change.has_design,
        "has_tasks": change.has_tasks,
        "delta_specs": change.delta_specs,
        "task_counts": change.task_counts.map(task_counts_json),
        "metadata": change_metadata_json(change),
    })
}

fn change_metadata_json(change: &ChangeStatus) -> Value {
    let mut value = json!({
        "groups": change.metadata.groups,
        "target_date": change.metadata.target_date,
        "status": change.metadata.status,
    });
    if !change.metadata.depends_on.is_empty() {
        value["depends_on"] = json!(change.metadata.depends_on);
    }
    if !change.metadata.cross_repo_dependencies.is_empty() {
        value["cross_repo_dependencies"] = json!(change.metadata.cross_repo_dependencies);
    }
    value
}

fn task_counts_json(counts: TaskCounts) -> Value {
    json!({
        "total": counts.total,
        "done": counts.done,
        "in_progress": counts.in_progress,
        "todo": counts.todo,
        "unmarked": counts.unmarked,
    })
}
