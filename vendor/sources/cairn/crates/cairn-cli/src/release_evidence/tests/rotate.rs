use std::{env, fs};

use serde_json::json;

use super::super::rotate::{release_evidence_rotate_receipt, ReleaseEvidenceRotateOptions};
use super::helpers::write_publish_tree;
use crate::support::path_string;

#[test]
fn rotate_dry_run_does_not_delete_prune_candidate() {
    // r[verify release_evidence.retention_rotation.rotation_execution]
    // r[verify release_evidence.retention_rotation.rotation_execution.verify_before_mutation]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-rotate-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let publish_root = root.join("published");
    write_publish_tree(&publish_root.join("old"), "stable", "2026-05-01", "short");
    let plan = root.join("plan.json");
    fs::write(
        &plan,
        serde_json::to_vec_pretty(&json!({
            "schema": "cairn.release-evidence.retention-plan.v1",
            "publish_root": path_string(&publish_root),
            "decisions": [{
                "path": "old",
                "action": "prune_candidate",
                "manifest_hash": "unused",
                "receipt_hash": "unused",
                "labels": {"channel": "stable", "release": "2026-05-01"},
                "valid": true,
            }]
        }))
        .expect("serialize plan"),
    )
    .expect("write plan");
    let receipt = release_evidence_rotate_receipt(&ReleaseEvidenceRotateOptions {
        root: root.clone(),
        plan,
        execute: false,
    });
    assert_eq!(receipt["valid"].as_bool(), Some(true));
    assert!(publish_root.join("old").exists());
    assert_eq!(receipt["mutation_count"].as_u64(), Some(1));
    fs::remove_dir_all(&root).expect("remove temp tree");
}
