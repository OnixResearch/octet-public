use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;

const BLAKE3_HEX_LENGTH: usize = 64;
const EXPECTED_KANBAN_COLUMNS: usize = 5;

#[test]
fn static_lifecycle_views_are_deterministic_and_read_only() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_views.verification]
    // r[verify cairn.lifecycle_views.verification.positive]
    // r[verify cairn.lifecycle_views.cli.formats]
    let fixture = view_fixture("static")?;
    let tasks_path = fixture.join(".cairn/changes/demo/tasks.md");
    let tasks_before = fs::read_to_string(&tasks_path)?;

    let first = run_json(&fixture, &["kanban"])?;
    let second = run_json(&fixture, &["kanban"])?;
    assert_eq!(first, second);
    assert_hash(&first, "input_hash");
    assert_hash(&first, "receipt_hash");
    assert_eq!(
        first["data"]["columns"]
            .as_array()
            .ok_or("kanban columns")?
            .len(),
        EXPECTED_KANBAN_COLUMNS
    );

    let mindmap = run_text(&fixture, &["mindmap", "--format", "mermaid"])?;
    assert!(mindmap.contains("mindmap"));
    assert!(mindmap.contains("input_hash:"));
    assert!(mindmap.contains("requirement: Demo requirement"));

    let graph_json = run_json(&fixture, &["graph"])?;
    let graph_json_text = serde_json::to_string(&graph_json)?;
    assert!(!graph_json_text.contains(".cairn/specs/.cairn/specs"));
    assert!(!graph_json_text.contains(".cairn/changes/demo/specs/.cairn/changes"));

    let graph = run_text(&fixture, &["graph", "--format", "dot"])?;
    assert!(graph.contains("digraph cairn_lifecycle"));
    assert!(graph.contains("implements"));
    assert!(graph.contains("receipt_hash:"));

    assert_eq!(fs::read_to_string(tasks_path)?, tasks_before);
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn invalid_view_options_and_non_terminal_interactive_mode_fail_explicitly(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_views.verification.negative]
    // r[verify cairn.lifecycle_views.terminal.read_only]
    let fixture = view_fixture("negative")?;

    let unsupported = run(&fixture, &["kanban", "--format", "dot"])?;
    assert!(!unsupported.status.success());
    assert!(String::from_utf8_lossy(&unsupported.stderr)
        .contains("unsupported format dot for lifecycle view kanban"));

    let conflict = run(&fixture, &["graph", "--interactive", "--format", "json"])?;
    assert!(!conflict.status.success());
    assert!(String::from_utf8_lossy(&conflict.stderr)
        .contains("--interactive cannot be combined with --format"));

    let non_terminal = run(&fixture, &["mindmap", "--interactive"])?;
    assert!(!non_terminal.status.success());
    assert!(String::from_utf8_lossy(&non_terminal.stderr)
        .contains("--interactive requires terminal stdin and stdout"));

    let unknown = run(&fixture, &["graph", "--unknown"])?;
    assert!(!unknown.status.success());
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("unknown view option"));

    fs::remove_dir_all(fixture)?;
    Ok(())
}

fn run_json(root: &Path, args: &[&str]) -> Result<Value, Box<dyn Error>> {
    let output = run(root, args)?;
    assert_success(&output);
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_text(root: &Path, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = run(root, args)?;
    assert_success(&output);
    Ok(String::from_utf8(output.stdout)?)
}

fn run(root: &Path, args: &[&str]) -> Result<Output, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(repo_root())
        .arg("view")
        .args(args)
        .arg("--root")
        .arg(root)
        .arg("--policy")
        .arg(policy_path());
    Ok(command.output()?)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_hash(value: &Value, field: &str) {
    let hash = value[field].as_str().expect("hash field");
    assert_eq!(hash.len(), BLAKE3_HEX_LENGTH);
    assert!(hash.chars().all(|character| character.is_ascii_hexdigit()));
}

fn view_fixture(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = env::temp_dir().join(format!(
        "cairn-lifecycle-view-{name}-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(root.join(".cairn/specs/demo"))?;
    fs::create_dir_all(root.join(".cairn/changes/demo/specs/demo"))?;
    fs::create_dir_all(root.join(".cairn/archive"))?;
    fs::create_dir_all(root.join("crates"))?;
    fs::write(
        root.join(".cairn/specs/demo/spec.md"),
        "# Demo Specification\n\n## Requirements\n\n### Requirement: Demo requirement\nr[demo.requirement] The system MUST expose a view.\n\n#### Scenario: Demo scenario\nr[demo.requirement.scenario]\n- GIVEN lifecycle state\n- WHEN Cairn renders it\n- THEN the view is deterministic\n",
    )?;
    fs::write(
        root.join(".cairn/changes/demo/proposal.md"),
        "## Why\n\nDemonstrate lifecycle views.\n",
    )?;
    fs::write(
        root.join(".cairn/changes/demo/design.md"),
        "## Context\n\nUse a deterministic projection.\n",
    )?;
    fs::write(
        root.join(".cairn/changes/demo/tasks.md"),
        "- [~] [serial] Implement the demo. r[demo.requirement]\n",
    )?;
    fs::write(
        root.join(".cairn/changes/demo/specs/demo/spec.md"),
        "## ADDED Requirements\n\n### Requirement: Proposed detail\nr[demo.proposed] The system MUST preserve proposed facts.\n\n#### Scenario: Proposed scenario\nr[demo.proposed.scenario]\n- GIVEN a proposed requirement\n- WHEN Cairn renders it\n- THEN the change owns its delta spec\n",
    )?;
    fs::write(
        root.join("crates/demo.rs"),
        [
            format!("// {}[impl demo.requirement]", "r"),
            format!("// {}[verify demo.requirement.scenario]", "r"),
        ]
        .join("\n"),
    )?;
    Ok(root)
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy_path() -> PathBuf {
    repo_root().join("cairn-policy/generated/cairn-policy.json")
}
