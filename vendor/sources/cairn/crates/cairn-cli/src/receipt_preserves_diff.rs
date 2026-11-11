use std::{collections::HashMap, error::Error, fs, path::PathBuf};

use cairn_core::{canonical_json_hash, receipt_field_drift, DriftReport};
use serde_json::{json, Value};

use crate::support::{
    default_policy_path, load_policy_file, path_string, print_json, read_option, with_receipt_hash,
};

pub(crate) fn run_receipt_preserves_diff(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.determinism.preserves_receipt_field_drift.command]
    // r[impl cairn.determinism.preserves_receipt_field_drift.read_only]
    // r[verify cairn.determinism.preserves_receipt_field_drift.command]
    // r[verify cairn.determinism.preserves_receipt_field_drift.read_only]

    let baseline_path = read_option(args.clone(), "--baseline", "");
    let candidate_path = read_option(args.clone(), "--candidate", "");

    if baseline_path.as_os_str().is_empty() || candidate_path.as_os_str().is_empty() {
        return Err("receipt-preserves-diff requires --baseline <path> --candidate <path>".into());
    }

    let baseline_text =
        fs::read_to_string(&baseline_path).map_err(|e| format!("failed to read baseline: {e}"))?;
    let candidate_text = fs::read_to_string(&candidate_path)
        .map_err(|e| format!("failed to read candidate: {e}"))?;

    let baseline: Value =
        serde_json::from_str(&baseline_text).map_err(|e| format!("baseline parse error: {e}"))?;
    let candidate: Value =
        serde_json::from_str(&candidate_text).map_err(|e| format!("candidate parse error: {e}"))?;

    let policy = load_policy_file(default_policy_path().into())?;
    let source_map = parse_source_id_map(&policy);

    let report = receipt_field_drift(&baseline, &candidate, &source_map);

    let receipt = diff_receipt_json(&baseline_path, &candidate_path, &report, &policy);
    let valid = report.entries.is_empty();
    print_json(with_receipt_hash(receipt))?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

/// Parse the policy-owned path→source-id mapping from generated policy JSON.
///
/// Looks for a `preserves_receipt_source_map` object under the top-level
/// `structured_lifecycle` policy section.  Each key is an RFC 6901 JSON
/// Pointer, each value is a requirement ID string.
pub(crate) fn parse_source_id_map(policy: &cairn_core::Policy) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Some(source_map) = policy
        .raw
        .get("structured_lifecycle")
        .and_then(|v| v.get("preserves_receipt_source_map"))
        .and_then(Value::as_object)
    {
        for (path, id_val) in source_map {
            if let Some(id) = id_val.as_str() {
                map.insert(path.clone(), id.to_string());
            }
        }
    }
    map
}

fn drift_entry_json(entry: &cairn_core::DriftEntry) -> Value {
    json!({
        "field_path": entry.field_path,
        "class": entry.class.as_str(),
        "expected": entry.expected,
        "actual": entry.actual,
        "source_id": entry.source_id,
    })
}

pub(crate) fn diff_receipt_json(
    baseline_path: &PathBuf,
    candidate_path: &PathBuf,
    report: &DriftReport,
    policy: &cairn_core::Policy,
) -> Value {
    let drift_entries: Vec<Value> = report.entries.iter().map(drift_entry_json).collect();
    json!({
        "command": "receipt-preserves-diff",
        "baseline_path": path_string(baseline_path),
        "candidate_path": path_string(candidate_path),
        "valid": report.entries.is_empty(),
        "mutated": false,
        "baseline_json_hash": report.baseline_json_hash,
        "candidate_json_hash": report.candidate_json_hash,
        "baseline_preserves_hash": report.baseline_preserves_hash,
        "candidate_preserves_hash": report.candidate_preserves_hash,
        "canonical_bytes_differ": report.canonical_bytes_differ,
        "drift_count": report.entries.len(),
        "drift": drift_entries,
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
    })
}
