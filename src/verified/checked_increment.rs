//! Bounded-increment source scope for the u64 checked Verus proof rail.
//!
//! This plain Rust function is the executable contract. The Verus proof target
//! models the same contract; the proof does not by itself establish a
//! refinement from the model to this function.

/// Increment one bounded `u64`, returning `None` at `u64::MAX`.
#[must_use]
pub fn increment_checked(value: u64) -> Option<u64> {
    if value < u64::MAX { Some(value + 1) } else { None }
}
