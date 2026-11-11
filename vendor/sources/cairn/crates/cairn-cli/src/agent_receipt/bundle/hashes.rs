use std::path::Path;

use serde_json::Value;

use crate::support::portable_json_hash;

pub(crate) fn receipt_stated_or_computed_hash(root: &Path, receipt: &Value) -> Value {
    if let Some(hash) = receipt.get("receipt_hash").and_then(Value::as_str) {
        return Value::String(hash.to_string());
    }
    let mut input = receipt.clone();
    if let Some(object) = input.as_object_mut() {
        object.remove("receipt_hash");
    }
    Value::String(portable_json_hash(root, &input))
}
