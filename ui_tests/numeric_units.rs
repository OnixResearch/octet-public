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

struct Endpoint {
    // Should NOT warn: a port number is an identifier, not a unit-bearing quantity.
    port: u16,
    // Should NOT warn: a whole `port` component is still an identifier.
    network_port: u16,
}

fn embedded_words() {
    // Should NOT warn: quantity words only count as whole `_`-separated words.
    let proportion = 0.25_f64;
    let report_lines = 40_u32;
    let transport_count = 3_u32;
    let support_count = 2_u32;
    let max_imports = 4_u32;
    let max_exports = 3_u32;
    let _ = (proportion, report_lines, transport_count, support_count, max_imports, max_exports);
}

fn whole_word_quantity() {
    // Should warn: `timeout` is a whole word in the name.
    let connect_timeout = 30_u64;
    let _ = connect_timeout;
}

fn network_quantities() {
    // Should warn: complete quantity components without an admitted unit.
    let request_timeout = 30_u64;
    let buffer_size = 1024_u64;
    let _ = (request_timeout, buffer_size);
}

fn spelled_out_units() {
    // Should NOT warn: spelled-out time, bit, and binary-size units are unit suffixes.
    let timeout_seconds = 30_u64;
    let delay_minutes = 2_u64;
    let interval_mins = 5_u64;
    let latency_hours = 1_u64;
    let retention_limit_days = 7_u64;
    let frame_size_bits = 64_u32;
    let cache_size_kib = 512_u32;
    let buffer_capacity_mib = 16_u32;
    let disk_limit_gib = 100_u32;
    let _ = (
        timeout_seconds,
        delay_minutes,
        interval_mins,
        latency_hours,
        retention_limit_days,
        frame_size_bits,
        cache_size_kib,
        buffer_capacity_mib,
        disk_limit_gib,
    );
}

// Should warn: an async fn parameter is still a raw numeric parameter, reported
// once through the parameter check rather than again as a lowered `let`.
async fn async_parameter(delay: u64) -> u64 {
    delay
}

fn closure_parameters() {
    // Should NOT warn: closure parameters are not checked, and an async
    // closure's parameters are not `let` bindings even though rustc lowers them
    // into `let` statements inside the async body.
    let sync_delay = |delay: u64| delay;
    let async_delay = async |delay: u64| delay;
    let _ = (sync_delay, async_delay);
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
    let endpoint = Endpoint { port: 8080, network_port: 8081 };
    let _ = (endpoint.port, endpoint.network_port);
    embedded_words();
    whole_word_quantity();
    network_quantities();
}
