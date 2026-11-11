#![warn(raw_arithmetic_overflow)]

fn unprotected_add(a: u64, b: u64) -> u64 {
    // Should warn: raw + on unsigned
    a + b
}

fn unprotected_mul(a: u64, b: u64) -> u64 {
    // Should warn: raw * on unsigned
    a * b
}

fn constant_arithmetic() -> u64 {
    // Should NOT warn: both operands are constants
    1024 * 1024
}

fn checked_add(a: u64, b: u64) -> Option<u64> {
    // Should NOT warn: using checked arithmetic
    a.checked_add(b)
}

#[cfg(test)]
fn test_only_add(a: u64, b: u64) -> u64 {
    // Should NOT warn: test-only code is outside production scope
    a + b
}

fn unsuffixed_literal_lhs(n: u64) -> u64 {
    // Should warn: the suggestion must name the type; `(2).saturating_mul(n)` is E0689
    2 * n
}

fn compound_add(start: u64, x: &u64) -> u64 {
    // Should warn: raw += on unsigned
    let mut t = start;
    t += *x;
    t
}

fn compound_sub(a: &mut u32, b: u32) {
    // Should warn: raw -= on unsigned
    *a -= b;
}

struct Counter {
    count: usize,
}

fn compound_mul(counter: &mut Counter, factor: usize) {
    // Should warn: raw *= on unsigned
    counter.count *= factor;
}

fn compound_indexed_add(values: &mut [u64], b: u64) {
    // Should warn without a suggestion: rewriting would evaluate the index twice
    values[0] += b;
}

fn compound_signed_add(a: &mut i64, b: i64) {
    // Should NOT warn: signed arithmetic
    *a += b;
}

#[cfg(test)]
fn test_only_compound_add(a: &mut u64, b: u64) {
    // Should NOT warn: test-only code is outside production scope
    *a += b;
}

fn main() {
    unprotected_add(1, 2);
    unprotected_mul(1, 2);
    constant_arithmetic();
    checked_add(1, 2);
}
