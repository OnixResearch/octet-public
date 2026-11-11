use std::{
    error::Error,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use serde_json::Value;

const PROJECT_FACT_PRIORITY: u32 = 10;
const ARTIFACT_GUIDANCE_PRIORITY: u32 = 20;
const OPERATION_GUIDANCE_PRIORITY: u32 = 30;
const EXPECTED_ENTRY_COUNT: usize = 4;
const MAX_SOURCE_TEXT_BYTES: usize = 262_144;
const FIRST_ADVISORY_ENTRY_INDEX: usize = 2;

const PROJECT_NON_CLAIMS: &[&str] = &[
    "a project pointer selects one declared store but does not grant lifecycle authority",
    "a direct reference provides read-only discovery and does not satisfy dependencies, gates, evidence, or requirements",
    "reference indexing does not prove truth, freshness, ownership, implementation, or satisfaction",
    "project configuration does not authorize transitive reference traversal or transport",
    "project facts and advisory guidance do not override generated policy or command authorization",
    "project context checks do not prove truth, secrecy, semantic consistency, agent compliance, or release eligibility",
];

fn test_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!(
        "cairn-agent-guidance-{name}-{}",
        std::process::id()
    ));
    match fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    fs::create_dir_all(root.join(".cairn/specs"))?;
    fs::create_dir_all(root.join(".cairn/changes"))?;
    fs::create_dir_all(root.join(".cairn/archive"))?;
    fs::create_dir_all(root.join("docs"))?;
    Ok(root)
}

fn project_source(context_path: &str) -> String {
    let non_claims = PROJECT_NON_CLAIMS
        .iter()
        .map(|claim| format!("    \"{claim}\","))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"{{
  schema = "cairn.lifecycle-store-project-source.v1",
  lifecycle_store = null,
  references = [],
  context_refs = [{{
    id = "project-overview",
    source_root = "project",
    path = "{context_path}",
    required = true,
    priority = {PROJECT_FACT_PRIORITY},
    authority = "project_fact",
    profiles = [],
    artifacts = [],
    operations = ["context", "workflow-instructions"],
  }}, {{
    id = "lifecycle-overview",
    source_root = "lifecycle",
    path = "docs/lifecycle.md",
    required = true,
    priority = {PROJECT_FACT_PRIORITY},
    authority = "project_fact",
    profiles = [],
    artifacts = [],
    operations = ["context", "workflow-instructions"],
  }}],
  artifact_guidance = [{{
    id = "task-advice",
    text = "Keep task evidence local and bounded.",
    priority = {ARTIFACT_GUIDANCE_PRIORITY},
    authority = "advisory_guidance",
    profiles = [],
    artifacts = ["tasks"],
  }}],
  operation_guidance = [{{
    id = "context-advice",
    text = "Treat project guidance as advisory input.",
    priority = {OPERATION_GUIDANCE_PRIORITY},
    authority = "advisory_guidance",
    operations = ["context", "workflow-instructions"],
  }}],
  non_claims = [
{non_claims}
  ],
}}
"#
    )
}

fn copy_tree(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn cairn(root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(root)
        .env_remove("CAIRN_GLOBAL_REGISTRY");
    command
}

fn export_project(root: &Path) -> Result<(), Box<dyn Error>> {
    let output = cairn(root)
        .args(["config", "export", "--kind", "project", "--root"])
        .arg(root)
        .arg("--execute")
        .output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

fn context_output(root: &Path, profile: &str, artifact: &str) -> Result<Output, Box<dyn Error>> {
    Ok(cairn(root)
        .args(["context", "--root"])
        .arg(root)
        .args(["--profile", profile, "--artifact", artifact])
        .output()?)
}

fn parse_success(output: Output) -> Result<Value, Box<dyn Error>> {
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn mcp_call(root: &Path, name: &str, arguments: Value) -> Result<Value, Box<dyn Error>> {
    let mut child = cairn(root)
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments}
    });
    writeln!(child.stdin.as_mut().ok_or("MCP stdin")?, "{request}")?;
    let output = child.wait_with_output()?;
    assert!(output.status.success());
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn mcp_context(root: &Path) -> Result<Value, Box<dyn Error>> {
    let response = mcp_call(
        root,
        "cairn_context",
        serde_json::json!({
            "root": root,
            "profile": "assurance",
            "artifact": "tasks"
        }),
    )?;
    assert!(
        response.get("error").is_none(),
        "MCP context failed: {response}"
    );
    Ok(response["result"]["structuredContent"].clone())
}

#[test]
fn guidance_guide_keeps_authority_and_privacy_claims_bounded() {
    // r[verify cairn.agent_guidance_context.docs]
    // r[verify cairn.agent_guidance_context.docs.authority]
    let guide = include_str!("../../../docs/agent-guidance-context.md");
    assert!(guide.contains("policy_constraint"));
    assert!(guide.contains("project_fact"));
    assert!(guide.contains("advisory_guidance"));
    assert!(guide.contains("do not prove that text is secret-free"));
    assert!(guide.contains("Validation, gate, sync, archive, evidence, and release commands"));
}

// r[verify cairn.agent_guidance_context.project_config.valid]
// r[verify cairn.agent_guidance_context.sources.valid]
// r[verify cairn.agent_guidance_context.selection.artifact]
// r[verify cairn.agent_guidance_context.mcp.parity]
// r[verify cairn.agent_guidance_context.privacy.secret]
// r[verify cairn.agent_guidance_context.privacy.undeclared]
// r[verify cairn.agent_guidance_context.verification]
#[test]
fn context_and_mcp_select_bounded_guidance_with_private_portable_evidence(
) -> Result<(), Box<dyn Error>> {
    let root = test_root("positive")?;
    fs::write(
        root.join("docs/context.md"),
        "# Project context\ncredential=secret-like-value\n",
    )?;
    fs::write(root.join("docs/lifecycle.md"), "# Lifecycle context\n")?;
    fs::write(
        root.join("docs/undeclared-secret.md"),
        "undeclared-secret-value",
    )?;
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("docs/context.md"),
    )?;
    export_project(&root)?;

    let context = parse_success(context_output(&root, "assurance", "tasks")?)?;
    assert_eq!(context["guidance"]["valid"], true);
    assert_eq!(
        context["guidance"]["entries"]
            .as_array()
            .ok_or("entries")?
            .len(),
        EXPECTED_ENTRY_COUNT
    );
    assert_eq!(
        context["guidance"]["entries"][0]["authority"],
        "project_fact"
    );
    assert_eq!(
        context["guidance"]["entries"][FIRST_ADVISORY_ENTRY_INDEX]["authority"],
        "advisory_guidance"
    );
    assert!(context["guidance"]["entries"][0]["text"]
        .as_str()
        .ok_or("local context text")?
        .contains("Lifecycle context"));
    assert!(context["guidance"]["entries"][1]["text"]
        .as_str()
        .ok_or("local project text")?
        .contains("Project context"));
    assert!(!context.to_string().contains("undeclared-secret-value"));
    assert!(context["guidance"]["portable_entries"][0]
        .get("text")
        .is_none());
    assert_eq!(context["guidance"]["portable_omits_raw_text"], true);
    assert!(!context["guidance"]["portable_evidence"]
        .to_string()
        .contains("secret-like-value"));

    let mcp = mcp_context(&root)?;
    assert_eq!(
        mcp["guidance"]["portable_identity"],
        context["guidance"]["portable_identity"]
    );
    assert_eq!(
        mcp["guidance"]["local_result_hash"],
        context["guidance"]["local_result_hash"]
    );
    // r[verify cairn.agent_guidance_context.mcp.invalid]
    let invalid = mcp_call(
        &root,
        "cairn_context",
        serde_json::json!({"root": root, "ambient_prompt": "ignored"}),
    )?;
    assert!(invalid["error"]["message"]
        .as_str()
        .ok_or("MCP argument error")?
        .contains("invalid arguments for cairn_context"));
    fs::remove_dir_all(root)?;
    Ok(())
}

// r[verify cairn.agent_guidance_context.integration.instructions]
// r[verify cairn.agent_guidance_context.authority.advisory]
#[test]
fn workflow_instructions_include_policy_constraints_and_selected_guidance(
) -> Result<(), Box<dyn Error>> {
    let root = test_root("workflow-instructions")?;
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    copy_tree(&repository_root.join("fixtures/basic"), &root)?;
    fs::write(root.join("docs/context.md"), "# Workflow context\n")?;
    fs::write(
        root.join("docs/lifecycle.md"),
        "# Lifecycle workflow context\n",
    )?;
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("docs/context.md"),
    )?;
    export_project(&root)?;

    let output = cairn(&root)
        .args([
            "workflow",
            "instructions",
            "tasks",
            "--change",
            "demo",
            "--root",
        ])
        .arg(&root)
        .arg("--policy")
        .arg(repository_root.join("cairn-policy/generated/cairn-policy.json"))
        .output()?;
    let receipt = parse_success(output)?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["artifact"]["artifact_id"], "tasks");
    assert_eq!(
        receipt["guidance"]["entries"][0]["authority"],
        "policy_constraint"
    );
    assert!(receipt["guidance"]["entries"]
        .as_array()
        .ok_or("workflow guidance entries")?
        .iter()
        .any(|entry| entry["id"] == "task-advice"));
    assert!(receipt["guidance"]["portable_entries"]
        .as_array()
        .ok_or("portable workflow guidance")?
        .iter()
        .all(|entry| entry.get("text").is_none()));
    assert!(receipt["guidance"]["portable_identity"].is_string());

    // r[verify cairn.agent_guidance_context.integration.non_authority]
    let gate = cairn(&root)
        .args(["gate", "tasks", "demo", "--root"])
        .arg(&root)
        .arg("--policy")
        .arg(repository_root.join("cairn-policy/generated/cairn-policy.json"))
        .output()?;
    let gate_receipt = parse_success(gate)?;
    assert!(gate_receipt.get("guidance").is_none());
    fs::remove_dir_all(root)?;
    Ok(())
}

// r[verify cairn.agent_guidance_context.project_config.invalid]
// r[verify cairn.agent_guidance_context.selection.unknown]
#[test]
fn context_rejects_stale_project_unknown_selectors_and_missing_required_sources(
) -> Result<(), Box<dyn Error>> {
    let root = test_root("negative")?;
    fs::write(root.join("docs/context.md"), "# Project context\n")?;
    fs::write(root.join("docs/lifecycle.md"), "# Lifecycle context\n")?;
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("docs/context.md"),
    )?;
    export_project(&root)?;

    fs::write(root.join(".cairn/project.ncl"), "# stale\n")?;
    let stale = context_output(&root, "assurance", "tasks")?;
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("project guidance projection is stale"));

    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("docs/context.md"),
    )?;
    export_project(&root)?;
    let unknown = context_output(&root, "unknown-profile", "tasks")?;
    assert!(!unknown.status.success());
    assert!(
        String::from_utf8_lossy(&unknown.stderr).contains("agent_guidance.selector.unknown"),
        "unexpected unknown-selector diagnostic: {}",
        String::from_utf8_lossy(&unknown.stderr)
    );

    fs::remove_file(root.join("docs/context.md"))?;
    let missing = context_output(&root, "assurance", "tasks")?;
    assert!(!missing.status.success());
    let receipt: Value = serde_json::from_slice(&missing.stdout)?;
    assert_eq!(receipt["guidance"]["valid"], false);
    assert_eq!(
        receipt["guidance"]["issues"][0]["code"],
        "agent_guidance.source.missing"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn empty_guidance_keeps_existing_context_and_workflow_receipts() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.agent_guidance_context.compatibility.empty]
    let root = test_root("compatibility-empty")?;
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    copy_tree(&repository_root.join("fixtures/basic"), &root)?;

    let context = parse_success(
        cairn(&root)
            .args(["context", "--root"])
            .arg(&root)
            .output()?,
    )?;
    assert!(context.get("guidance").is_none());

    let instructions = parse_success(
        cairn(&root)
            .args([
                "workflow",
                "instructions",
                "tasks",
                "--change",
                "demo",
                "--root",
            ])
            .arg(&root)
            .arg("--policy")
            .arg(repository_root.join("cairn-policy/generated/cairn-policy.json"))
            .output()?,
    )?;
    assert!(instructions.get("guidance").is_none());
    assert_eq!(instructions["artifact"]["artifact_id"], "tasks");
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn portable_identity_is_checkout_independent() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.agent_guidance_context.receipts.portable]
    let first = test_root("portable-first")?;
    let second = test_root("portable-second")?;
    for root in [&first, &second] {
        fs::write(root.join("docs/context.md"), "# Same project context\n")?;
        fs::write(root.join("docs/lifecycle.md"), "# Same lifecycle context\n")?;
        fs::create_dir_all(root.join(".cairn"))?;
        fs::write(
            root.join(".cairn/project.ncl"),
            project_source("docs/context.md"),
        )?;
        export_project(root)?;
    }
    let first_receipt = parse_success(context_output(&first, "assurance", "tasks")?)?;
    let second_receipt = parse_success(context_output(&second, "assurance", "tasks")?)?;
    assert_eq!(
        first_receipt["guidance"]["portable_identity"],
        second_receipt["guidance"]["portable_identity"]
    );
    assert_eq!(
        first_receipt["guidance"]["portable_evidence"],
        second_receipt["guidance"]["portable_evidence"]
    );
    fs::remove_dir_all(first)?;
    fs::remove_dir_all(second)?;
    Ok(())
}

#[test]
fn source_shell_rejects_symlinks_invalid_utf8_and_oversize_text() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.agent_guidance_context.sources.invalid]
    // r[verify cairn.agent_guidance_context.functional_boundary.shell]
    let root = test_root("source-rejections")?;
    fs::write(root.join("docs/context.md"), "# Project context\n")?;
    fs::write(root.join("docs/lifecycle.md"), "# Lifecycle context\n")?;
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("docs/context.md"),
    )?;
    export_project(&root)?;

    fs::write(root.join("docs/symlink-target.md"), "target")?;
    fs::remove_file(root.join("docs/context.md"))?;
    std::os::unix::fs::symlink("symlink-target.md", root.join("docs/context.md"))?;
    let symlink = context_output(&root, "assurance", "tasks")?;
    assert!(!symlink.status.success());
    let symlink_receipt: Value = serde_json::from_slice(&symlink.stdout)?;
    assert!(symlink_receipt["guidance"]["issues"]
        .as_array()
        .ok_or("symlink issues")?
        .iter()
        .any(|issue| issue["code"] == "agent_guidance.source.invalid"));

    fs::remove_file(root.join("docs/context.md"))?;
    fs::write(root.join("docs/context.md"), [u8::MAX])?;
    let invalid_utf8 = context_output(&root, "assurance", "tasks")?;
    assert!(!invalid_utf8.status.success());
    let invalid_receipt: Value = serde_json::from_slice(&invalid_utf8.stdout)?;
    assert_eq!(invalid_receipt["guidance"]["valid"], false);

    fs::write(
        root.join("docs/context.md"),
        vec![b'x'; MAX_SOURCE_TEXT_BYTES.saturating_add(1)],
    )?;
    let oversized = context_output(&root, "assurance", "tasks")?;
    assert!(!oversized.status.success());
    let oversized_receipt: Value = serde_json::from_slice(&oversized.stdout)?;
    assert_eq!(oversized_receipt["guidance"]["valid"], false);
    fs::remove_dir_all(root)?;
    Ok(())
}

// r[verify cairn.agent_guidance_context.project_config.invalid]
#[test]
fn project_export_rejects_ambient_and_escaping_context_paths() -> Result<(), Box<dyn Error>> {
    let root = test_root("path-rejection")?;
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(
        root.join(".cairn/project.ncl"),
        project_source("../ambient-agent-instructions.md"),
    )?;
    let output = cairn(&root)
        .args(["config", "export", "--kind", "project", "--root"])
        .arg(&root)
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("contract broken") || stderr.contains("agent_guidance.source.path_invalid"),
        "unexpected path diagnostic: {stderr}"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}
