use std::{error::Error, fs};

use serde_json::Value;

use super::support::{run_json_strings, EvidenceChain, RegistryChain};

pub(crate) fn verify(
    chain: &EvidenceChain,
    registry_chain: &RegistryChain,
) -> Result<(), Box<dyn Error>> {
    let dir = &chain.dir;
    let export_dir = &chain.export_dir;
    let registry_path = &registry_chain.registry_path;
    let registry_verify_path = &registry_chain.registry_verify_path;
    let selection_receipt_path = &registry_chain.selection_receipt_path;
    let fetch_plan_path = &registry_chain.fetch_plan_path;
    let registry_manifest_path = dir.join("registry-bound-manifest.json");
    let registry_manifest = run_json_strings(&[
        "evidence-manifest".into(),
        "generate".into(),
        "--bundle".into(),
        export_dir
            .join("agent-evidence-bundle.json")
            .display()
            .to_string(),
        "--agent-receipt".into(),
        export_dir.join("agent-receipt.json").display().to_string(),
        "--replay".into(),
        export_dir.join("replay-agent.json").display().to_string(),
        "--replay-verification".into(),
        export_dir
            .join("replay-verification.json")
            .display()
            .to_string(),
        "--registry".into(),
        registry_path.display().to_string(),
        "--registry-verification".into(),
        registry_verify_path.display().to_string(),
        "--registry-selection".into(),
        selection_receipt_path.display().to_string(),
        "--registry-fetch-plan".into(),
        fetch_plan_path.display().to_string(),
        "--out".into(),
        registry_manifest_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(registry_manifest["valid"], true);
    let registry_manifest_value: Value =
        serde_json::from_str(&fs::read_to_string(&registry_manifest_path)?)?;
    let registry_manifest_roles = registry_manifest_value["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["role"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(registry_manifest_roles.contains(&"evidence_registry"));
    assert!(registry_manifest_roles.contains(&"evidence_registry_verification"));
    assert!(registry_manifest_roles.contains(&"evidence_registry_selection"));
    assert!(registry_manifest_roles.contains(&"evidence_registry_fetch_plan"));
    let registry_manifest_verified = run_json_strings(&[
        "evidence-manifest".into(),
        "verify".into(),
        registry_manifest_path.display().to_string(),
        "--root".into(),
        ".".into(),
    ])?;
    assert_eq!(registry_manifest_verified["valid"], true);
    assert_eq!(
        registry_manifest_verified["manifest_hash"],
        registry_manifest["manifest_hash"]
    );
    let verified_registry_roles = registry_manifest_verified["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["role"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(verified_registry_roles.contains(&"evidence_registry"));
    assert!(verified_registry_roles.contains(&"evidence_registry_verification"));
    assert!(verified_registry_roles.contains(&"evidence_registry_selection"));
    assert!(verified_registry_roles.contains(&"evidence_registry_fetch_plan"));
    Ok(())
}
