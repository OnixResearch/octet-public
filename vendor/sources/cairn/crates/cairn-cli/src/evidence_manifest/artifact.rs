use std::{error::Error, fs::File, io::Read, path::Path};

use cairn_core::{blake3_hex, CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1};
use serde_json::{json, Value};

use crate::agent_receipt::{
    read_json_artifact, receipt_stated_or_computed_hash, AGENT_RECEIPT_MAX_BYTES,
};
use crate::support::path_string;

pub(super) const REQUIREMENT_REGISTRY_ROLE: &str = "requirement_registry";
const BYTES_PER_KIBIBYTE: u64 = 1_024;
const KIBIBYTES_PER_MEBIBYTE: u64 = 1_024;
const REQUIREMENT_REGISTRY_MAX_MEBIBYTES: u64 = 64;
const REQUIREMENT_REGISTRY_MAX_BYTES: u64 =
    REQUIREMENT_REGISTRY_MAX_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;

pub(super) fn artifact_read_limit(role: &str) -> u64 {
    if role == REQUIREMENT_REGISTRY_ROLE {
        REQUIREMENT_REGISTRY_MAX_BYTES
    } else {
        AGENT_RECEIPT_MAX_BYTES
    }
}

pub(super) fn read_manifest_artifact(
    root: &Path,
    role: &str,
    path: &Path,
) -> Result<(Value, Value), Box<dyn Error>> {
    if role == REQUIREMENT_REGISTRY_ROLE {
        return read_requirement_registry_artifact(path);
    }
    let value = read_json_artifact(path, artifact_read_limit(role))?;
    let hash = receipt_stated_or_computed_hash(root, &value);
    Ok((value, hash))
}

pub(super) fn manifest_artifact_entry(
    role: &str,
    path: &Path,
    value: &Value,
    hash: Value,
) -> Result<Value, Box<dyn Error>> {
    if role != REQUIREMENT_REGISTRY_ROLE {
        return Ok(json!({
            "role": role,
            "path": path_string(path),
            "hash": hash,
        }));
    }
    let receipt_hash = requirement_registry_receipt_hash(value)
        .ok_or("content-bound requirement registry receipt hash is missing")?;
    Ok(json!({
        "role": role,
        "path": path_string(path),
        "hash": hash,
        "schema": CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1,
        "receipt_hash": receipt_hash,
    }))
}

fn read_requirement_registry_artifact(path: &Path) -> Result<(Value, Value), Box<dyn Error>> {
    let read_limit = REQUIREMENT_REGISTRY_MAX_BYTES.saturating_add(1);
    let mut reader = File::open(path)?.take(read_limit);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let byte_count = u64::try_from(bytes.len())?;
    if byte_count > REQUIREMENT_REGISTRY_MAX_BYTES {
        return Err("content-bound requirement registry exceeds the bounded read limit".into());
    }
    let value = serde_json::from_slice(&bytes)?;
    let hash = Value::String(blake3_hex(&bytes));
    Ok((value, hash))
}

pub(super) fn requirement_registry_receipt_hash(value: &Value) -> Option<&str> {
    value.get("receipt")?.get("receipt_blake3")?.as_str()
}
