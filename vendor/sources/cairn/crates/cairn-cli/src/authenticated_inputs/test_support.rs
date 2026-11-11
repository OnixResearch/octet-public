use std::error::Error;
use std::fs;
use std::path::Path;

use cairn_core::MeasuredEvidenceInput;
use serde_json::Value;

use super::{HEX_ALPHABET_LENGTH, HEX_PAIR_BYTES, LOW_NIBBLE_MASK, NIBBLE_BITS};

pub(super) fn write_manifest(path: &Path, value: &Value) -> Result<(), Box<dyn Error>> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

pub(super) fn measured(path: &Path, role: &str) -> Result<MeasuredEvidenceInput, Box<dyn Error>> {
    let digest = file_hash(path)?;
    Ok(MeasuredEvidenceInput {
        role: role.to_string(),
        path: path.display().to_string(),
        declared_blake3: digest.clone(),
        measured_blake3: digest,
    })
}

pub(super) fn file_hash(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(blake3::hash(&fs::read(path)?).to_hex().to_string())
}

pub(super) fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; HEX_ALPHABET_LENGTH] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * HEX_PAIR_BYTES);
    for byte in bytes {
        encoded.push(HEX[usize::from(byte >> NIBBLE_BITS)] as char);
        encoded.push(HEX[usize::from(byte & LOW_NIBBLE_MASK)] as char);
    }
    encoded
}
