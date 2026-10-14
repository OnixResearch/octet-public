#![warn(numeric_units)]

fn bare_timeout() {
    // Should warn: quantity word without unit suffix
    let timeout = 5000;
    let _ = timeout;
}

fn suffixed_timeout() {
    // Should NOT warn: has unit suffix
    let timeout_ms: u64 = 5000;
    let _ = timeout_ms;
}

fn loop_counter() {
    // Should NOT warn: common loop variable
    let count = 10;
    let _ = count;
}

fn bare_latency() {
    // Should warn: quantity word without suffix
    let latency = 100;
    let _ = latency;
}

fn main() {
    bare_timeout();
    suffixed_timeout();
    loop_counter();
    bare_latency();
}
