fn min_bounded(x: u64) -> u32 {
    // Should NOT warn: bounded by target max before cast
    x.min(u32::MAX as u64) as u32
}

fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn main() {
    min_bounded(100);
    bare_narrow(100);
}
