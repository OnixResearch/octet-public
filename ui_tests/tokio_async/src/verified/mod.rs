// Should NOT error by itself: the `time` module also holds pure `Duration`.
use std::time;

pub fn pure_window() -> time::Duration {
    // Should NOT error: Duration is a pure value type
    time::Duration::from_secs(1)
}

pub fn wall_clock_through_module() -> time::Instant {
    // Should error: the module import reaches the wall clock although the
    // `mod verified;` declaration sits in a crate root outside `verified/`.
    time::Instant::now()
}

pub fn wall_clock_through_local_glob() -> u64 {
    // Should NOT error by itself: the glob also brings in pure `Duration`.
    use std::time::*;

    // Should error: a function-local glob still reaches the wall clock.
    let _started = SystemTime::now();
    Duration::from_millis(5).as_secs()
}

mod stand_in {
    pub struct SystemTime;

    impl SystemTime {
        pub fn now() -> Self {
            Self
        }
    }
}

pub fn local_glob_does_not_leak() {
    use stand_in::SystemTime;

    // Should NOT error: the glob above is local to another function.
    let _ = SystemTime::now();
}

#[test]
fn harness_test_reads_clock() {
    // Should NOT error: `#[test]` functions may use I/O and the wall clock for setup.
    let _started = std::time::Instant::now();
    let _contents = std::fs::read("fixture.bin");
}

#[tokio::test]
async fn tokio_harness_test_reads_clock() {
    // Should NOT error: `#[tokio::test]` expands to `#[test]`.
    let _started = time::Instant::now();
}

fn helper_beside_tests() -> time::Instant {
    // Should error: helpers next to test functions stay checked.
    time::Instant::now()
}
