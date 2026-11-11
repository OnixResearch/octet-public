use super::*;

pub(super) fn evidence_export_nested_issue(code: &str, issue: Value) -> Value {
    json!({
        "code": code,
        "field_path": issue.get("field_path").and_then(Value::as_str).unwrap_or("nested"),
        "expected": issue.get("expected").and_then(Value::as_str).unwrap_or("valid nested receipt"),
        "actual": issue.get("actual").and_then(Value::as_str).unwrap_or("invalid"),
    })
}

pub(super) fn evidence_export_issue(
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
