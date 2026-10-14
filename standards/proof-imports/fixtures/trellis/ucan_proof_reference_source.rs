//! Imported Trellis proof-reference source fixture for Octet proof-card validation.
//! Original path: ../trellis/src/ucan_proof_reference.rs
//! This fixture preserves the digest-admission contract shape without acting as proof evidence.

pub const BLAKE3_DIGEST_LEN: usize = 32;

pub fn proof_reference_digest_bytes_valid(bytes: &[u8]) -> bool {
    bytes.len() == BLAKE3_DIGEST_LEN
}
