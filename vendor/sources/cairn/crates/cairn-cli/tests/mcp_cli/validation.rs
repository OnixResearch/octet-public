use std::{env, error::Error, fs};

use super::common::{cairn, copy_dir_recursive, repo_root, run_json};

#[test]
fn mcp_rejects_invalid_arguments_before_dispatch() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.mcp_server.strict_schemas.validation]
    // r[verify cairn.supersede.mcp_server.strict_schemas.errors]
    // r[verify cairn.supersede.mcp_server.strict_schemas.verification]
    let missing = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_read_spec",
            r#"{"root":"fixtures/basic"}"#,
        ])
        .output()?;
    assert!(!missing.status.success());
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        stderr.contains("invalid arguments for cairn_read_spec: missing required argument spec"),
        "{stderr}"
    );

    let unknown = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_status",
            r#"{"root":"fixtures/basic","surprise":"no"}"#,
        ])
        .output()?;
    assert!(!unknown.status.success());
    let stderr = String::from_utf8_lossy(&unknown.stderr);
    assert!(
        stderr.contains("invalid arguments for cairn_status: unknown argument surprise"),
        "{stderr}"
    );

    let bad_stage = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_gate",
            r#"{"root":"fixtures/basic","change":"demo","stage":"archive"}"#,
        ])
        .output()?;
    assert!(!bad_stage.status.success());
    let stderr = String::from_utf8_lossy(&bad_stage.stderr);
    assert!(
        stderr.contains("invalid arguments for cairn_gate: argument stage must be one of proposal, design, tasks"),
        "{stderr}"
    );

    let bad_source = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_migrate_plan",
            r#"{"root":"fixtures/basic","source":"native"}"#,
        ])
        .output()?;
    assert!(!bad_source.status.success());
    let stderr = String::from_utf8_lossy(&bad_source.stderr);
    assert!(
        stderr.contains("invalid arguments for cairn_migrate_plan: argument source must be one of cairn, openspec, ironspec"),
        "{stderr}"
    );

    let missing_plan_hash = cairn()
        .args([
            "mcp",
            "--allow-mutations",
            "--call",
            "cairn_migrate_execute",
            r#"{"root":"fixtures/basic","source":"openspec"}"#,
        ])
        .output()?;
    assert!(!missing_plan_hash.status.success());
    let stderr = String::from_utf8_lossy(&missing_plan_hash.stderr);
    assert!(
        stderr.contains(
            "invalid arguments for cairn_migrate_execute: missing required argument plan_hash"
        ),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn mcp_vacuous_validation_and_mutation_share_fail_closed_diagnostics() -> Result<(), Box<dyn Error>>
{
    // r[verify cairn.lifecycle_validation.mutation_block.consistent]
    // r[verify cairn.lifecycle_validation.tests.negative]
    let fixture = env::temp_dir().join(format!("cairn-mcp-vacuous-{}", std::process::id()));
    if fixture.exists() {
        fs::remove_dir_all(&fixture)?;
    }
    copy_dir_recursive(&repo_root().join("fixtures/archive-ready"), &fixture)?;
    let change = fixture.join(".cairn/changes/ready");
    fs::write(change.join("proposal.md"), "\n")?;
    fs::write(change.join("design.md"), "CAIRN_PLACEHOLDER\n")?;
    fs::write(
        change.join("tasks.md"),
        "- [x] [depends:] CAIRN_PLACEHOLDER\n",
    )?;
    fs::write(change.join("specs/demo/spec.md"), "\n")?;
    let accepted_path = fixture.join(".cairn/specs/demo/spec.md");
    let accepted_before = fs::read_to_string(&accepted_path)?;
    let root = fixture.display().to_string();

    let validate_input = serde_json::json!({"root": &root}).to_string();
    let validate = run_json(&["mcp", "--call", "cairn_validate", &validate_input])?;
    let validation_receipt = &validate["structuredContent"];
    assert_eq!(validation_receipt["valid"], false);
    assert!(validation_receipt["substance_findings"]
        .as_array()
        .ok_or("substance findings")?
        .iter()
        .any(|finding| finding["code"]
            .as_str()
            .unwrap_or_default()
            .starts_with("lifecycle_validation.")));
    assert!(validation_receipt["substance_issues"]
        .as_array()
        .ok_or("substance issues")?
        .iter()
        .any(|issue| issue
            .as_str()
            .unwrap_or_default()
            .contains("lifecycle_validation")));

    let mutation_input = serde_json::json!({"change": "ready", "root": &root}).to_string();
    let mutation = run_json(&[
        "mcp",
        "--allow-mutations",
        "--call",
        "cairn_sync_execute",
        &mutation_input,
    ])?;
    let mutation_receipt = &mutation["structuredContent"];
    assert_eq!(mutation_receipt["blocked"], true);
    assert_eq!(mutation_receipt["mutated"], false);
    assert!(mutation_receipt["reasons"]
        .as_array()
        .ok_or("mutation reasons")?
        .iter()
        .any(|reason| reason
            .as_str()
            .unwrap_or_default()
            .contains("lifecycle_validation")));
    assert_eq!(fs::read_to_string(accepted_path)?, accepted_before);
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn mcp_rejects_traversal_and_default_mutation() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.mcp_server.boundary.shell]
    // r[verify cairn.supersede.mcp_server.mutation_guard.default_deny]
    // r[verify cairn.supersede.mcp_server.verification.negative]
    let traversal = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_read_spec",
            r#"{"spec":"../secret","root":"fixtures/basic"}"#,
        ])
        .output()?;
    assert!(!traversal.status.success());
    let stderr = String::from_utf8_lossy(&traversal.stderr);
    assert!(
        stderr.contains("selector escapes lifecycle bounds"),
        "{stderr}"
    );

    let denied = run_json(&[
        "mcp",
        "--call",
        "cairn_sync_execute",
        r#"{"change":"demo","root":"fixtures/basic"}"#,
    ])?;
    let receipt = &denied["structuredContent"];
    assert_eq!(receipt["allowed"], false);
    assert_eq!(receipt["mutated"], false);
    Ok(())
}
