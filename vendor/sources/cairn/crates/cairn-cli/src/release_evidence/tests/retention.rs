use std::{env, fs};

use super::super::retention::{
    release_evidence_retention_plan_receipt, ReleaseEvidenceRetentionOptions,
};
use super::helpers::write_publish_tree;

#[test]
fn retention_plan_marks_newest_keep_and_older_prune_candidate() {
    // r[verify release_evidence.retention_rotation.plan_receipt]
    // r[verify release_evidence.retention_rotation.plan_receipt.decisions]
    // r[verify release_evidence.retention_rotation.plan_receipt.ordering]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-retention-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let publish_root = root.join("published");
    write_publish_tree(
        &publish_root.join("stable/2026-05-01"),
        "stable",
        "2026-05-01",
        "short",
    );
    write_publish_tree(
        &publish_root.join("stable/2026-05-02"),
        "stable",
        "2026-05-02",
        "short",
    );
    let out = root.join("plan.json");
    let receipt = release_evidence_retention_plan_receipt(&ReleaseEvidenceRetentionOptions {
        root: root.clone(),
        publish_root: publish_root.clone(),
        out: out.clone(),
        channel: Some("stable".to_string()),
        keep_latest: 1,
        retain: None,
    });
    assert_eq!(receipt["valid"].as_bool(), Some(true));
    let decisions = receipt["decisions"].as_array().expect("decisions");
    assert!(decisions.iter().any(|decision| {
        decision["path"].as_str() == Some("stable/2026-05-02")
            && decision["action"].as_str() == Some("keep")
    }));
    assert!(decisions.iter().any(|decision| {
        decision["path"].as_str() == Some("stable/2026-05-01")
            && decision["action"].as_str() == Some("prune_candidate")
    }));
    assert!(out.exists());
    fs::remove_dir_all(&root).expect("remove temp tree");
}

#[test]
fn retention_rejects_duplicate_release_identity() {
    // r[verify release_evidence.retention_rotation.fail_closed]
    // r[verify release_evidence.retention_rotation.fail_closed.unsafe_or_stale]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-retention-duplicate-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let publish_root = root.join("published");
    write_publish_tree(&publish_root.join("a"), "stable", "2026-05-01", "short");
    write_publish_tree(&publish_root.join("b"), "stable", "2026-05-01", "short");
    let receipt = release_evidence_retention_plan_receipt(&ReleaseEvidenceRetentionOptions {
        root: root.clone(),
        publish_root,
        out: root.join("plan.json"),
        channel: Some("stable".to_string()),
        keep_latest: 1,
        retain: None,
    });
    assert_eq!(receipt["valid"].as_bool(), Some(false));
    assert!(receipt["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|issue| {
            issue["code"].as_str() == Some("release_evidence_retention.duplicate_channel_release")
        }));
    fs::remove_dir_all(&root).expect("remove temp tree");
}
