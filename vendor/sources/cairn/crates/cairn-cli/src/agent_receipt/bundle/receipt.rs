use std::path::Path;

use serde_json::{json, Value};

use crate::support::{path_string, with_portable_content_hash_receipt};

use super::REQUIRED_AGENT_REPLAY_GROUP;

#[allow(clippy::too_many_arguments)]
pub(super) fn agent_bundle_receipt(
    root: &Path,
    issues: Vec<Value>,
    current_replay_policy_hash: String,
    agent_receipt_hash: Value,
    replay_receipt_hash: Value,
    replay_verification_receipt_hash: Value,
    environment_attestation_hash: Value,
    release_readiness_receipt_hash: Value,
    coverage_audit_receipt_hash: Value,
    runtime_evidence_receipt_hash: Value,
) -> Value {
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "agent-receipt verify-bundle",
            "root": path_string(root),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "current_replay_policy_hash": current_replay_policy_hash,
            "required_replay_group": REQUIRED_AGENT_REPLAY_GROUP,
            "agent_receipt_hash": agent_receipt_hash,
            "replay_receipt_hash": replay_receipt_hash,
            "replay_verification_receipt_hash": replay_verification_receipt_hash,
            "environment_attestation_hash": environment_attestation_hash,
            "release_readiness_receipt_hash": release_readiness_receipt_hash,
            "coverage_audit_receipt_hash": coverage_audit_receipt_hash,
            "runtime_evidence_receipt_hash": runtime_evidence_receipt_hash,
        }),
    )
}
