use std::{env, error::Error, fs};

use super::common::{assert_hash_field, copy_dir_recursive, repo_root, run_json};

#[test]
fn mcp_drain_plan_reports_blockers_without_mutation() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.mcp_drain_all.plan.read_only]
    // r[verify cairn.mcp_drain_all.plan.readiness]
    // r[verify cairn.mcp_drain_all.plan.next_actions]
    // r[verify cairn.mcp_drain_all.plan.mutation_boundary]
    let value = run_json(&[
        "mcp",
        "--call",
        "cairn_drain_plan",
        r#"{"root":"fixtures/basic"}"#,
    ])?;
    assert_eq!(
        value,
        run_json(&[
            "mcp",
            "--call",
            "cairn_drain_plan",
            r#"{"root":"fixtures/basic"}"#,
        ])?
    );
    let receipt = &value["structuredContent"];
    assert_eq!(receipt["command"], "mcp cairn_drain_plan");
    assert_eq!(receipt["mcp"]["read_only"], true);
    assert_eq!(receipt["mcp"]["destructive"], false);
    assert_eq!(receipt["active_change_count"], 1);
    assert_eq!(receipt["ready_change_count"], 0);
    let change = &receipt["changes"][0];
    assert_eq!(change["name"], "demo");
    assert_eq!(change["drain_ready"], false);
    assert_eq!(change["task_counts"]["todo"], 2);
    assert_eq!(change["gates"]["proposal"]["valid"], true);
    assert_eq!(change["gates"]["design"]["valid"], true);
    assert_eq!(change["gates"]["tasks"]["valid"], true);
    assert!(change["blockers"]
        .as_array()
        .ok_or("blockers")?
        .iter()
        .any(|blocker| blocker["class"] == "unfinished_tasks"));
    assert!(change["next_actions"]
        .as_array()
        .ok_or("next_actions")?
        .iter()
        .any(|action| action["action"]
            == "complete implementation tasks and mark tasks.md checkboxes"));
    assert!(receipt["next_actions"]
        .as_array()
        .ok_or("root next_actions")?
        .iter()
        .all(|action| action["requires_allow_mutations"].as_bool() != Some(true)));
    assert_hash_field(receipt, "receipt_hash")?;
    Ok(())
}

#[test]
fn mcp_drain_plan_recommends_explicit_mutation_tools_for_ready_changes(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.mcp_drain_all.plan.mutation_boundary]
    let temp_root = env::temp_dir().join(format!("cairn-mcp-drain-ready-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_root);
    copy_dir_recursive(&repo_root().join("fixtures/basic"), &temp_root)?;
    let tasks_path = temp_root.join(".cairn/changes/demo/tasks.md");
    let tasks = fs::read_to_string(&tasks_path)?.replace("- [ ]", "- [x]");
    fs::write(&tasks_path, tasks)?;

    let request = format!(r#"{{"root":"{}"}}"#, temp_root.display());
    let value = run_json(&["mcp", "--call", "cairn_drain_plan", &request])?;
    let receipt = &value["structuredContent"];
    assert_eq!(receipt["ready_change_count"], 1);
    let change = &receipt["changes"][0];
    assert_eq!(change["drain_ready"], true);
    let actions = change["next_actions"].as_array().ok_or("next_actions")?;
    assert!(actions
        .iter()
        .any(|action| action["tool"] == "cairn_sync_execute"
            && action["requires_allow_mutations"] == true));
    assert!(actions
        .iter()
        .any(|action| action["tool"] == "cairn_archive_execute"
            && action["requires_allow_mutations"] == true));
    fs::remove_dir_all(&temp_root)?;
    Ok(())
}

#[test]
fn mcp_drain_plan_orders_active_dependencies_before_dependents() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.change_dependency_graph.drain_order]
    // r[verify cairn.change_dependency_graph.drain_order.topological]
    // r[verify cairn.change_dependency_graph.positive_negative_tests]
    let temp_root = env::temp_dir().join(format!(
        "cairn-mcp-drain-dependencies-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&temp_root);
    copy_dir_recursive(&repo_root().join("fixtures/archive-ready"), &temp_root)?;
    let ready_change = temp_root.join(".cairn/changes/ready");
    let prereq_change = temp_root.join(".cairn/changes/z-prereq");
    let dependent_change = temp_root.join(".cairn/changes/a-dependent");
    copy_dir_recursive(&ready_change, &prereq_change)?;
    copy_dir_recursive(&ready_change, &dependent_change)?;
    fs::write(
        dependent_change.join("metadata.json"),
        "{\"depends_on\":[\"z-prereq\"]}\n",
    )?;
    fs::remove_dir_all(&ready_change)?;

    let request = format!(r#"{{"root":"{}"}}"#, temp_root.display());
    let value = run_json(&["mcp", "--call", "cairn_drain_plan", &request])?;
    let receipt = &value["structuredContent"];
    assert_eq!(
        receipt["drain_order"],
        serde_json::json!(["z-prereq", "a-dependent"])
    );
    assert_eq!(receipt["ready_change_count"], 1);
    let changes = receipt["changes"].as_array().ok_or("changes")?;
    let dependent = changes
        .iter()
        .find(|change| change["name"] == "a-dependent")
        .ok_or("dependent change")?;
    assert_eq!(dependent["blocked_by"], serde_json::json!(["z-prereq"]));
    assert_eq!(dependent["dependency_ready"], false);
    assert!(dependent["blockers"]
        .as_array()
        .ok_or("blockers")?
        .iter()
        .any(|blocker| blocker["class"] == "dependency_blocked"));
    fs::remove_dir_all(&temp_root)?;
    Ok(())
}
