#![warn(ambiguous_params)]

// Should warn: 2 consecutive u64 params
fn compute_deadline(now_ms: u64, timeout_ms: u64) -> u64 {
    now_ms + timeout_ms
}

// Should warn: repeated owned boundary values are also swappable.
fn copy_session(source: String, destination: String) -> String {
    let _ = source;
    destination
}

// Should warn: separation by another type does not remove the category risk.
fn copy_session_with_attempts(source: String, attempts: u32, destination: String) -> String {
    let _ = (source, attempts);
    destination
}

struct Transport;

impl Transport {
    // Should warn: inherent methods are checked after the receiver.
    fn connect(&self, source: String, destination: String) {
        let _ = (source, destination);
    }
}

trait Router {
    // Should warn: trait declarations define the caller-facing contract.
    fn route(&self, source: String, destination: String);
}

struct SourceId(String);
struct DestinationId(String);

// Should NOT warn: distinct nominal domains cannot be swapped.
fn copy_typed(source: SourceId, destination: DestinationId) {
    let _ = (source.0, destination.0);
}

// Should NOT warn: different primitive types.
fn set_value(key: u32, value: u64) -> u64 {
    value + key as u64
}

fn main() {
    compute_deadline(1000, 5000);
    let _ = copy_session(String::new(), String::new());
    let _ = copy_session_with_attempts(String::new(), 1, String::new());
    Transport.connect(String::new(), String::new());
    copy_typed(SourceId(String::new()), DestinationId(String::new()));
    set_value(1, 2);
}
