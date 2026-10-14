//! Opt-in source selector for the production Verus negative proof fixture.
//!
//! This file is deliberately outside `src/verified/` so ordinary positive
//! `cargo octet prove --source src/verified` runs cannot select the invalid
//! proof rail by accident.
/// Marker used only to select the fail-closed production Verus rail.
pub fn production_verus_negative_fixture_marker(value: u64) -> u64 {
    value
}
