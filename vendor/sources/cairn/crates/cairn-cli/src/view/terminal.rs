use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    io::{self, Stdout},
};

use cairn_core::verified::lifecycle_views::{
    KanbanColumnKind, LifecycleMindmapNode, LifecycleViewEdgeKind, LifecycleViewModel,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rat_nodegraph::{
    layout::{auto_layout, LayoutConfig, LayoutDirection},
    model::{Graph, NodeId, PortSpec},
    view::{NodeGraphState, NodeGraphWidget},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

const KANBAN_COLUMN_COUNT: u32 = 5;
const GRAPH_PAN_STEP: i32 = 4;
const GRAPH_LABEL_MAX_CHARS: usize = 48;
const GRAPH_PORT_TYPE: &str = "lifecycle";
const GRAPH_INPUT_PORT: &str = "in";
const GRAPH_OUTPUT_PORT: &str = "out";

pub(super) fn run_terminal_view(
    view: &str,
    model: &LifecycleViewModel,
) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.terminal]
    // r[impl cairn.lifecycle_views.terminal.read_only]
    // r[impl cairn.lifecycle_views.determinism.display_state]
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let _restore = TerminalRestore;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    match view {
        "kanban" => run_kanban_loop(&mut terminal, model),
        "mindmap" => run_graph_loop(&mut terminal, model, true),
        "graph" => run_graph_loop(&mut terminal, model, false),
        unknown => Err(format!("unsupported interactive lifecycle view: {unknown}").into()),
    }
}

struct TerminalRestore;

impl Drop for TerminalRestore {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

type CairnTerminal = Terminal<CrosstermBackend<Stdout>>;

fn run_kanban_loop(
    terminal: &mut CairnTerminal,
    model: &LifecycleViewModel,
) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|frame| render_kanban(frame, model))?;
        if quit_event(event::read()?) {
            return Ok(());
        }
    }
}

fn run_graph_loop(
    terminal: &mut CairnTerminal,
    model: &LifecycleViewModel,
    mindmap_only: bool,
) -> Result<(), Box<dyn Error>> {
    let graph = adapt_node_graph(model, mindmap_only)?;
    let size = terminal.size()?;
    let mut state = NodeGraphState::new(graph, size.width, size.height);
    loop {
        terminal.draw(|frame| render_graph(frame, &mut state, mindmap_only))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Left => state.viewport.pan(-GRAPH_PAN_STEP, 0),
            KeyCode::Right => state.viewport.pan(GRAPH_PAN_STEP, 0),
            KeyCode::Up => state.viewport.pan(0, -GRAPH_PAN_STEP),
            KeyCode::Down => state.viewport.pan(0, GRAPH_PAN_STEP),
            KeyCode::Char('+') | KeyCode::Char('=') => state.viewport.zoom_in(),
            KeyCode::Char('-') => state.viewport.zoom_out(),
            KeyCode::Char('0') => state.viewport.reset_zoom(),
            KeyCode::Tab => {
                let _selection_only_actions = state.handle_key("Tab", false);
            }
            _ => {}
        }
    }
}

fn quit_event(event: Event) -> bool {
    matches!(
        event,
        Event::Key(key) if matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
    )
}

fn render_kanban(frame: &mut Frame<'_>, model: &LifecycleViewModel) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    frame.render_widget(
        Paragraph::new("Cairn kanban — read-only lifecycle projection")
            .style(Style::default().add_modifier(Modifier::BOLD)),
        header,
    );
    let column_areas = Layout::horizontal(
        model
            .kanban
            .columns
            .iter()
            .map(|_column| Constraint::Ratio(1, KANBAN_COLUMN_COUNT))
            .collect::<Vec<_>>(),
    )
    .split(body);
    for (column, area) in model.kanban.columns.iter().zip(column_areas.iter()) {
        let title = format!("{} ({})", column.kind.as_str(), column.cards.len());
        let lines = column
            .cards
            .iter()
            .flat_map(|card| {
                let progress = card
                    .task_counts
                    .map(|counts| format!("{}/{}", counts.done, counts.total))
                    .unwrap_or_else(|| "no tasks".to_string());
                [
                    Line::styled(
                        card.change.clone(),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Line::from(format!("  {progress}")),
                ]
            })
            .collect::<Vec<_>>();
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(kanban_style(column.kind));
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(block)
                .wrap(Wrap { trim: true }),
            *area,
        );
    }
    frame.render_widget(
        Paragraph::new("q/Esc quit • visual state is excluded from Cairn receipts")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}

fn kanban_style(kind: KanbanColumnKind) -> Style {
    let color = match kind {
        KanbanColumnKind::Planned => Color::DarkGray,
        KanbanColumnKind::Ready => Color::Cyan,
        KanbanColumnKind::InProgress => Color::Yellow,
        KanbanColumnKind::Blocked => Color::Red,
        KanbanColumnKind::ReadyToArchive => Color::Green,
    };
    Style::default().fg(color)
}

fn render_graph(frame: &mut Frame<'_>, state: &mut NodeGraphState, mindmap_only: bool) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let title = if mindmap_only {
        "Cairn mindmap — read-only containment projection"
    } else {
        "Cairn knowledge graph — read-only typed-link projection"
    };
    frame.render_widget(
        Paragraph::new(title).style(Style::default().add_modifier(Modifier::BOLD)),
        header,
    );
    frame.render_stateful_widget(NodeGraphWidget::default(), body, state);
    frame.render_widget(
        Paragraph::new("arrows pan • +/- zoom • 0 reset • Tab select • q/Esc quit")
            .style(Style::default().fg(Color::DarkGray)),
        footer,
    );
}

fn adapt_node_graph(
    model: &LifecycleViewModel,
    mindmap_only: bool,
) -> Result<Graph, Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.terminal.subwayrat]
    let included = if mindmap_only {
        mindmap_ids(&model.mindmap)
    } else {
        model
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<BTreeSet<_>>()
    };
    let mut graph = Graph::new();
    graph.dag_mode = mindmap_only;
    let mut node_ids = BTreeMap::<String, NodeId>::new();
    for node in model
        .nodes
        .iter()
        .filter(|node| included.contains(&node.id))
    {
        let label = truncate_label(&format!("{}: {}", node.kind.as_str(), node.label));
        let node_id = graph.add_node(
            label,
            &[PortSpec::new(GRAPH_INPUT_PORT, GRAPH_PORT_TYPE)],
            &[PortSpec::new(GRAPH_OUTPUT_PORT, GRAPH_PORT_TYPE)],
        );
        node_ids.insert(node.id.clone(), node_id);
    }
    for edge in model
        .edges
        .iter()
        .filter(|edge| !mindmap_only || edge.kind == LifecycleViewEdgeKind::Contains)
    {
        let source = node_ids
            .get(&edge.source)
            .copied()
            .ok_or_else(|| format!("view edge source node is missing: {}", edge.source))?;
        let target = node_ids
            .get(&edge.target)
            .copied()
            .ok_or_else(|| format!("view edge target node is missing: {}", edge.target))?;
        let source_port = graph
            .node(source)
            .and_then(|node| node.output_ports.first())
            .map(|port| port.id)
            .ok_or("view source node has no output port")?;
        let target_port = graph
            .node(target)
            .and_then(|node| node.input_ports.first())
            .map(|port| port.id)
            .ok_or("view target node has no input port")?;
        graph.add_edge(source_port, target_port)?;
    }
    let direction = if mindmap_only {
        LayoutDirection::TopToBottom
    } else {
        LayoutDirection::LeftToRight
    };
    auto_layout(
        &mut graph,
        &LayoutConfig {
            direction,
            ..LayoutConfig::default()
        },
    );
    Ok(graph)
}

fn mindmap_ids(root: &LifecycleMindmapNode) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        ids.insert(node.id.clone());
        pending.extend(node.children.iter());
    }
    ids
}

fn truncate_label(label: &str) -> String {
    let mut characters = label.chars();
    let truncated = characters
        .by_ref()
        .take(GRAPH_LABEL_MAX_CHARS)
        .collect::<String>();
    if characters.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use cairn_core::verified::lifecycle_views::{
        KanbanBoard, KanbanColumn, KanbanColumnKind, LifecycleMindmapNode, LifecycleViewEdge,
        LifecycleViewEdgeKind, LifecycleViewModel, LifecycleViewNode, LifecycleViewNodeKind,
    };

    use super::adapt_node_graph;

    const EXPECTED_ADAPTED_NODE_COUNT: usize = 2;

    #[test]
    fn subwayrat_adapter_preserves_read_only_semantic_nodes_and_edges() {
        // r[verify cairn.lifecycle_views.verification.positive]
        // r[verify cairn.lifecycle_views.terminal.subwayrat]
        let model = model_with_edge("change:demo");
        let graph = adapt_node_graph(&model, false).expect("adapt graph");
        assert_eq!(graph.node_count(), EXPECTED_ADAPTED_NODE_COUNT);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn subwayrat_adapter_rejects_edge_to_missing_node() {
        // r[verify cairn.lifecycle_views.verification.negative]
        let model = model_with_edge("change:missing");
        let error = adapt_node_graph(&model, false).expect_err("missing target must fail");
        assert!(error.to_string().contains("target node is missing"));
    }

    fn model_with_edge(target: &str) -> LifecycleViewModel {
        let root = LifecycleViewNode {
            id: "repository:root".to_string(),
            kind: LifecycleViewNodeKind::Repository,
            label: "repository".to_string(),
            reference: None,
            state: None,
            path: None,
        };
        let change = LifecycleViewNode {
            id: "change:demo".to_string(),
            kind: LifecycleViewNodeKind::Change,
            label: "demo".to_string(),
            reference: Some("demo".to_string()),
            state: Some("current".to_string()),
            path: Some(".cairn/changes/demo".to_string()),
        };
        LifecycleViewModel {
            schema: "test".to_string(),
            repository: "repository".to_string(),
            valid: true,
            nodes: vec![root.clone(), change.clone()],
            edges: vec![LifecycleViewEdge {
                source: root.id.clone(),
                target: target.to_string(),
                kind: LifecycleViewEdgeKind::Contains,
                state: None,
                details: Vec::new(),
            }],
            issues: Vec::new(),
            kanban: KanbanBoard {
                columns: KanbanColumnKind::ORDERED
                    .iter()
                    .map(|kind| KanbanColumn {
                        kind: *kind,
                        cards: Vec::new(),
                    })
                    .collect(),
            },
            mindmap: LifecycleMindmapNode {
                id: root.id,
                kind: root.kind,
                label: root.label,
                children: vec![LifecycleMindmapNode {
                    id: change.id,
                    kind: change.kind,
                    label: change.label,
                    children: Vec::new(),
                }],
            },
            boundary: "test boundary".to_string(),
            non_claims: vec!["not correctness".to_string()],
        }
    }
}
