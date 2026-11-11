use std::{error::Error, fs};

use super::common::{assert_hash_field, cairn, repo_root, run_json};

#[test]
fn mcp_agent_smoke_receipt_proves_operator_flow() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.mcp_agent_smoke_receipts.flow]
    // r[verify cairn.supersede.mcp_agent_smoke_receipts.evidence]
    // r[verify cairn.supersede.mcp_agent_smoke_receipts.read_only]
    // r[verify cairn.supersede.mcp_agent_smoke_receipts.docs]
    // r[verify cairn.supersede.mcp_agent_smoke_receipts.verification]
    let receipt = run_json(&["agent-receipt", "smoke-mcp", "--root", "."])?;
    assert_eq!(receipt["command"], "agent-receipt smoke-mcp");
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(receipt["fixture"], "fixtures/agent-completion");
    assert_eq!(receipt["mcp"]["read_only"], true);
    assert_eq!(receipt["mutation_denial"]["allowed"], false);
    assert_eq!(receipt["mutation_denial"]["mutated"], false);
    assert!(receipt["catalog_tool_count"].as_u64().unwrap_or_default() >= 9);
    assert!(receipt["invalid_argument_error"]
        .as_str()
        .unwrap_or_default()
        .contains("invalid arguments for cairn_gate"));
    assert_hash_field(&receipt, "receipt_hash")?;
    assert_hash_field(&receipt, "receipt_tool_schema_hash")?;
    assert_hash_field(&receipt, "agent_receipt_validation_hash")?;
    assert_hash_field(&receipt, "agent_receipt_hash")?;
    let checks = receipt["checks"].as_array().ok_or("checks")?;
    assert_eq!(checks.len(), 5);
    assert!(checks
        .iter()
        .all(|check| check["passed"].as_bool() == Some(true)));
    Ok(())
}

#[test]
fn mcp_agent_receipt_validation_is_read_only_and_bounded() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.agent_receipt_validation_surface.mcp]
    // r[verify cairn.supersede.agent_receipt_validation_surface.mcp.read_only]
    // r[verify cairn.supersede.agent_receipt_validation_surface.verification]
    let catalog = run_json(&["mcp", "--describe"])?;
    let tools = catalog["tools"].as_array().ok_or("tools")?;
    assert!(tools
        .iter()
        .any(|tool| tool["name"] == "cairn_agent_receipt_validate"
            && tool["annotations"]["readOnlyHint"] == true
            && tool["annotations"]["destructiveHint"] == false));

    let by_path = run_json(&[
        "mcp",
        "--call",
        "cairn_agent_receipt_validate",
        r#"{"root":".","path":"fixtures/agent-receipts/positive.json"}"#,
    ])?;
    let receipt = &by_path["structuredContent"];
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "INSPECTED");
    assert_eq!(receipt["classification"], "legacy_unscoped");
    assert_eq!(receipt["authoritative_completion"], false);
    assert_eq!(receipt["mcp"]["read_only"], true);
    assert_hash_field(receipt, "agent_receipt_hash")?;
    assert_hash_field(receipt, "receipt_hash")?;

    let inline_fixture =
        fs::read_to_string(repo_root().join("fixtures/agent-receipts/positive.json"))?;
    let inline_request = format!(r#"{{"receipt":{inline_fixture}}}"#);
    let inline = run_json(&[
        "mcp",
        "--call",
        "cairn_agent_receipt_validate",
        &inline_request,
    ])?;
    assert_eq!(inline["structuredContent"]["input_kind"], "inline");
    assert_eq!(inline["structuredContent"]["valid"], true);
    assert_eq!(
        inline["structuredContent"]["classification"],
        "legacy_unscoped"
    );

    let traversal = cairn()
        .args([
            "mcp",
            "--call",
            "cairn_agent_receipt_validate",
            r#"{"root":".","path":"../secret.json"}"#,
        ])
        .output()?;
    assert!(!traversal.status.success());
    let stderr = String::from_utf8_lossy(&traversal.stderr);
    assert!(
        stderr.contains("selector escapes lifecycle bounds"),
        "{stderr}"
    );
    Ok(())
}
