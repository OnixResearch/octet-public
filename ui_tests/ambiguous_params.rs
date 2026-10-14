#![warn(ambiguous_params)]

// Should warn: 2 consecutive u64 params
fn compute_deadline(now_ms: u64, timeout_ms: u64) -> u64 {
    now_ms + timeout_ms
}

// Should NOT warn: different types
fn set_value(key: u32, value: u64) -> u64 {
    value + key as u64
}

fn main() {
    compute_deadline(1000, 5000);
    set_value(1, 2);
}
