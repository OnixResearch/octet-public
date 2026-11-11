//! Codec fixture with one unproven Tiger Style allowance and one anchored
//! proof reference site.
//!
//! `narrow_offset` performs a checked narrowing: the assertion proves the
//! `u64` value fits in `u16` before the cast. The allowance records that the
//! cast itself is a suppression rather than a checked conversion. A proof-ref
//! anchor placed beside the function links the suppression to an existing
//! proof card; `cargo octet prove --candidates` uses that linkage to decide
//! whether the allowance still needs a proof.
//!
//! This fixture is deliberately tiny. It exercises candidate discovery,
//! ranking input, and the drop-off rule; it is not a consumer codec.
//!
//! The maximal value is documented once so the assertion and the mutant stay
//! easy to compare:
//!
//! - checked variant: asserts the bound before the cast;
//! - mutant variant: removes the assertion, keeping the cast.
//!
//! Neither variant is a formal proof of the codec. The proof-ref only links
//! this source site to a saved proof card, and the negative check decides
//! whether that rail is about the intended property.



















#[allow(tigerstyle::unchecked_narrowing)]
// octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0
pub fn narrow_offset(offset: u64) -> u16 {
    assert!(offset <= u64::from(u16::MAX), "offset exceeds the u16 range");
    offset as u16
}
