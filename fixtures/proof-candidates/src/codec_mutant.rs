//! Negative-check mutant for `src/codec.rs`.
//!
//! The bound assertion is removed while the narrowing cast remains. A rail
//! that still passes against this variant is not demonstrating the narrowing
//! property, so `cargo octet prove --record-negative-check` reports
//! `property_not_demonstrated`.

pub fn narrow_offset(offset: u64) -> u16 {
    offset as u16
}
