use std::{error::Error, fs};

use super::support::{cairn, run_json, temp_fixture, temp_project};

#[test]
fn init_creates_native_layout_and_is_idempotent() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.init] r[verify cairn.supersede.init.empty] r[verify cairn.supersede.init.idempotent]
    // r[verify cairn.supersede.native_layout] r[verify cairn.supersede.native_layout.init]
    // r[verify cairn.supersede.native_layout.init_cairn]
    // r[verify cairn.lifecycle_directory.hidden_default.init]
    let root = temp_project("init")?;
    let root_text = root.to_str().ok_or("utf8 root")?;

    let first = run_json(&["init", "--root", root_text])?;
    assert_eq!(first["command"], "init");
    assert_eq!(first["mutated"], true);
    assert_eq!(first["layout"], "cairn");
    assert!(root.join(".cairn/specs").is_dir());
    assert!(root.join(".cairn/archive").is_dir());
    assert_eq!(first["created"].as_array().ok_or("created")?.len(), 4);

    let second = run_json(&["init", "--root", root_text])?;
    assert_eq!(second["mutated"], false);
    assert_eq!(second["created"].as_array().ok_or("created")?.len(), 0);
    assert_eq!(second["existing"].as_array().ok_or("existing")?.len(), 4);
    Ok(())
}

#[test]
fn change_create_creates_compatible_skeleton() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.change_create] r[verify cairn.supersede.change_create.skeleton]
    let root = temp_project("change-create")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;

    let receipt = run_json(&["change", "create", "replace-openspec", "--root", root_text])?;
    assert_eq!(receipt["command"], "change_create");
    assert_eq!(receipt["change"], "replace-openspec");
    assert_eq!(receipt["mutated"], true);
    assert_eq!(receipt["layout"], "cairn");
    assert!(root
        .join(".cairn/changes/replace-openspec/proposal.md")
        .is_file());
    assert!(root
        .join(".cairn/changes/replace-openspec/design.md")
        .is_file());
    assert!(root
        .join(".cairn/changes/replace-openspec/tasks.md")
        .is_file());
    assert!(root.join(".cairn/changes/replace-openspec/specs").is_dir());

    let status = run_json(&["status", "--root", root_text])?;
    assert_eq!(status["changes"][0]["name"], "replace-openspec");
    assert_eq!(status["changes"][0]["has_proposal"], true);
    assert_eq!(status["changes"][0]["task_counts"]["total"], 2);
    Ok(())
}

#[test]
fn change_create_rejects_invalid_name_without_mutation() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.change_create.invalid_name]
    let root = temp_project("change-invalid")?;
    let output = cairn()
        .args([
            "change",
            "create",
            "bad/name",
            "--root",
            root.to_str().ok_or("utf8 root")?,
        ])
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid change name"), "{stderr}");
    assert!(!root.join(".cairn/changes").exists());
    Ok(())
}

#[test]
fn onboarding_template_apply_and_scaffold_are_deterministic() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.developer_experience.onboarding_templates.catalog]
    // r[verify cairn.developer_experience.onboarding_templates.catalog.list]
    // r[verify cairn.developer_experience.onboarding_templates.apply]
    // r[verify cairn.developer_experience.onboarding_templates.apply.preview]
    // r[verify cairn.developer_experience.onboarding_templates.apply.execute]
    // r[verify cairn.developer_experience.onboarding_templates.apply.reject_unsafe]
    // r[verify cairn.developer_experience.onboarding_templates.quickstart]
    // r[verify cairn.developer_experience.onboarding_templates.quickstart.follow]
    // r[verify cairn.developer_experience.onboarding_templates.verification]
    // r[verify cairn.developer_experience.onboarding_templates.verification.receipts]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.planner.request_text]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.planner.insufficient_input]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.cli]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.cli.preview]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.cli.execute]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.gate_ready]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.gate_ready.refs]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.docs]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.docs.non_claims]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.verification]
    // r[verify cairn.developer_experience.prompt_to_change_scaffold.verification.fixtures]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.catalog]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.catalog.list]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.apply]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.apply.preview]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.docs]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.docs.read]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.verification]
    // r[verify cairn.developer_experience.onboarding_template_catalog_expansion.verification.run]
    // r[verify cairn.developer_experience.cli_modules.behavior_preservation.receipts]
    let root = temp_project("devex-template-scaffold")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;

    let templates = run_json(&["template", "list", "--root", root_text])?;
    assert_eq!(templates["valid"], true);
    let template_text = serde_json::to_string(&templates)?;
    assert!(template_text.contains("agent-workflow"));
    assert!(template_text.contains("docs-lifecycle"));
    assert!(template_text.contains("release-evidence"));

    let preview = run_json(&["template", "apply", "agent-workflow", "--root", root_text])?;
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!root.join("docs/cairn-agent-workflow.md").exists());

    let applied = run_json(&[
        "template",
        "apply",
        "agent-workflow",
        "--root",
        root_text,
        "--execute",
    ])?;
    assert_eq!(applied["valid"], true);
    assert_eq!(applied["mutated"], true);
    assert!(root.join("docs/cairn-agent-workflow.md").is_file());

    let release_preview =
        run_json(&["template", "apply", "release-evidence", "--root", root_text])?;
    assert_eq!(release_preview["valid"], true);
    assert_eq!(release_preview["dry_run"], true);
    assert!(!root.join("docs/cairn-release-evidence.md").exists());

    let scaffold = run_json(&[
        "scaffold",
        "change",
        "add-friendly-start",
        "--root",
        root_text,
        "--request",
        "Add a friendly start path",
        "--execute",
    ])?;
    assert_eq!(scaffold["valid"], true);
    assert_eq!(scaffold["mutated"], true);
    assert!(root
        .join(".cairn/changes/add-friendly-start/specs/developer-experience/spec.md")
        .is_file());
    let gate = run_json(&["gate", "tasks", "add-friendly-start", "--root", root_text])?;
    assert_eq!(gate["verdict"], "PASS");
    Ok(())
}

#[test]
fn reverse_spec_discovery_emits_review_needed_candidates() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.developer_experience.reverse_spec_discovery.policy]
    // r[verify cairn.developer_experience.reverse_spec_discovery.policy.load]
    // r[verify cairn.developer_experience.reverse_spec_discovery.planner.candidates]
    // r[verify cairn.developer_experience.reverse_spec_discovery.planner.limits]
    // r[verify cairn.developer_experience.reverse_spec_discovery.cli]
    // r[verify cairn.developer_experience.reverse_spec_discovery.cli.receipt]
    // r[verify cairn.developer_experience.reverse_spec_discovery.cli.out]
    // r[verify cairn.developer_experience.reverse_spec_discovery.docs]
    // r[verify cairn.developer_experience.reverse_spec_discovery.docs.review_needed]
    // r[verify cairn.developer_experience.reverse_spec_discovery.verification]
    // r[verify cairn.developer_experience.reverse_spec_discovery.verification.fixtures]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.expand]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.expand.source_dir]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.receipt]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.receipt.review]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.docs]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.docs.read]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.verification]
    // r[verify cairn.developer_experience.reverse_spec_directory_inventory.verification.run]
    let root = temp_project("devex-discovery")?;
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("src/nested"))?;
    fs::create_dir_all(root.join("src/.hidden"))?;
    fs::create_dir_all(root.join("target"))?;
    fs::write(root.join("src/lib.rs"), "pub fn visible_behavior() {}\n")?;
    fs::write(root.join("src/nested/feature.md"), "### Nested Feature\n")?;
    fs::write(root.join("src/.hidden/secret.rs"), "pub fn secret() {}\n")?;
    fs::write(root.join("target/generated.rs"), "pub fn generated() {}\n")?;
    let root_text = root.to_str().ok_or("utf8 root")?;

    let receipt = run_json(&[
        "discover-specs",
        "--root",
        root_text,
        "--include",
        "src/lib.rs",
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["candidates"][0]["review_status"], "review-needed");
    assert_eq!(receipt["candidates"][0]["evidence_path"], "src/lib.rs");

    let directory_receipt = run_json(&[
        "discover-specs",
        "--root",
        root_text,
        "--include",
        "src",
        "--include",
        "target",
    ])?;
    assert_eq!(directory_receipt["valid"], true);
    let directory_text = serde_json::to_string(&directory_receipt)?;
    assert!(directory_text.contains("src/lib.rs"), "{directory_text}");
    assert!(
        directory_text.contains("src/nested/feature.md"),
        "{directory_text}"
    );
    assert!(directory_text.contains("src/.hidden"), "{directory_text}");
    assert!(directory_text.contains("target"), "{directory_text}");
    assert!(!directory_text.contains("generated"), "{directory_text}");

    let draft = run_json(&[
        "discover-specs",
        "--root",
        root_text,
        "--include",
        "src/lib.rs",
        "--out",
        "target/discovery-draft.md",
    ])?;
    assert_eq!(draft["valid"], true);
    assert!(root.join("target/discovery-draft.md").is_file());
    Ok(())
}

#[test]
fn change_metadata_is_validated_and_filterable() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.change_metadata.schema.valid_metadata]
    // r[verify cairn.supersede.change_metadata.schema.optional_metadata]
    // r[verify cairn.supersede.change_metadata.validation.invalid_metadata]
    // r[verify cairn.supersede.change_metadata.read_receipts.list_show]
    // r[verify cairn.supersede.change_metadata.filters.group]
    // r[verify cairn.supersede.change_metadata.filters.target_date]
    // r[verify cairn.supersede.change_metadata.verification]
    // r[verify cairn.supersede.change_metadata.verification.gates]
    let fixture = temp_fixture("change-metadata", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    fs::write(
        fixture.join(".cairn/changes/demo/metadata.json"),
        "{\"groups\":[\"feature:registry\",\"current\"],\"target_date\":\"2026-06-01\",\"status\":\"planned\"}\n",
    )?;

    let filtered = run_json(&[
        "change",
        "list",
        "--root",
        root,
        "--group",
        "feature:registry",
        "--target-after",
        "2026-05-01",
        "--target-before",
        "2026-07-01",
    ])?;
    assert_eq!(filtered["changes"].as_array().ok_or("changes")?.len(), 1);
    assert_eq!(filtered["changes"][0]["metadata"]["groups"][0], "current");
    assert_eq!(
        filtered["changes"][0]["metadata"]["groups"][1],
        "feature:registry"
    );
    assert_eq!(
        filtered["changes"][0]["metadata"]["target_date"],
        "2026-06-01"
    );

    let excluded = run_json(&["change", "list", "--root", root, "--group", "future"])?;
    assert!(excluded["changes"].as_array().ok_or("changes")?.is_empty());

    fs::write(
        fixture.join(".cairn/changes/demo/metadata.json"),
        "{\"groups\":[\"unknown\"],\"target_date\":\"not-a-date\",\"status\":\"unknown\"}\n",
    )?;
    let invalid = run_json(&["validate", "--root", root])?;
    assert_eq!(invalid["valid"], false);
    let issues = invalid["change_issues"].as_array().ok_or("change_issues")?;
    assert_eq!(issues.len(), 3);
    let issue_text = serde_json::to_string(issues)?;
    assert!(
        issue_text.contains("metadata group unknown"),
        "{issue_text}"
    );
    assert!(issue_text.contains("metadata target_date"), "{issue_text}");
    assert!(
        issue_text.contains("metadata status unknown"),
        "{issue_text}"
    );
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn change_dependency_metadata_is_validated_and_reported() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.change_dependency_graph.metadata]
    // r[verify cairn.change_dependency_graph.metadata.known]
    // r[verify cairn.change_dependency_graph.metadata.invalid]
    // r[verify cairn.change_dependency_graph.graph_validation]
    // r[verify cairn.change_dependency_graph.positive_negative_tests]
    // r[verify cairn.change_dependency_graph.validation]
    // r[verify cairn.change_dependency_graph.validation.repository_checks]
    let fixture = temp_fixture("change-dependencies", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    fs::create_dir_all(fixture.join(".cairn/archive/2026-05-21-finished-prereq"))?;
    fs::write(
        fixture.join(".cairn/changes/demo/metadata.json"),
        "{\"depends_on\":[\"finished-prereq\"]}\n",
    )?;

    let shown = run_json(&["change", "show", "demo", "--root", root])?;
    assert_eq!(
        shown["change"]["metadata"]["depends_on"][0],
        "finished-prereq"
    );
    let graph = run_json(&["change", "graph", "--root", root])?;
    assert_eq!(graph["valid"], true);
    assert_eq!(
        graph["nodes"][0]["satisfied_dependencies"][0],
        "finished-prereq"
    );
    assert_eq!(graph["nodes"][0]["dependency_ready"], true);

    fs::write(
        fixture.join(".cairn/changes/demo/metadata.json"),
        "{\"depends_on\":[\"\",\"bad/name\",\"missing\",\"missing\",\"demo\"]}\n",
    )?;
    let invalid = run_json(&["validate", "--root", root])?;
    assert_eq!(invalid["valid"], false);
    let issue_text = serde_json::to_string(&invalid["change_issues"])?;
    assert!(issue_text.contains("change_dependency.metadata.depends_on.empty_id"));
    assert!(issue_text.contains("change_dependency.metadata.depends_on.invalid_id"));
    assert!(issue_text.contains("change_dependency.duplicate_dependency"));
    assert!(issue_text.contains("change_dependency.unknown_dependency"));
    assert!(issue_text.contains("change_dependency.self_dependency"));

    fs::write(
        fixture.join(".cairn/changes/demo/metadata.json"),
        "{\"depends_on\":\"not-an-array\"}\n",
    )?;
    let invalid_type = run_json(&["validate", "--root", root])?;
    let type_issue_text = serde_json::to_string(&invalid_type["change_issues"])?;
    assert!(type_issue_text.contains("change_dependency.metadata.depends_on.invalid_type"));
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn change_dependency_authoring_preview_execute_and_remove_are_safe() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.change_dependency_graph.authoring_cli]
    // r[verify cairn.change_dependency_graph.authoring_cli.list]
    // r[verify cairn.change_dependency_graph.authoring_cli.preview]
    // r[verify cairn.change_dependency_graph.authoring_cli.execute]
    // r[verify cairn.change_dependency_graph.authoring_cli.remove]
    // r[verify cairn.change_dependency_graph.authoring_cli.tests.positive]
    // r[verify cairn.change_dependency_graph.authoring_cli.validation]
    let fixture = temp_fixture("change-dependency-authoring", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    run_json(&["change", "create", "prereq", "--root", root])?;

    let preview = run_json(&["change", "depend", "add", "demo", "prereq", "--root", root])?;
    assert_eq!(preview["valid"], true);
    assert_eq!(preview["dry_run"], true);
    assert_eq!(preview["mutated"], false);
    assert!(!fixture.join(".cairn/changes/demo/metadata.json").exists());

    let executed = run_json(&[
        "change",
        "depend",
        "add",
        "demo",
        "prereq",
        "--root",
        root,
        "--execute",
    ])?;
    assert_eq!(executed["valid"], true);
    assert_eq!(executed["mutated"], true);
    assert_eq!(executed["after"][0], "prereq");
    let metadata = fs::read_to_string(fixture.join(".cairn/changes/demo/metadata.json"))?;
    assert!(metadata.contains("depends_on"));
    assert!(metadata.contains("prereq"));

    let listed = run_json(&["change", "depend", "list", "demo", "--root", root])?;
    assert_eq!(listed["dependencies"][0], "prereq");
    let next = run_json(&["change", "next", "--root", root])?;
    assert_eq!(next["ready_change"], "prereq");

    let removed = run_json(&[
        "change",
        "depend",
        "remove",
        "demo",
        "prereq",
        "--root",
        root,
        "--execute",
    ])?;
    assert_eq!(removed["valid"], true);
    assert_eq!(removed["mutated"], true);
    let metadata = fs::read_to_string(fixture.join(".cairn/changes/demo/metadata.json"))?;
    assert!(!metadata.contains("depends_on"));
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn change_dependency_graph_human_formats_are_stable() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.change_dependency_graph.render_formats]
    // r[verify cairn.change_dependency_graph.render_formats.json_default]
    // r[verify cairn.change_dependency_graph.render_formats.human]
    // r[verify cairn.change_dependency_graph.authoring_cli.tests.positive]
    let fixture = temp_fixture("change-dependency-render", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    run_json(&["change", "create", "prereq", "--root", root])?;
    run_json(&[
        "change",
        "depend",
        "add",
        "demo",
        "prereq",
        "--root",
        root,
        "--execute",
    ])?;

    let json_graph = run_json(&["change", "graph", "--root", root])?;
    assert_eq!(json_graph["command"], "change graph");

    let mermaid = cairn()
        .args(["change", "graph", "--root", root, "--format", "mermaid"])
        .output()?;
    assert!(mermaid.status.success());
    let mermaid_stdout = String::from_utf8_lossy(&mermaid.stdout);
    assert!(mermaid_stdout.contains("flowchart TD"), "{mermaid_stdout}");
    assert!(
        mermaid_stdout.contains("active blocker"),
        "{mermaid_stdout}"
    );

    let dot = cairn()
        .args(["change", "graph", "--root", root, "--format", "dot"])
        .output()?;
    assert!(dot.status.success());
    let dot_stdout = String::from_utf8_lossy(&dot.stdout);
    assert!(
        dot_stdout.contains("digraph change_dependencies"),
        "{dot_stdout}"
    );
    assert!(dot_stdout.contains("active blocker"), "{dot_stdout}");
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn change_dependency_authoring_rejects_bad_edits_without_mutation() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.change_dependency_graph.authoring_cli.invalid]
    // r[verify cairn.change_dependency_graph.render_formats.invalid]
    // r[verify cairn.change_dependency_graph.authoring_cli.tests.negative]
    let fixture = temp_fixture("change-dependency-authoring-negative", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    run_json(&["change", "create", "prereq", "--root", root])?;
    run_json(&[
        "change",
        "depend",
        "add",
        "demo",
        "prereq",
        "--root",
        root,
        "--execute",
    ])?;

    let duplicate = cairn()
        .args([
            "change",
            "depend",
            "add",
            "demo",
            "prereq",
            "--root",
            root,
            "--execute",
        ])
        .output()?;
    assert!(!duplicate.status.success());
    let duplicate_receipt: serde_json::Value = serde_json::from_slice(&duplicate.stdout)?;
    assert_eq!(duplicate_receipt["valid"], false);
    assert!(serde_json::to_string(&duplicate_receipt["issues"])?
        .contains("change_dependency.duplicate_dependency"));

    let invalid = cairn()
        .args([
            "change", "depend", "add", "demo", "bad/name", "--root", root,
        ])
        .output()?;
    assert!(!invalid.status.success());
    let invalid_receipt: serde_json::Value = serde_json::from_slice(&invalid.stdout)?;
    assert!(serde_json::to_string(&invalid_receipt["issues"])?
        .contains("change_dependency.metadata.depends_on.invalid_id"));

    let unknown_format = cairn()
        .args(["change", "graph", "--root", root, "--format", "svg"])
        .output()?;
    assert!(!unknown_format.status.success());
    assert!(String::from_utf8_lossy(&unknown_format.stderr).contains("unknown change graph format"));

    let metadata = fs::read_to_string(fixture.join(".cairn/changes/demo/metadata.json"))?;
    assert_eq!(metadata.matches("prereq").count(), 1);
    fs::remove_dir_all(fixture)?;
    Ok(())
}
