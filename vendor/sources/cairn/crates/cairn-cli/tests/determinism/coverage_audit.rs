use std::error::Error;

use serde_json::json;

use super::support::{assert_hash_field, policy_json, receipt_schema, run_json};

#[test]
fn determinism_coverage_audit_reports_policy_surface_coverage() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.determinism.preserves_hash_cli.surface_inventory]
    // r[verify cairn.determinism.preserves_hash_cli.replay_validation]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.cli]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.inventory]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.failures]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.docs]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.gates]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.archive]
    // r[verify cairn.supersede.determinism_surface_coverage_audit.tests]
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.surface]
    // r[verify cairn.determinism.preserves_corpus_policy_inventory.tests]
    let first = run_json(&["determinism", "coverage-audit", "--root", "."])?;
    let second = run_json(&["determinism", "coverage-audit", "--root", "."])?;
    assert_eq!(first["valid"], true);
    assert_eq!(first["verdict"], "pass");
    assert_eq!(first["receipt_hash"], second["receipt_hash"]);
    assert_eq!(first["issues"].as_array().unwrap().len(), 0);
    assert_eq!(first["missing_surfaces"].as_array().unwrap().len(), 0);
    assert_hash_field(&first, "receipt_hash")?;
    assert_hash_field(&first, "current_replay_policy_hash")?;
    assert_hash_field(&first, "surface_inventory_hash")?;

    let policy = policy_json()?;
    let surfaces = policy["determinism_surface_inventory"].as_array().unwrap();
    assert_eq!(first["surface_count"], surfaces.len());
    assert_eq!(
        first["covered_surfaces"].as_array().unwrap().len(),
        surfaces.len()
    );
    let covered_ids = first["covered_surfaces"]
        .as_array()
        .unwrap()
        .iter()
        .map(|surface| surface["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        covered_ids,
        vec![
            "agent_completion_validate",
            "agent_receipt_preserves_hash",
            "agent_receipt_validate",
            "archive_dry_run",
            "evidence_registry_build",
            "evidence_registry_fetch_execute",
            "evidence_registry_fetch_plan",
            "evidence_registry_mirror_verify",
            "evidence_registry_select",
            "evidence_registry_verify",
            "gate_design",
            "gate_proposal",
            "gate_tasks",
            "mcp_describe",
            "mcp_status",
            "migrate_dry_run",
            "preserves_corpus_freshness",
            "receipt_preserves_diff",
            "release_readiness",
            "status",
            "sync_dry_run",
            "validate",
            "view_graph",
            "view_kanban",
            "view_mindmap"
        ]
    );
    let schema = receipt_schema("determinism coverage-audit")?;
    assert!(schema["top_level_keys"]
        .as_array()
        .unwrap()
        .contains(&json!("surface_inventory_hash")));
    assert!(schema["issue_keys"]
        .as_array()
        .unwrap()
        .contains(&json!("surface_id")));
    Ok(())
}
