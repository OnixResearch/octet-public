use std::{error::Error, fs};

use cairn_core::{canonical_json_hash, preserves_content_hash};
use serde_json::{json, Value};

use crate::support::{print_json, read_option, with_receipt_hash};

pub(crate) fn run_receipt_verify_content_hash(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.determinism.preserves_authoritative_content_hash.verification]
    // r[verify cairn.determinism.preserves_authoritative_content_hash.verification]
    // r[verify cairn.determinism.preserves_authoritative_content_hash.validation]

    let path = read_option(args, "--path", "");
    if path.as_os_str().is_empty() {
        return Err("receipt verify-content-hash requires --path <receipt.json>".into());
    }

    let text = fs::read_to_string(&path).map_err(|e| format!("failed to read receipt: {e}"))?;
    let receipt: Value =
        serde_json::from_str(&text).map_err(|e| format!("receipt parse error: {e}"))?;

    let receipt_object = receipt.as_object().ok_or_else(|| {
        format!(
            "receipt root must be a JSON object, got {}",
            json_type_name(&receipt)
        )
    })?;

    let stated_hash = receipt_object
        .get("preserves_content_hash")
        .and_then(Value::as_str)
        .map(str::to_string);

    let computed = preserves_content_hash(&receipt)?;
    let valid = match &stated_hash {
        Some(stated) => *stated == computed,
        None => {
            // No stated hash — the field is optional; we report the computed
            // hash but do not fail.  Compatibility mode.
            true
        }
    };

    let receipt = json!({
        "command": "receipt verify-content-hash",
        "path": path.to_string_lossy(),
        "mutated": false,
        "valid": valid,
        "preserves_content_hash_present": stated_hash.is_some(),
        "preserves_content_hash_computed": computed,
        "preserves_content_hash_stated": stated_hash,
        "content_hash_matches": valid,
        "input_hash": canonical_json_hash(&receipt),
    });
    print_json(with_receipt_hash(receipt))?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

fn json_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use cairn_core::preserves_content_hash;
    use serde_json::json;

    #[test]
    fn verify_content_hash_positive() {
        let receipt = json!({
            "schema": "cairn.receipt.v1",
            "valid": true,
            "count": 2
        });
        let hash = preserves_content_hash(&receipt).expect("hash");
        let receipt_with_hash = json!({
            "schema": "cairn.receipt.v1",
            "valid": true,
            "count": 2,
            "preserves_content_hash": hash
        });
        let recomputed = preserves_content_hash(&receipt_with_hash).expect("recomputed");
        assert_eq!(hash, recomputed);
    }

    #[test]
    fn verify_content_hash_tampered_fails() {
        let receipt = json!({
            "schema": "cairn.receipt.v1",
            "valid": true,
            "count": 2
        });
        let hash = preserves_content_hash(&receipt).expect("hash");
        // Tamper with the receipt's valid field but keep the stated hash
        let tampered = json!({
            "schema": "cairn.receipt.v1",
            "valid": false,
            "count": 2,
            "preserves_content_hash": hash
        });
        let recomputed = preserves_content_hash(&tampered).expect("recomputed");
        assert_ne!(hash, recomputed);
    }

    #[test]
    fn verify_content_hash_compatibility_without_field() {
        let receipt = json!({
            "schema": "cairn.receipt.v1",
            "valid": true
        });
        // Receipts without preserves_content_hash should still produce a valid hash
        let hash = preserves_content_hash(&receipt).expect("hash");
        assert!(hash.len() == 64);
    }
}
