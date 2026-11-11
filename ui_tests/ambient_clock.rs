#![allow(non_trait_imports, renamed_imports, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, SystemTimeError};

fn deadline_from(timeout: Duration) -> Instant {
    Instant::now() + timeout
}

mod renamed {
    use std::time::Instant as Clock;

    pub fn started() -> Clock {
        // Should warn: the renamed import still resolves to `Instant::now`
        Clock::now()
    }
}

type WallClock = std::time::SystemTime;

fn wall_clock() -> WallClock {
    // Should warn: the type alias still resolves to `SystemTime::now`
    WallClock::now()
}

struct LogicalClock;

impl LogicalClock {
    fn now() -> u64 {
        7
    }

    fn elapsed(&self) -> u64 {
        0
    }
}

fn logical_now() -> u64 {
    // Should NOT warn: a local `now` function is not a wall-clock read
    LogicalClock::now()
}

fn first_seen(seen: &mut HashMap<u32, Instant>, key: u32) -> Instant {
    // Should warn: `Instant::now` passed as a function value reads the clock when called
    *seen.entry(key).or_insert_with(Instant::now)
}

fn wall_clock_source() -> fn() -> SystemTime {
    // Should warn: the function value still resolves to `SystemTime::now`
    SystemTime::now
}

fn logical_source() -> fn() -> u64 {
    // Should NOT warn: a local `now` function value is not a wall-clock read
    LogicalClock::now
}

fn age(started: Instant) -> Duration {
    // Should warn: `Instant::elapsed` reads the clock
    started.elapsed()
}

fn age_by_path(started: Instant) -> Duration {
    // Should warn: the path form of `Instant::elapsed` reads the clock
    Instant::elapsed(&started)
}

fn wall_age(started: SystemTime) -> Result<Duration, SystemTimeError> {
    // Should warn: `SystemTime::elapsed` reads the clock
    started.elapsed()
}

fn logical_age(clock: &LogicalClock) -> u64 {
    // Should NOT warn: a local `elapsed` method is not a wall-clock read
    clock.elapsed()
}

fn span_between(start: Instant, end: Instant) -> Duration {
    // Should NOT warn: comparing injected instants does not read the clock
    end.duration_since(start)
}

fn main() {
    let _deadline = deadline_from(Duration::from_secs(1));
    let _ = renamed::started();
    let _ = wall_clock();
    let _ = logical_now();
}
