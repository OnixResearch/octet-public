//! Domain-separated canonical identity helpers for the bounded merge rail.

use serde::Serialize;

pub const BLAKE3_PREFIX: &str = "b3:";
const FRAME_LENGTH_BYTES: usize = std::mem::size_of::<u64>();
const BLAKE3_HEX_LENGTH: usize = 64;

/// Compute a domain-separated BLAKE3 identity over length-framed byte fields.
#[must_use]
pub fn framed_blake3(domain: &str, fields: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain.as_bytes());
    for field in fields {
        update_frame(&mut hasher, field);
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

/// Serialize a value as canonical JSON bytes.
///
/// # Errors
///
/// Returns an error when the value cannot be serialized as JSON.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| format!("serialize canonical merge-rail value: {error}"))
}

/// Compute a domain-separated BLAKE3 identity over a value's canonical JSON bytes.
///
/// # Errors
///
/// Returns an error when the value cannot be serialized as JSON.
pub fn canonical_identity<T: Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let bytes = canonical_json(value)?;
    Ok(framed_blake3(domain, &[&bytes]))
}

/// Return whether a value is a canonical `b3:` identity.
#[must_use]
pub fn is_identity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(BLAKE3_PREFIX) else {
        return false;
    };
    hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn update_frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    let length = u64::try_from(value.len()).expect("merge-rail identity input length must fit u64");
    let length_bytes = length.to_le_bytes();
    assert_eq!(length_bytes.len(), FRAME_LENGTH_BYTES);
    hasher.update(&length_bytes);
    hasher.update(value);
}
