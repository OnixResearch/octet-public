use std::{env, fs};

use serde_json::json;

use super::super::common::safe_relative_path_text;
use super::super::publish::{copy_publish_artifact, validate_publish_label};
use cairn_core::blake3_hex;

#[test]
fn publish_materialization_copies_safe_artifact_and_binds_hash() {
    // r[verify release_evidence.publish_materialization.manifest_receipt]
    // r[verify release_evidence.publish_materialization.manifest_receipt.artifact_binding]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-publish-copy-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let source_bundle = root.join("bundle");
    let publish_dir = root.join("published");
    fs::create_dir_all(&source_bundle).expect("source bundle");
    let content = b"release evidence\n";
    fs::write(source_bundle.join("receipt.json"), content).expect("source artifact");
    let artifact = json!({
        "role": "release_readiness",
        "path": "receipt.json",
        "hash": blake3_hex(content),
    });
    let mut materialized = Vec::new();
    let mut issues = Vec::new();

    copy_publish_artifact(
        &artifact,
        &source_bundle,
        &publish_dir,
        &mut materialized,
        &mut issues,
    );

    assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    assert_eq!(materialized.len(), 1);
    assert_eq!(
        materialized[0]["path"].as_str(),
        Some("artifacts/receipt.json")
    );
    assert_eq!(
        fs::read(publish_dir.join("artifacts/receipt.json")).expect("copied artifact"),
        content
    );
    fs::remove_dir_all(&root).expect("remove temp tree");
}

#[test]
fn publish_materialization_rejects_unsafe_paths_and_labels() {
    // r[verify release_evidence.publish_materialization.fail_closed]
    // r[verify release_evidence.publish_materialization.fail_closed.path_or_hash]
    // r[verify release_evidence.publish_materialization.manifest_receipt.metadata]
    assert!(!safe_relative_path_text("../escape.json"));
    let mut issues = Vec::new();
    validate_publish_label("channel", Some("../stable"), &mut issues);
    assert!(issues
        .iter()
        .any(|issue| issue["code"].as_str() == Some("release_evidence_publish.label_invalid")));
}
