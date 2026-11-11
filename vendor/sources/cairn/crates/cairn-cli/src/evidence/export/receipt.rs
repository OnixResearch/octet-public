use super::issue::evidence_export_issue;
use super::*;
use crate::support::CapabilityRoot;

pub(crate) fn write_evidence_json(
    output_root: &CapabilityRoot,
    path: &Path,
    value: &Value,
    issues: &mut Vec<Value>,
    role: &str,
) {
    if let Some(parent) = path.parent() {
        if output_root.create_dir_all_path(parent).is_err() {
            issues.push(evidence_export_issue(
                "export.artifact_parent_create_failed",
                role,
                "writable parent directory",
                "failed",
            ));
            return;
        }
    }
    if output_root
        .write_path(
            path,
            format!(
                "{}\n",
                serde_json::to_string_pretty(value).unwrap_or_default()
            )
            .as_bytes(),
        )
        .is_err()
    {
        issues.push(evidence_export_issue(
            "export.artifact_write_failed",
            role,
            "writable JSON artifact",
            "failed",
        ));
    }
}

pub(crate) fn evidence_export_final_receipt(
    root: &Path,
    _out_dir: &Path,
    mut issues: Vec<Value>,
    artifacts: Vec<Value>,
    manifest_hash: Value,
    manifest_verification_receipt_hash: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "evidence export",
            "root": path_string(root),
            "output": "<out>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "artifacts": artifacts,
            "manifest_hash": manifest_hash,
            "manifest_verification_receipt_hash": manifest_verification_receipt_hash,
        }),
    )
}
