use std::path::Path;

use cairn_core::{
    canonical_json_hash,
    verified::lifecycle_views::{
        KanbanBoard, KanbanCard, LifecycleMindmapNode, LifecycleViewEdge, LifecycleViewIssue,
        LifecycleViewModel, LifecycleViewNode,
    },
};
use serde_json::{json, Value};

use crate::support::{path_string, with_portable_receipt_hash};

pub(super) fn lifecycle_view_receipt(root: &Path, view: &str, model: &LifecycleViewModel) -> Value {
    // r[impl cairn.lifecycle_views.determinism]
    // r[impl cairn.lifecycle_views.cli]
    let data = match view {
        "kanban" => kanban_json(&model.kanban),
        "mindmap" => mindmap_node_json(&model.mindmap),
        "graph" => graph_json(model),
        _ => Value::Null,
    };
    let semantic = json!({
        "schema": model.schema,
        "view": view,
        "valid": model.valid,
        "data": data,
        "issues": issues_json(&model.issues),
        "boundary": model.boundary,
        "non_claims": model.non_claims,
    });
    let input_hash = canonical_json_hash(&semantic);
    with_portable_receipt_hash(
        root,
        json!({
            "command": format!("view {view}"),
            "root": path_string(root),
            "schema": model.schema,
            "view": view,
            "valid": model.valid,
            "data": semantic["data"].clone(),
            "issues": semantic["issues"].clone(),
            "boundary": model.boundary,
            "non_claims": model.non_claims,
            "input_hash": input_hash,
        }),
    )
}

fn graph_json(model: &LifecycleViewModel) -> Value {
    json!({
        "nodes": model.nodes.iter().map(node_json).collect::<Vec<_>>(),
        "edges": model.edges.iter().map(edge_json).collect::<Vec<_>>(),
    })
}

fn node_json(node: &LifecycleViewNode) -> Value {
    json!({
        "id": node.id,
        "kind": node.kind.as_str(),
        "label": node.label,
        "reference": node.reference,
        "state": node.state,
        "path": node.path,
    })
}

fn edge_json(edge: &LifecycleViewEdge) -> Value {
    json!({
        "source": edge.source,
        "target": edge.target,
        "kind": edge.kind.as_str(),
        "state": edge.state,
        "details": edge.details,
    })
}

fn kanban_json(board: &KanbanBoard) -> Value {
    json!({
        "columns": board.columns.iter().map(|column| {
            json!({
                "id": column.kind.as_str(),
                "cards": column.cards.iter().map(kanban_card_json).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

fn kanban_card_json(card: &KanbanCard) -> Value {
    json!({
        "change": card.change,
        "metadata_status": card.metadata_status,
        "task_counts": card.task_counts.map(|counts| json!({
            "total": counts.total,
            "done": counts.done,
            "in_progress": counts.in_progress,
            "todo": counts.todo,
            "unmarked": counts.unmarked,
        })),
        "dependencies": card.dependencies,
        "blocked_by": card.blocked_by,
        "reasons": card.reasons,
    })
}

fn mindmap_node_json(node: &LifecycleMindmapNode) -> Value {
    json!({
        "id": node.id,
        "kind": node.kind.as_str(),
        "label": node.label,
        "children": node.children.iter().map(mindmap_node_json).collect::<Vec<_>>(),
    })
}

fn issues_json(issues: &[LifecycleViewIssue]) -> Value {
    Value::Array(
        issues
            .iter()
            .map(|issue| {
                json!({
                    "code": issue.code,
                    "message": issue.message,
                    "source": issue.source,
                    "target": issue.target,
                })
            })
            .collect(),
    )
}
