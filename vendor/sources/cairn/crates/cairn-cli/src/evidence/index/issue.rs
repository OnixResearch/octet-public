use serde_json::{json, Value};

pub(super) fn evidence_index_issue(
    code: &str,
    field_path: &str,
    expected: &str,
    actual: &str,
) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}

pub(super) fn issues_with_index_missing() -> Vec<Value> {
    vec![evidence_index_issue(
        "evidence_index_verify.index_unreadable",
        "index",
        "readable index receipt",
        "unreadable",
    )]
}
