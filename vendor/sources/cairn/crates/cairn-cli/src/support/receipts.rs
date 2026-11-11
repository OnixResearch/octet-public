use super::*;

pub(crate) fn print_json(value: Value) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

pub(crate) fn with_receipt_hash(mut value: Value) -> Value {
    let receipt_hash = canonical_json_hash(&value);
    value["receipt_hash"] = json!(receipt_hash);
    value
}

pub(crate) fn with_portable_receipt_hash(root: &Path, mut value: Value) -> Value {
    let receipt_hash = portable_json_hash(root, &value);
    value["receipt_hash"] = json!(receipt_hash);
    value
}

// r[impl cairn.determinism.preserves_content_hash_binding_readiness.binding]
pub(crate) fn with_portable_content_hash_receipt(root: &Path, mut value: Value) -> Value {
    let portable_value = super::paths::portable_json(root, &value);
    let hash = preserves_content_hash(&portable_value).unwrap_or_else(|error| {
        format!("preserves_content_hash_error:{}", error.replace('\"', "'"))
    });
    value["preserves_content_hash"] = json!(hash);
    with_portable_receipt_hash(root, value)
}
