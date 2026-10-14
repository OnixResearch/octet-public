use serde::Serialize;

const FRAME_LENGTH_BYTES: usize = std::mem::size_of::<u64>();

#[must_use]
pub fn framed_blake3(domain: &str, fields: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, domain.as_bytes());
    for field in fields {
        update_frame(&mut hasher, field);
    }
    hasher.finalize().to_hex().to_string()
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|error| format!("serialize canonical architecture value: {error}"))
}

pub fn canonical_identity<T: Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let bytes = canonical_json(value)?;
    Ok(framed_blake3(domain, &[&bytes]))
}

fn update_frame(hasher: &mut blake3::Hasher, value: &[u8]) {
    let length = u64::try_from(value.len()).expect("architecture identity input length must fit u64");
    let length_bytes = length.to_le_bytes();
    assert_eq!(length_bytes.len(), FRAME_LENGTH_BYTES);
    hasher.update(&length_bytes);
    hasher.update(value);
}
