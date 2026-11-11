#![allow(dead_code)]
#![deny(magic_number)]

const RECONNECT_TIMEOUT_MS: u64 = 30_000;

fn reconnect() {}
fn work(_value: i32) {}
fn sleep_ms(_delay_ms: u64) {}

fn inline_timeout_threshold(elapsed_ms: u64) {
    if elapsed_ms > 30_000 {
        reconnect();
    }
}

fn named_constant(elapsed_ms: u64) {
    if elapsed_ms > RECONNECT_TIMEOUT_MS {
        reconnect();
    }
}

fn conventional_loop_literals() {
    let mut i = 0;
    i += 1;
    if i > -1 {
        work(i);
    }
}

fn unit_bearing_binding() {
    let retry_delay_ms = 250;
    sleep_ms(retry_delay_ms);
}

fn embedded_domain_words(report_lines: u32, transport_count: u32) {
    // Should NOT warn: `port` inside `report`/`transport` is not a policy name.
    if report_lines > 40 || transport_count > 3 {
        reconnect();
    }
}

fn whole_word_domain_name(retry_attempts: u32) {
    if retry_attempts > 5 {
        reconnect();
    }
}

struct ReconnectState {
    elapsed_ms: u64,
    retry_attempts: u32,
    attempts: u32,
}

impl ReconnectState {
    fn retry_limit(&self) -> u32 {
        self.retry_attempts
    }
}

fn field_domain_names(state: &ReconnectState) {
    // Should warn: a field operand names the policy just like a local binding.
    if state.elapsed_ms > 30_000 {
        reconnect();
    }
    // Should warn: a field operand with a domain word.
    if state.retry_attempts > 5 {
        reconnect();
    }
    // Should warn: a method operand with a domain word.
    if state.retry_limit() > 7 {
        reconnect();
    }
    // Should NOT warn: neither the field nor the method names a policy.
    if state.attempts > 3 || report_count(state) > 40 {
        reconnect();
    }
}

fn report_count(state: &ReconnectState) -> u32 {
    state.attempts
}

fn unit_binding_around_threshold(elapsed_ms: u64) -> u64 {
    // Should warn on `30_000`: the binding names the selected delay, not the
    // threshold nested in its initializer.
    let retry_delay_ms = if elapsed_ms > 30_000 { 50 } else { 10 };
    retry_delay_ms
}

#[test]
fn test_context_suppressed() {
    assert_eq!(30_000, 30_000);
}

fn main() {}