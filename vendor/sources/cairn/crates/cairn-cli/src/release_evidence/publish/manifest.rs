use serde_json::{json, Value};

use super::labels::release_evidence_publish_labels;
use super::options::ReleaseEvidencePublishOptions;

pub(super) fn release_evidence_publish_manifest(
    options: &ReleaseEvidencePublishOptions,
    payload_commit: String,
    source_index_hash: String,
    source_index_receipt_hash: Value,
    materialized: Vec<Value>,
) -> Value {
    json!({
        "schema": "cairn.release-evidence.publish-manifest.v1",
        "layout_version": "release-evidence-publish-v1",
        "payload_commit": payload_commit,
        "source_index_hash": source_index_hash,
        "source_index_receipt_hash": source_index_receipt_hash,
        "labels": release_evidence_publish_labels(options),
        "materialized": materialized,
        "non_claims": [
            "local_producer_materialization_only",
            "no_network_transport",
            "no_credential_validation",
            "no_remote_mirror_trust",
            "no_signature_validation",
            "no_tag_movement",
            "no_public_unattended_production_readiness",
        ],
    })
}
