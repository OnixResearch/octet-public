use std::{error::Error, io::IsTerminal, path::Path};

use serde_json::Value;

mod json;
mod model;
mod render;
mod terminal;

use crate::support::{default_policy_path, has_flag, print_json, read_option, read_string_option};

use self::{
    json::lifecycle_view_receipt,
    model::load_lifecycle_view_model,
    render::{
        render_graph_dot, render_graph_mermaid, render_kanban_markdown, render_mindmap_mermaid,
    },
    terminal::run_terminal_view,
};

const DEFAULT_VIEW_FORMAT: &str = "json";

pub(crate) fn lifecycle_view_receipt_for_replay(
    root: &Path,
    view: &str,
) -> Result<Value, Box<dyn Error>> {
    let model = load_lifecycle_view_model(root, Path::new(default_policy_path()))?;
    Ok(lifecycle_view_receipt(root, view, &model))
}

pub(crate) fn run_view(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.cli]
    let Some(view) = args.first().map(String::as_str) else {
        return Err("view requires a kind: kanban, mindmap, or graph".into());
    };
    if !matches!(view, "kanban" | "mindmap" | "graph") {
        return Err(format!("unknown lifecycle view: {view}").into());
    }
    validate_view_args(&args[1..])?;
    let options = args[1..].to_vec();
    let root = read_option(options.clone(), "--root", ".");
    let policy_path = read_option(options.clone(), "--policy", default_policy_path());
    let interactive = has_flag(&options, "--interactive");
    let format_is_explicit = has_flag(&options, "--format");
    if interactive && format_is_explicit {
        return Err("--interactive cannot be combined with --format".into());
    }
    let format =
        read_string_option(&options, "--format").unwrap_or_else(|| DEFAULT_VIEW_FORMAT.to_string());
    let model = load_lifecycle_view_model(&root, &policy_path)?;
    let receipt = lifecycle_view_receipt(&root, view, &model);
    if interactive {
        if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
            return Err("--interactive requires terminal stdin and stdout".into());
        }
        return run_terminal_view(view, &model);
    }
    match (view, format.as_str()) {
        (_, "json") => print_json(receipt),
        ("kanban", "markdown") => {
            print!("{}", render_kanban_markdown(&model, &receipt));
            Ok(())
        }
        ("mindmap", "mermaid") => {
            print!("{}", render_mindmap_mermaid(&model, &receipt));
            Ok(())
        }
        ("graph", "mermaid") => {
            print!("{}", render_graph_mermaid(&model, &receipt));
            Ok(())
        }
        ("graph", "dot") => {
            print!("{}", render_graph_dot(&model, &receipt));
            Ok(())
        }
        _ => Err(format!("unsupported format {format} for lifecycle view {view}").into()),
    }
}

fn validate_view_args(args: &[String]) -> Result<(), Box<dyn Error>> {
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--interactive" => {
                index = index.saturating_add(1);
            }
            "--root" | "--policy" | "--format" => {
                let option = &args[index];
                let value_index = index.saturating_add(1);
                let Some(value) = args.get(value_index) else {
                    return Err(format!("{option} requires a value").into());
                };
                if value.starts_with("--") {
                    return Err(format!("{option} requires a value").into());
                }
                index = value_index.saturating_add(1);
            }
            unknown => return Err(format!("unknown view option: {unknown}").into()),
        }
    }
    Ok(())
}
