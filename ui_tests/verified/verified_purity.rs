use std::time::Duration;

pub async fn impure_async() {
    // Should error: async fn in verified module
}

fn pure_duration() -> Duration {
    // Should NOT error: Duration is a pure value type
    Duration::from_millis(500)
}

fn main() {
    let _ = pure_duration();
}
