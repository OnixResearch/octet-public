#![warn(numeric_units)]

const TYPED_DURATION_MS: u64 = 100;
const SCHEDULE_DELAY_MS: u64 = 10;

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

struct RawLimits {
    // Should warn: raw numeric fields need units.
    timeout: u64,
}

// Should warn: raw numeric parameters need units.
fn schedule(delay: u64) -> u64 {
    delay
}

struct Milliseconds(u64);

struct TypedLimits {
    // Should NOT warn: the nominal type carries the unit.
    timeout: Milliseconds,
}

fn typed_local() {
    // Should NOT warn: the nominal type carries the unit.
    let duration = Milliseconds(TYPED_DURATION_MS);
    let limits = TypedLimits { timeout: duration };
    let _ = limits.timeout.0;
}

fn main() {
    bare_timeout();
    suffixed_timeout();
    loop_counter();
    bare_latency();
    let raw = RawLimits {
        timeout: schedule(SCHEDULE_DELAY_MS),
    };
    let _ = raw.timeout;
    typed_local();
}
