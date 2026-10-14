#![warn(unjustified_allow)]

#[allow(clippy::needless_return)]
fn bare_allow() -> u32 {
    return 1;
}

// Should NOT warn: has justification comment
// Verified casts: Verus checks every narrowing in this module.
#[allow(clippy::cast_possible_truncation)]
fn justified_allow(x: u64) -> u32 {
    x as u32
}

fn main() {
    bare_allow();
    justified_allow(1);
}
