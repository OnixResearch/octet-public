use serde_json::{json, Value};

pub(super) fn evidence_manifest_issue(
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
