fn asserted_narrow(x: u64) -> u32 {
    // Should NOT warn: earlier same-block assert proves bound
    assert!(x <= u32::MAX as u64);
    let _note = x.saturating_add(1);
    x as u32
}

fn unrelated_assert(x: u64, y: u64) -> u32 {
    // Should warn: assert is about a different value
    assert!(y <= u32::MAX as u64);
    x as u32
}

fn shadowed_assert(x: u64) -> u32 {
    // Should warn: earlier assert was about a different binding with same text
    assert!(x <= u32::MAX as u64);
    let x = x.saturating_add(u32::MAX as u64 + 1);
    x as u32
}

fn wrong_bound_shape(x: u64) -> u32 {
    // Should warn: earlier assert uses the wrong target bound shape
    assert!(x <= u16::MAX as u64);
    x as u32
}

fn later_assert(x: u64) -> u32 {
    // Should warn: later same-block assert cannot justify an earlier cast
    let narrowed = x as u32;
    assert!(x <= u32::MAX as u64);
    narrowed
}

fn main() {
    asserted_narrow(100);
    unrelated_assert(100, 100);
    shadowed_assert(100);
    wrong_bound_shape(100);
    later_assert(100);
}
