use std::{error::Error, fs};

use serde_json::Value;

use super::support::{
    assert_hash_field, assert_object_keys, repo_root, run_json_strings, temp_project, EvidenceChain,
};

pub(crate) fn materialize() -> Result<EvidenceChain, Box<dyn Error>> {
    // r[verify cairn.supersede.evidence_export_bundle.command]
    // r[verify cairn.supersede.evidence_export_bundle.artifacts]
    // r[verify cairn.supersede.evidence_export_bundle.failures]
    // r[verify cairn.supersede.evidence_export_bundle.gates]
    // r[verify cairn.supersede.evidence_export_bundle.docs]
    // r[verify cairn.supersede.evidence_export_bundle.archive]
    // r[verify cairn.supersede.evidence_bundle_verify_command.command]
    // r[verify cairn.supersede.evidence_bundle_verify_command.artifacts]
    // r[verify cairn.supersede.evidence_bundle_verify_command.failures]
    // r[verify cairn.supersede.evidence_bundle_verify_command.gates]
    // r[verify cairn.supersede.evidence_bundle_verify_command.docs]
    // r[verify cairn.supersede.evidence_bundle_verify_command.archive]
    // r[verify cairn.supersede.evidence_import_consume_rail.command]
    // r[verify cairn.supersede.evidence_import_consume_rail.artifacts]
    // r[verify cairn.supersede.evidence_import_consume_rail.failures]
    // r[verify cairn.supersede.evidence_import_consume_rail.gates]
    // r[verify cairn.supersede.evidence_import_consume_rail.archive]
    // r[verify cairn.supersede.evidence_import_republish_rail.command]
    // r[verify cairn.supersede.evidence_import_republish_rail.artifacts]
    // r[verify cairn.supersede.evidence_import_republish_rail.failures]
    // r[verify cairn.supersede.evidence_import_republish_rail.gates]
    // r[verify cairn.supersede.evidence_import_republish_rail.archive]
    // r[verify cairn.supersede.evidence_publish_index_rail.command]
    // r[verify cairn.supersede.evidence_publish_index_rail.verify]
    // r[verify cairn.supersede.evidence_publish_index_rail.failures]
    // r[verify cairn.supersede.evidence_publish_index_rail.gates]
    // r[verify cairn.supersede.evidence_publish_index_rail.archive]
    // r[verify cairn.supersede.evidence_registry_rail.build]
    // r[verify cairn.supersede.evidence_registry_rail.verify]
    // r[verify cairn.supersede.evidence_registry_rail.failures]
    // r[verify cairn.supersede.evidence_registry_rail.gates]
    // r[verify cairn.supersede.evidence_registry_rail.archive]
    // r[verify cairn.supersede.evidence_registry_consume.select]
    // r[verify cairn.supersede.evidence_registry_consume.fetch_plan]
    // r[verify cairn.supersede.evidence_registry_consume.policy]
    // r[verify cairn.supersede.evidence_registry_consume.docs]
    // r[verify cairn.supersede.evidence_registry_consume.verify]
    // r[verify cairn.supersede.registry_channel_metadata.labels]
    // r[verify cairn.supersede.registry_channel_metadata.policy]
    // r[verify cairn.supersede.registry_channel_metadata.build]
    // r[verify cairn.supersede.registry_channel_metadata.consume]
    // r[verify cairn.supersede.registry_channel_metadata.verify]
    // r[verify cairn.supersede.registry_manifest_binding.roles]
    // r[verify cairn.supersede.registry_manifest_binding.generate]
    // r[verify cairn.supersede.registry_manifest_binding.verify]
    // r[verify cairn.supersede.registry_manifest_binding.failure]
    // r[verify cairn.verification_role.evidence_verify]
    // r[verify cairn.verification_role.evidence_verify.summary]
    let dir = temp_project("evidence-export-bundle")?;
    let export_dir = dir.join("export");
    let receipt = run_json_strings(&[
        "evidence".into(),
        "export".into(),
        "--root".into(),
        ".".into(),
        "--out".into(),
        export_dir.display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(receipt["issues"].as_array().unwrap().len(), 0);
    assert_hash_field(&receipt, "manifest_hash")?;
    assert_hash_field(&receipt, "manifest_verification_receipt_hash")?;
    assert_hash_field(&receipt, "receipt_hash")?;
    let roles = receipt["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|artifact| artifact["role"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        roles,
        vec![
            "agent_evidence_bundle",
            "agent_evidence_bundle_verification",
            "agent_receipt",
            "coverage_audit",
            "evidence_manifest",
            "evidence_manifest_verification",
            "release_readiness",
            "replay_receipt",
            "replay_verification",
        ]
    );
    for artifact in receipt["artifacts"].as_array().unwrap() {
        assert_hash_field(artifact, "hash")?;
        assert_eq!(artifact["required"], true);
        let path = artifact["path"].as_str().unwrap();
        assert!(!path.contains(&dir.display().to_string()), "{path}");
        assert!(
            export_dir.join(path).exists(),
            "missing exported artifact: {path}"
        );
    }
    let saved_receipt: Value =
        serde_json::from_slice(&fs::read(export_dir.join("evidence-export-receipt.json"))?)?;
    assert_eq!(saved_receipt["receipt_hash"], receipt["receipt_hash"]);
    let bundle_verification: Value = serde_json::from_slice(&fs::read(
        export_dir.join("agent-evidence-bundle-verification.json"),
    )?)?;
    assert_eq!(bundle_verification["valid"], true);
    let manifest_verification: Value = serde_json::from_slice(&fs::read(
        export_dir.join("evidence-manifest-verification.json"),
    )?)?;
    assert_eq!(manifest_verification["valid"], true);
    assert_eq!(
        manifest_verification["receipt_hash"],
        receipt["manifest_verification_receipt_hash"]
    );
    assert_eq!(
        manifest_verification["manifest_hash"],
        receipt["manifest_hash"]
    );
    let text = serde_json::to_string(&receipt)?;
    assert!(!text.contains(&dir.display().to_string()), "{text}");
    assert!(!text.contains("SECRET"), "{text}");

    let verification = run_json_strings(&[
        "evidence".into(),
        "verify".into(),
        "--root".into(),
        ".".into(),
        "--bundle".into(),
        export_dir.display().to_string(),
    ])?;
    assert_eq!(verification["valid"], true);
    assert_eq!(verification["verdict"], "pass");
    assert_eq!(verification["issues"].as_array().unwrap().len(), 0);
    assert_eq!(verification["bundle"], "<bundle>");
    assert_eq!(verification["export_receipt_hash"], receipt["receipt_hash"]);
    assert_eq!(verification["manifest_hash"], receipt["manifest_hash"]);
    assert_eq!(
        verification["manifest_verification_receipt_hash"],
        receipt["manifest_verification_receipt_hash"]
    );
    assert_hash_field(&verification, "receipt_hash")?;
    assert_object_keys(
        &verification["verification_role_summary"],
        &[
            "boundary",
            "boundary_count",
            "property_count",
            "recorded_only_count",
            "total",
            "unknown_count",
            "verified_count",
        ],
    )?;
    assert_eq!(
        verification["verification_role_summary"]["total"],
        roles.len()
    );
    assert_eq!(
        verification["verification_role_summary"]["verified_count"],
        verification["verification_role_summary"]["property_count"]
    );
    let verification_text = serde_json::to_string(&verification)?;
    assert!(
        !verification_text.contains(&dir.display().to_string()),
        "{verification_text}"
    );

    let stack_verification = run_json_strings(&[
        "evidence".into(),
        "verify".into(),
        "--root".into(),
        ".".into(),
        "--bundle".into(),
        export_dir.display().to_string(),
        "--stack-provenance".into(),
        repo_root()
            .join("fixtures/stack-provenance/positive/required-complete.json")
            .display()
            .to_string(),
        "--stack-provenance-graph".into(),
        repo_root()
            .join("fixtures/stack-provenance/positive/graph-summary.json")
            .display()
            .to_string(),
    ])?;
    assert_eq!(stack_verification["valid"], true);
    assert_eq!(stack_verification["stack_provenance"]["passed"], true);
    assert_hash_field(&stack_verification, "stack_provenance_hash")?;
    assert_hash_field(
        &stack_verification["stack_provenance"]["graph_summary"],
        "report_hash",
    )?;

    let import_dir = dir.join("imported");
    let import = run_json_strings(&[
        "evidence".into(),
        "import".into(),
        "--root".into(),
        ".".into(),
        "--bundle".into(),
        export_dir.display().to_string(),
        "--into".into(),
        import_dir.display().to_string(),
    ])?;
    assert_eq!(import["valid"], true);
    assert_eq!(import["verdict"], "pass");
    assert_eq!(import["bundle"], "<bundle>");
    assert_eq!(import["into"], "<into>");
    assert_eq!(
        import["verification_receipt_hash"],
        verification["receipt_hash"]
    );
    assert_eq!(import["artifacts"].as_array().unwrap().len(), roles.len());
    assert_hash_field(&import, "receipt_hash")?;
    let saved_import: Value =
        serde_json::from_slice(&fs::read(import_dir.join("evidence-import-receipt.json"))?)?;
    assert_eq!(saved_import["receipt_hash"], import["receipt_hash"]);
    for artifact in import["artifacts"].as_array().unwrap() {
        assert_hash_field(artifact, "hash")?;
        let path = artifact["path"].as_str().unwrap();
        assert!(
            import_dir.join(path).exists(),
            "missing imported artifact: {path}"
        );
        assert_eq!(
            fs::read(import_dir.join(path))?,
            fs::read(export_dir.join(path))?,
            "imported artifact bytes must match export for {path}"
        );
    }
    let import_text = serde_json::to_string(&import)?;
    assert!(
        !import_text.contains(&dir.display().to_string()),
        "{import_text}"
    );

    let republish_dir = dir.join("republished");
    let republish = run_json_strings(&[
        "evidence".into(),
        "republish".into(),
        "--root".into(),
        ".".into(),
        "--imported".into(),
        import_dir.display().to_string(),
        "--out".into(),
        republish_dir.display().to_string(),
    ])?;
    assert_eq!(republish["valid"], true);
    assert_eq!(republish["verdict"], "pass");
    assert_eq!(republish["imported"], "<imported>");
    assert_eq!(republish["out"], "<out>");
    assert_eq!(republish["import_receipt_hash"], import["receipt_hash"]);
    assert_eq!(
        republish["artifacts"].as_array().unwrap().len(),
        roles.len()
    );
    assert_hash_field(&republish, "receipt_hash")?;
    let saved_republish: Value = serde_json::from_slice(&fs::read(
        republish_dir.join("evidence-republish-receipt.json"),
    )?)?;
    assert_eq!(saved_republish["receipt_hash"], republish["receipt_hash"]);
    assert!(republish_dir.join("evidence-import-receipt.json").exists());
    for artifact in republish["artifacts"].as_array().unwrap() {
        assert_hash_field(artifact, "hash")?;
        let path = artifact["path"].as_str().unwrap();
        assert!(
            republish_dir.join(path).exists(),
            "missing republished artifact: {path}"
        );
        assert_eq!(
            fs::read(republish_dir.join(path))?,
            fs::read(import_dir.join(path))?,
            "republished artifact bytes must match import for {path}"
        );
    }
    let republish_text = serde_json::to_string(&republish)?;
    assert!(
        !republish_text.contains(&dir.display().to_string()),
        "{republish_text}"
    );

    let index_path = republish_dir.join("evidence-index-receipt.json");
    let index = run_json_strings(&[
        "evidence".into(),
        "index".into(),
        "--root".into(),
        ".".into(),
        "--bundle".into(),
        republish_dir.display().to_string(),
        "--out".into(),
        index_path.display().to_string(),
    ])?;
    assert_eq!(index["valid"], true);
    assert_eq!(index["verdict"], "pass");
    assert_eq!(index["bundle"], "<bundle>");
    assert_eq!(index["out"], "<out>");
    assert_eq!(index["bundle_kind"], "republished");
    assert_eq!(index["bundle_receipt_hash"], republish["receipt_hash"]);
    assert_eq!(
        index["artifacts"].as_array().unwrap().len(),
        republish["artifacts"].as_array().unwrap().len()
    );
    assert_hash_field(&index, "receipt_hash")?;
    let saved_index: Value = serde_json::from_slice(&fs::read(&index_path)?)?;
    assert_eq!(saved_index["receipt_hash"], index["receipt_hash"]);
    let index_verify = run_json_strings(&[
        "evidence".into(),
        "index".into(),
        "verify".into(),
        "--root".into(),
        ".".into(),
        "--index".into(),
        index_path.display().to_string(),
    ])?;
    assert_eq!(index_verify["valid"], true);
    assert_eq!(index_verify["verdict"], "pass");
    assert_eq!(index_verify["index_receipt_hash"], index["receipt_hash"]);
    assert_eq!(index_verify["bundle_kind"], "republished");
    assert_hash_field(&index_verify, "receipt_hash")?;
    Ok(EvidenceChain {
        dir,
        export_dir,
        import_dir,
        republish_dir,
        index_path,
        republish,
        index,
    })
}
