use cairn_core::blake3_hex;
use serde_json::json;
use std::{error::Error, fs, path::Path};

use super::{artifacts::artifact_schema, model::Report};

pub(super) fn render_case_receipt(
    root: &Path,
    case: &str,
    seed: &str,
    max: usize,
    artifact: &Path,
    report: &Report,
) -> Result<String, Box<dyn Error>> {
    let bytes = fs::read(artifact).unwrap_or_default();
    let rel = crate::support::safe_relative_path(root, artifact);
    let receipt = crate::support::with_portable_receipt_hash(
        root,
        json!({"schema":"cairn.test-harness.receipt.v1","command":"test-harness","case_id":case,"seed":seed,"root":".","outcome":report.outcome.s(),"failure_class":report.class.s(),"issue_code":report.code,"diagnostic":report.diag,"replay_command":format!("cairn test-harness --root . --case {case} --seed {seed}"),"artifact":{"path":rel,"schema":artifact_schema(&bytes),"blake3":blake3_hex(&bytes),"bytes":bytes.len()},"retry_policy":{"max_reruns":max,"same_case_identity":true},"attempts":report.attempts}),
    );
    Ok(serde_json::to_string_pretty(&receipt)?)
}
