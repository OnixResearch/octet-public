fn min_bounded(x: u64) -> u32 {
    // Should NOT warn: bounded by target max before cast
    x.min(u32::MAX as u64) as u32
}

fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn signed_min_only(x: i64) -> u32 {
    // Should warn: negative values pass `.min(u32::MAX as i64)` and wrap
    x.min(u32::MAX as i64) as u32
}

fn signed_min_then_max(x: i64) -> u32 {
    // Should NOT warn: clamped to both target bounds before cast
    x.min(u32::MAX as i64).max(0) as u32
}

fn signed_max_then_min(x: i64) -> i32 {
    // Should NOT warn: clamped to both target bounds before cast
    x.max(i32::MIN as i64).min(i32::MAX as i64) as i32
}

#[allow(platform_dependent_cast)]
fn usize_min_bounded(x: usize) -> u16 {
    // Should NOT warn: bounded by target max before cast
    x.min(u16::MAX as usize) as u16
}

fn main() {
    min_bounded(100);
    bare_narrow(100);
    signed_min_only(-1);
    signed_min_then_max(-1);
    signed_max_then_min(-1);
}
