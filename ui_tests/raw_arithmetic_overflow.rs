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

fn main() {
    unprotected_add(1, 2);
    unprotected_mul(1, 2);
    constant_arithmetic();
    checked_add(1, 2);
}
