use std::time::{Duration, Instant};

fn deadline_from(timeout: Duration) -> Instant {
    Instant::now() + timeout
}

fn main() {
    let _deadline = deadline_from(Duration::from_secs(1));
}
