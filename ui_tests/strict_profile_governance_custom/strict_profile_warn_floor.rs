#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(no_unwrap)]
#![warn(function_length)]

// Under a strict profile a source `warn` lowers a floor lint from `deny`, so
// it needs the same local justification as an `allow`. The crate-level
// `warn(no_unwrap)` above has none and should be reported; raising the
// non-floor `function_length` to `warn` should not be.

// Should NOT be reported: a justified `warn` of a floor lint.
// Bounded fixture input: callers always pass a non-empty slice.
#[warn(tigerstyle::no_unwrap)]
fn justified(values: &[u32]) -> u32 {
    *values.first().unwrap()
}

// Should be reported: the canonical name of a floor lint, unjustified.

#[warn(tigerstyle::no_panic)]
fn lowered() {}

fn main() {
    let _ = justified(&[1]);
    lowered();
}

// Should NOT be reported: the later `deny` raises `no_unwrap` back.

#[warn(no_unwrap)]
#[deny(tigerstyle::no_unwrap)]
mod restored {}
