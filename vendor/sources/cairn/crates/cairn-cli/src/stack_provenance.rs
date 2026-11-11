use std::{fs, path::PathBuf};

use cairn_core::{
    canonical_json_hash,
    verified::stack_provenance::{
        evaluate_stack_provenance, StackProvenanceEvidence, STACK_PROVENANCE_BOUNDARY,
    },
};
use serde_json::{json, Value};

use crate::support::load_policy_file;

pub(crate) fn stack_provenance_receipt_value(
    policy_path: PathBuf,
    receipt_path: Option<PathBuf>,
    graph_path: Option<PathBuf>,
) -> Value {
    // r[impl cairn.stack_provenance_gates.policy]
    // r[impl cairn.stack_provenance_gates.receipt_validation]
    // r[impl cairn.stack_provenance_gates.graph_summary]
    let policy = match load_policy_file(policy_path) {
        Ok(policy) => policy,
        Err(error) => {
            return stack_provenance_error_value(
                "stack_provenance.policy_unreadable",
                "policy",
                "parseable policy with stack_provenance_gate",
                &error.to_string(),
            );
        }
    };
    let (valence_receipt, receipt_issue) = read_optional_json(receipt_path, "valence_receipt");
    let (graph_summary, graph_issue) = read_optional_json(graph_path, "graph_summary");
    let evidence = StackProvenanceEvidence {
        valence_receipt,
        graph_summary,
    };
    let mut value = evaluate_stack_provenance(&policy.stack_provenance_policy, &evidence).to_json();
    append_optional_stack_issue(&mut value, receipt_issue);
    append_optional_stack_issue(&mut value, graph_issue);
    value
}

pub(crate) fn stack_provenance_hash(value: &Value) -> String {
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
                "code": "stack_provenance.input_unreadable",
                "field_path": field,
                "expected": "readable JSON stack provenance input",
                "actual": error,
            })),
        ),
    }
}

fn append_optional_stack_issue(value: &mut Value, issue: Option<Value>) {
    let Some(issue) = issue else {
        return;
    };
    if let Some(issues) = value.get_mut("issues").and_then(Value::as_array_mut) {
        issues.push(issue);
    }
    value["passed"] = Value::Bool(false);
    value["skipped"] = Value::Bool(false);
    value["disposition"] = Value::String("failed".to_string());
}

fn stack_provenance_error_value(
    code: &str,
    field_path: &str,
    expected: &str,
    actual: &str,
) -> Value {
    json!({
        "name": "stack_provenance",
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
        "claim_scope": null,
        "graph_summary": null,
        "boundary": STACK_PROVENANCE_BOUNDARY,
    })
}
