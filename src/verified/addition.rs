/// Tiny verified-module fixture covered by the Verus proof rail.
///
/// octet-formal-proof-claim: `checked_add_one` preserves the addition-by-one
/// postcondition for the fixture and production pilot rails covered by
/// `verus/proofs/addition_bound.rs`.
pub fn checked_add_one(value: u64) -> u64 {
    value + 1
}
