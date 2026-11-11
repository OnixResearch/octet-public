const SAMPLE_U64: u64 = 100;
const SAMPLE_U32: u32 = 100;

fn bare_narrow(x: u64) -> u32 {
    // Should warn: narrowing without bounds check
    x as u32
}

fn guarded_narrow(x: u64) -> u32 {
    // Should NOT warn: bounded by if guard with MAX
    if x <= u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn unrelated_guard_still_warns(x: u64, y: u64) -> u32 {
    // Should warn: guard proves a different value
    if y <= u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn wrong_bound_still_warns(x: u64) -> u32 {
    // Should warn: guard proves the wrong target bound
    if x <= u16::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn unsafe_then_branch_still_warns(x: u64) -> u32 {
    // Should warn: the cast is in the branch where the bound is false
    if x > u32::MAX as u64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn safe_else_branch_is_allowed(x: u64) -> u32 {
    // Should NOT warn: the else branch implies x <= u32::MAX
    if x > u32::MAX as u64 {
        u32::MIN
    } else {
        x as u32
    }
}

fn try_from_guard_is_allowed(x: u64) -> u32 {
    // Should NOT warn: exact TryFrom guard proves target range
    if u32::try_from(x).is_ok() {
        x as u32
    } else {
        u32::MIN
    }
}

fn unrelated_try_from_guard_still_warns(x: u64, y: u64) -> u32 {
    // Should warn: TryFrom guard proves a different value
    if u32::try_from(y).is_ok() {
        x as u32
    } else {
        u32::MIN
    }
}

fn widening(x: u32) -> u64 {
    // Should NOT warn: widening cast
    x as u64
}

fn signed_upper_bound_only_still_warns(x: i64) -> u32 {
    // Should warn: negative values pass the upper-bound guard and wrap
    if x <= u32::MAX as i64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn signed_else_upper_bound_only_still_warns(x: i64) -> i32 {
    // Should warn: the else branch only proves `x <= i32::MAX`
    if x > i32::MAX as i64 {
        i32::MIN
    } else {
        x as i32
    }
}

fn signed_both_bounds_allowed(x: i64) -> u32 {
    // Should NOT warn: lower and upper bounds are both proven
    if x >= 0 && x <= u32::MAX as i64 {
        x as u32
    } else {
        u32::MIN
    }
}

fn signed_nested_bounds_allowed(x: i64) -> i32 {
    // Should NOT warn: nested guards prove both target bounds
    if x >= i32::MIN as i64 {
        if x <= i32::MAX as i64 { x as i32 } else { i32::MAX }
    } else {
        i32::MIN
    }
}

struct Narrower;

impl Narrower {
    #[allow(clippy::cast_possible_truncation)]
    fn method_allow(&self, x: u64) -> u32 {
        // Should NOT warn: method-level clippy truncation allow
        x as u32
    }
}

trait NarrowDefault {
    #[allow(clippy::cast_possible_truncation)]
    fn narrow_default(&self, x: u64) -> u32 {
        // Should NOT warn: trait-method-level clippy truncation allow
        x as u32
    }
}

impl NarrowDefault for Narrower {}

#[deny(clippy::cast_possible_truncation)]
fn deny_is_not_allow(x: u64) -> u32 {
    // Should warn: `deny` is not an escape hatch
    x as u32
}

fn usize_len_narrow(values: &[u8]) -> u32 {
    // Should warn: `usize` is treated as 64 bits wide
    values.len() as u32
}

fn isize_narrow(x: isize) -> i8 {
    // Should warn: `isize` is treated as 64 bits wide
    x as i8
}

#[allow(platform_dependent_cast)]
fn usize_guarded_narrow(x: usize) -> u32 {
    // Should NOT warn: bounded by if guard with MAX
    if x <= u32::MAX as usize { x as u32 } else { u32::MAX }
}

#[allow(platform_dependent_cast)]
fn isize_both_bounds_allowed(x: isize) -> i8 {
    // Should NOT warn: lower and upper bounds are both proven
    if x >= i8::MIN as isize && x <= i8::MAX as isize { x as i8 } else { i8::MIN }
}

fn usize_to_u64_is_not_narrowing(x: usize) -> u64 {
    // Should NOT warn: no supported target is wider than 64 bits
    x as u64
}

#[allow(platform_dependent_cast)]
fn u64_to_usize_is_left_to_platform_dependent_cast(x: u64) -> usize {
    // Should NOT warn: casts to `usize`/`isize` belong to `platform_dependent_cast`
    x as usize
}

fn main() {
    bare_narrow(SAMPLE_U64);
    guarded_narrow(SAMPLE_U64);
    unrelated_guard_still_warns(SAMPLE_U64, SAMPLE_U64);
    wrong_bound_still_warns(SAMPLE_U64);
    unsafe_then_branch_still_warns(SAMPLE_U64);
    safe_else_branch_is_allowed(SAMPLE_U64);
    try_from_guard_is_allowed(SAMPLE_U64);
    unrelated_try_from_guard_still_warns(SAMPLE_U64, SAMPLE_U64);
    widening(SAMPLE_U32);
    signed_upper_bound_only_still_warns(-1);
    signed_else_upper_bound_only_still_warns(-1);
    signed_both_bounds_allowed(-1);
    signed_nested_bounds_allowed(-1);
    Narrower.method_allow(SAMPLE_U64);
    Narrower.narrow_default(SAMPLE_U64);
    deny_is_not_allow(SAMPLE_U64);
}

fn suggestion_for_plain_return(values: &[u8]) -> u32 {
    // Should warn: `?` is unavailable, so the rewrite needs a placeholder
    values.len() as u32
}

fn suggestion_for_option_return(x: u64) -> Option<u32> {
    // Should warn: an `Option` body propagates the failure with `.ok()?`
    Some(x as u32)
}

fn suggestion_for_compatible_result(x: u64) -> Result<u32, std::num::TryFromIntError> {
    // Should warn: the error type converts from `TryFromIntError`, so `?` compiles
    Ok(x as u32)
}

fn suggestion_for_incompatible_result(x: u64) -> Result<u32, String> {
    // Should warn: the error type needs a `map_err` placeholder
    Ok(x as u32)
}

fn suggestion_for_closure(values: &[u64]) -> Vec<Option<u32>> {
    // Should warn: `?` returns from the closure, which returns `Option`
    values.iter().map(|x| Some(*x as u32)).collect()
}

async fn suggestion_for_async_fn(x: u64) -> Option<u32> {
    // Should warn: `?` returns from the async body, which returns `Option`
    Some(x as u32)
}

const fn suggestion_for_const_fn(x: u64) -> u32 {
    // Should warn without a suggestion: `try_from` is not callable in const contexts
    x as u32
}
