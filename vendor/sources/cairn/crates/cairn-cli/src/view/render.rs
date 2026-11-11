use std::collections::BTreeMap;

use cairn_core::verified::lifecycle_views::{
    KanbanBoard, LifecycleMindmapNode, LifecycleViewModel,
};
use serde_json::Value;

const INDENT_WIDTH: usize = 2;
const TASK_PROGRESS_SEPARATOR: &str = "/";

pub(super) fn render_kanban_markdown(model: &LifecycleViewModel, receipt: &Value) -> String {
    let mut lines = receipt_header("# Cairn Kanban", receipt, "<!--", " -->");
    append_kanban_columns(&mut lines, &model.kanban);
    render_lines(lines)
}

pub(super) fn render_mindmap_mermaid(model: &LifecycleViewModel, receipt: &Value) -> String {
    let mut lines = receipt_header("mindmap", receipt, "%%", "");
    let mut ordinal = 0usize;
    append_mindmap_node(&mut lines, &model.mindmap, 1, &mut ordinal, true);
    render_lines(lines)
}

pub(super) fn render_graph_mermaid(model: &LifecycleViewModel, receipt: &Value) -> String {
    let mut lines = receipt_header("flowchart LR", receipt, "%%", "");
    let ids = graph_render_ids(model);
    for node in &model.nodes {
        let Some(id) = ids.get(&node.id) else {
            continue;
        };
        lines.push(format!(
            "  {id}[\"{}: {}\"]",
            node.kind.as_str(),
            escape_mermaid(&node.label)
        ));
    }
    for edge in &model.edges {
        let (Some(source), Some(target)) = (ids.get(&edge.source), ids.get(&edge.target)) else {
            continue;
        };
        lines.push(format!(
            "  {source} -->|\"{}\"| {target}",
            escape_mermaid(&edge_label(edge.kind.as_str(), edge.state.as_deref()))
        ));
    }
    render_lines(lines)
}

pub(super) fn render_graph_dot(model: &LifecycleViewModel, receipt: &Value) -> String {
    let mut lines = receipt_header("digraph cairn_lifecycle {", receipt, "//", "");
    let ids = graph_render_ids(model);
    for node in &model.nodes {
        let Some(id) = ids.get(&node.id) else {
            continue;
        };
        lines.push(format!(
            "  {id} [label=\"{}: {}\"];",
            node.kind.as_str(),
            escape_dot(&node.label)
        ));
    }
    for edge in &model.edges {
        let (Some(source), Some(target)) = (ids.get(&edge.source), ids.get(&edge.target)) else {
            continue;
        };
        lines.push(format!(
            "  {source} -> {target} [label=\"{}\"];",
            escape_dot(&edge_label(edge.kind.as_str(), edge.state.as_deref()))
        ));
    }
    lines.push("}".to_string());
    render_lines(lines)
}

fn append_kanban_columns(lines: &mut Vec<String>, board: &KanbanBoard) {
    for column in &board.columns {
        lines.push(String::new());
        lines.push(format!(
            "## {} ({})",
            title_case(column.kind.as_str()),
            column.cards.len()
        ));
        if column.cards.is_empty() {
            lines.push(String::new());
            lines.push("_No active changes._".to_string());
            continue;
        }
        for card in &column.cards {
            lines.push(String::new());
            lines.push(format!("### `{}`", card.change));
            if let Some(counts) = card.task_counts {
                lines.push(format!(
                    "- Tasks: {}{TASK_PROGRESS_SEPARATOR}{} done; {} in progress; {} todo; {} unmarked",
                    counts.done, counts.total, counts.in_progress, counts.todo, counts.unmarked
                ));
            } else {
                lines.push("- Tasks: not authored".to_string());
            }
            if !card.dependencies.is_empty() {
                lines.push(format!("- Dependencies: {}", card.dependencies.join(", ")));
            }
            for reason in &card.reasons {
                lines.push(format!("- {reason}"));
            }
        }
    }
}

fn append_mindmap_node(
    lines: &mut Vec<String>,
    node: &LifecycleMindmapNode,
    depth: usize,
    ordinal: &mut usize,
    root: bool,
) {
    let indentation = " ".repeat(depth.saturating_mul(INDENT_WIDTH));
    let label = escape_mermaid(&format!("{}: {}", node.kind.as_str(), node.label));
    if root {
        lines.push(format!("{indentation}root((\"{label}\"))"));
    } else {
        *ordinal = ordinal.saturating_add(1);
        lines.push(format!("{indentation}n{ordinal}[\"{label}\"]"));
    }
    for child in &node.children {
        append_mindmap_node(lines, child, depth.saturating_add(1), ordinal, false);
    }
}

fn graph_render_ids(model: &LifecycleViewModel) -> BTreeMap<String, String> {
    model
        .nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id.clone(), format!("n{index}")))
        .collect()
}

fn edge_label(kind: &str, state: Option<&str>) -> String {
    state
        .map(|state| format!("{kind} ({state})"))
        .unwrap_or_else(|| kind.to_string())
}

fn receipt_header(title: &str, receipt: &Value, prefix: &str, suffix: &str) -> Vec<String> {
    vec![
        format!(
            "{prefix} input_hash: {}{suffix}",
            receipt["input_hash"].as_str().unwrap_or("missing")
        ),
        format!(
            "{prefix} receipt_hash: {}{suffix}",
            receipt["receipt_hash"].as_str().unwrap_or("missing")
        ),
        title.to_string(),
    ]
}

fn title_case(value: &str) -> String {
    value
        .split('_')
        .map(|word| {
            let mut characters = word.chars();
            let Some(first) = characters.next() else {
                return String::new();
            };
            format!("{}{}", first.to_ascii_uppercase(), characters.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn escape_mermaid(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace(['\n', '\r'], " ")
}

fn escape_dot(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace(['\n', '\r'], " ")
}

fn render_lines(lines: Vec<String>) -> String {
    let mut rendered = lines.join("\n");
    rendered.push('\n');
    rendered
}
