use std::convert::TryFrom;

fn checked_fallback(x: u64) -> u32 {
    // Should NOT warn: checked conversion with non-extreme fallback
    u32::try_from(x).unwrap_or(0)
}

fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn main() {
    checked_fallback(100);
    bare_narrow(100);
}
