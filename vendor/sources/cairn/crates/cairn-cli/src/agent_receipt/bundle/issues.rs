use serde_json::{json, Value};

pub(super) fn verify_optional_bundle_hash(
    issues: &mut Vec<Value>,
    bundle: &Value,
    field: &str,
    actual: &Value,
) {
    let Some(expected) = bundle.get(field) else {
        return;
    };
    if expected != actual {
        issues.push(agent_bundle_issue(
            "bundle.expected_hash_mismatch",
            field,
            expected.as_str().unwrap_or("hash"),
            actual.as_str().unwrap_or("missing"),
        ));
    }
}

pub(super) fn compare_bundle_field(
    issues: &mut Vec<Value>,
    code: &str,
    field_path: &str,
    expected: &Value,
    actual: &Value,
) {
    if expected != actual {
        issues.push(agent_bundle_issue(
            code,
            field_path,
            expected.as_str().unwrap_or("value"),
            actual.as_str().unwrap_or("missing"),
        ));
    }
}

pub(super) fn agent_bundle_issue(
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
