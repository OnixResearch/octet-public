use super::issue::evidence_export_issue;
use super::*;
use crate::support::CapabilityRoot;

pub(super) fn evidence_export_artifacts(
    output_root: &CapabilityRoot,
    out_dir: &Path,
    issues: &mut Vec<Value>,
) -> Vec<Value> {
    let roles = [
        ("agent_evidence_bundle", "agent-evidence-bundle.json", true),
        (
            "agent_evidence_bundle_verification",
            "agent-evidence-bundle-verification.json",
            true,
        ),
        ("agent_receipt", "agent-receipt.json", true),
        ("coverage_audit", "coverage-audit.json", true),
        ("evidence_manifest", "evidence-manifest.json", true),
        (
            "evidence_manifest_verification",
            "evidence-manifest-verification.json",
            true,
        ),
        ("release_readiness", "release-readiness.json", true),
        ("replay_receipt", "replay-agent.json", true),
        ("replay_verification", "replay-verification.json", true),
        ("runtime_evidence", "runtime-evidence.json", false),
        ("runtime_chain", "runtime-chain.json", false),
        ("witness_checkpoint", "witness-checkpoint.json", false),
    ];
    roles
        .iter()
        .filter_map(|(role, relative, required)| {
            let path = out_dir.join(relative);
            let hash = match output_root.read_path(&path) {
                Ok(bytes) => Value::String(blake3_hex(&bytes)),
                Err(_) => {
                    if *required {
                        issues.push(evidence_export_issue(
                            "export.artifact_missing",
                            role,
                            "generated artifact file",
                            "missing",
                        ));
                        Value::Null
                    } else {
                        return None;
                    }
                }
            };
            Some(json!({
                "role": role,
                "path": relative,
                "hash": hash,
                "required": required,
            }))
        })
        .collect()
}
