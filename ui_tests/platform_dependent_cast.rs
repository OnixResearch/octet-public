#![allow(non_trait_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::error::Error;
use std::num::TryFromIntError;

fn fixed_to_usize(x: u32) -> usize {
    // Should warn: as usize from fixed-width type. The body cannot use `?`,
    // so the suggestion is a placeholder.
    x as usize
}

fn usize_to_usize(x: usize) -> usize {
    // Should NOT warn: already platform-sized
    x
}

fn widening_to_u64(x: u32) -> u64 {
    // Should NOT warn: not casting to usize/isize
    x as u64
}

fn checked_result(x: u64) -> Result<usize, TryFromIntError> {
    // Should warn and suggest `?`: the error type is `TryFromIntError`.
    Ok(x as usize)
}

fn boxed_error_result(x: i64) -> Result<isize, Box<dyn Error>> {
    // Should warn and suggest `?`: `Box<dyn Error>: From<TryFromIntError>`.
    Ok(x as isize)
}

fn string_error_result(x: u64) -> Result<usize, String> {
    // Should warn with a placeholder `map_err`: `String` does not convert from
    // `TryFromIntError`.
    Ok(x as usize)
}

fn optional(x: u32) -> Option<usize> {
    // Should warn and suggest `.ok()?`.
    Some(x as usize)
}

fn closure_optional(values: &[u32]) -> Vec<Option<usize>> {
    // Should warn and suggest `.ok()?`: `?` returns from the closure.
    values.iter().map(|value| -> Option<usize> { Some(*value as usize) }).collect()
}

fn lossless_widen(x: u16, y: i8) -> (usize, isize) {
    // Should warn and suggest the lossless `From` conversion.
    (x as usize, y as isize)
}

// Should warn without a suggestion: conversions are not callable in const context.
const CONST_LEN: usize = 4u32 as usize;

macro_rules! widen_index {
    ($value:expr) => {
        // Should warn without a suggestion: rewriting the macro body would
        // splice one call site's operand into every expansion.
        $value as usize
    };
}

macro_rules! small_value {
    () => {
        7u16
    };
}

fn macro_casts(x: u8) -> (usize, usize) {
    // Should warn without a suggestion: the operand's text is the macro body,
    // not the `small_value!()` call written here.
    (widen_index!(x), small_value!() as usize)
}

fn main() {
    fixed_to_usize(42);
    usize_to_usize(42);
    widening_to_u64(42);
    let results = [checked_result(42).is_ok(), boxed_error_result(42).is_ok(), string_error_result(42).is_ok()];
    let _ = results;
    let _ = optional(42);
    let _ = closure_optional(&[1, 2]);
    let _ = lossless_widen(1, 2);
    let _ = CONST_LEN;
    let _ = macro_casts(1);
}
