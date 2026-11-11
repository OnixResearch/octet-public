use std::{error::Error, fs, path::Path};

use serde_json::Value;

use super::AGENT_RECEIPT_MAX_BYTES;

pub(crate) fn read_json_artifact(path: &Path, max_bytes: u64) -> Result<Value, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > max_bytes {
        return Err("artifact exceeds bounded read limit".into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

pub(crate) fn read_agent_receipt_file(path: &Path) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.agent_receipt_validation_surface.cli]
    let metadata = fs::metadata(path)?;
    if metadata.len() > AGENT_RECEIPT_MAX_BYTES {
        return Err(format!(
            "agent receipt exceeds bounded read limit: {} bytes > {}",
            metadata.len(),
            AGENT_RECEIPT_MAX_BYTES
        )
        .into());
    }
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(crate) fn ensure_inline_receipt_is_bounded(receipt: &Value) -> Result<(), Box<dyn Error>> {
    let bytes = serde_json::to_vec(receipt)?;
    if bytes.len() as u64 > AGENT_RECEIPT_MAX_BYTES {
        return Err(format!(
            "inline agent receipt exceeds bounded read limit: {} bytes > {}",
            bytes.len(),
            AGENT_RECEIPT_MAX_BYTES
        )
        .into());
    }
    Ok(())
}
