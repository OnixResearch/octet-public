use std::{fs, path::PathBuf};

use cairn_core::{
    canonical_json_hash,
    verified::function_address_gates::{
        evaluate_function_address_evidence, normalize_function_address_evidence_digests,
        FunctionAddressEvidence, FUNCTION_ADDRESS_BOUNDARY,
    },
};
use serde_json::{json, Value};

use crate::support::load_policy_file;

pub(crate) fn function_address_receipt_value(
    policy_path: PathBuf,
    valence_receipt_path: Option<PathBuf>,
    mantle_binding_path: Option<PathBuf>,
    kamacite_receipt_path: Option<PathBuf>,
    graph_summary_path: Option<PathBuf>,
) -> Value {
    // r[impl cairn.function_address_gates.policy]
    // r[impl cairn.function_address_gates.receipt_validation]
    // r[impl cairn.function_address_gates.graph_summary]
    let policy = match load_policy_file(policy_path) {
        Ok(policy) => policy,
        Err(error) => {
            return function_address_error_value(
                "function_address.policy_unreadable",
                "policy",
                "parseable policy with function_address_gate",
                &error.to_string(),
            );
        }
    };
    let (valence_receipt, valence_issue) =
        read_optional_json(valence_receipt_path, "valence_receipt");
    let (mantle_binding, mantle_issue) = read_optional_json(mantle_binding_path, "mantle_binding");
    let (kamacite_receipt, kamacite_issue) =
        read_optional_json(kamacite_receipt_path, "kamacite_receipt");
    let (graph_summary, graph_issue) = read_optional_json(graph_summary_path, "graph_summary");
    let evidence = FunctionAddressEvidence {
        valence_receipt,
        mantle_binding,
        kamacite_receipt,
        graph_summary,
    };
    let normalization = normalize_function_address_evidence_digests(&evidence);
    let mut value = evaluate_function_address_evidence(
        &policy.function_address_policy,
        &normalization.evidence,
    )
    .to_json();
    for issue in normalization.issues {
        append_optional_function_address_issue(&mut value, Some(issue.to_json()));
    }
    append_optional_function_address_issue(&mut value, valence_issue);
    append_optional_function_address_issue(&mut value, mantle_issue);
    append_optional_function_address_issue(&mut value, kamacite_issue);
    append_optional_function_address_issue(&mut value, graph_issue);
    value
}

pub(crate) fn function_address_hash(value: &Value) -> String {
    canonical_json_hash(value)
}

fn read_optional_json(
    path: Option<PathBuf>,
    field: &'static str,
) -> (Option<Value>, Option<Value>) {
    let Some(path) = path else {
        return (None, None);
    };
    match fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| serde_json::from_str::<Value>(&text).map_err(|error| error.to_string()))
    {
        Ok(value) => (Some(value), None),
        Err(error) => (
            None,
            Some(json!({
                "code": "function_address.input_unreadable",
                "field_path": field,
                "expected": "readable JSON function-address input",
                "actual": error,
            })),
        ),
    }
}

fn append_optional_function_address_issue(value: &mut Value, issue: Option<Value>) {
    let Some(issue) = issue else {
        return;
    };
    if let Some(issues) = value.get_mut("issues").and_then(Value::as_array_mut) {
        issues.push(issue);
    }
    value["passed"] = Value::Bool(false);
    value["skipped"] = Value::Bool(false);
    value["disposition"] = Value::String("failed".to_string());
    value["evidence_chain_hash"] = Value::Null;
}

fn function_address_error_value(
    code: &str,
    field_path: &str,
    expected: &str,
    actual: &str,
) -> Value {
    json!({
        "name": "function_address_evidence",
        "mode": "unknown",
        "required": true,
        "passed": false,
        "skipped": false,
        "disposition": "failed",
        "issues": [{
            "code": code,
            "field_path": field_path,
            "expected": expected,
            "actual": actual,
        }],
        "valence_receipt_hash": null,
        "mantle_binding_hash": null,
        "kamacite_receipt_hash": null,
        "canonical_preserves_receipt_hash": null,
        "evidence_chain_hash": null,
        "json_projection_authority": null,
        "claim_scope": null,
        "graph_summary": null,
        "boundary": FUNCTION_ADDRESS_BOUNDARY,
    })
}
