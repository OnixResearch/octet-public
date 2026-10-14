#![allow(ambient_clock)]

use std::time::Duration;
use std::time::Instant;

fn contradictory(now_ms: u64) -> u64 {
    // Should warn: takes time param but reads system time
    let _t = Instant::now();
    now_ms + 1000
}

fn pure_time(now_ms: u64, deadline_ms: u64) -> bool {
    // Should NOT warn: no system time read
    now_ms > deadline_ms
}

fn shell_poll() -> Duration {
    // Should NOT warn: no time parameter
    Instant::now().elapsed()
}

fn main() {
    contradictory(1000);
    pure_time(1000, 2000);
    let _ = shell_poll();
}
