//! Bounded-increment pilot subject for the Vermilion provider evaluation.
//!
//! The executable contract is the common semantic contract at
//! `standards/proof-rails/vermilion-lean/pilot/semantic-spec.txt`:
//!
//! - precondition: `value < u64::MAX`
//! - postcondition: `result == Some(value + 1)`
//! - overflow behavior: checked; `None` when the precondition is false
//! - termination: one call, no loops
//!
//! This file is a review subject, not verified code. No Vermilion, Lean,
//! Aeneas, or Verus proof run is claimed for it.

/// Increment one bounded `u64`, returning `None` at `u64::MAX`.
#[must_use]
pub fn increment_checked(value: u64) -> Option<u64> {
    if value < u64::MAX { Some(value + 1) } else { None }
}
