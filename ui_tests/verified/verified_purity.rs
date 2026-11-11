#![allow(non_trait_imports, path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

use std::time::Duration;
// Should error: importing a wall-clock type is itself the violation.
use std::time::Instant;
// Should error: importing an I/O type is itself the violation.
use std::fs::File;
// Should error: nested import groups are checked per imported path.
use std::{fs, time::SystemTime};

pub async fn impure_async() {
    // Should error: async fn in verified module
}

fn pure_duration() -> Duration {
    // Should NOT error: Duration is a pure value type
    Duration::from_millis(500)
}

fn wall_clock() -> Instant {
    Instant::now()
}

fn open_config() -> Option<File> {
    File::open("config.toml").ok()
}

struct Ledger;

impl Ledger {
    // Should error: async inherent method in verified module
    async fn sync_remote(&self) {}
}

trait Store {
    // Should error: async trait method in verified module
    async fn load(&self) -> u64;
}

fn spawn_work() -> impl Future<Output = u64> {
    // Should error: async block in verified module
    async move { 1 }
}

mod module_import {
    // Should NOT error by itself: the `time` module also holds pure `Duration`.
    use std::time;

    pub fn pure_window() -> time::Duration {
        // Should NOT error: Duration is a pure value type
        time::Duration::from_secs(1)
    }

    pub fn wall_clock_through_module() -> time::Instant {
        // Should error: the module import still reaches the wall clock
        time::Instant::now()
    }
}

mod glob_import {
    // Should NOT error by itself: the glob also brings in pure `Duration`.
    use std::time::*;

    pub fn pure_window() -> Duration {
        // Should NOT error: Duration is a pure value type
        Duration::from_millis(5)
    }

    pub fn wall_clock_through_glob() -> SystemTime {
        // Should error: the glob import still reaches the wall clock
        SystemTime::now()
    }
}

// Should NOT error: this fixture compiles with `--cfg test`, and test-only code
// may use I/O, async, and the wall clock for setup.
#[cfg(test)]
mod tests {
    use std::fs;

    async fn async_helper() {}

    fn read_fixture() -> Vec<u8> {
        let _started = std::time::Instant::now();
        fs::read("fixture.bin").unwrap_or_default()
    }
}

fn main() {
    let _ = pure_duration();
    let _ = wall_clock();
    let _ = open_config();
}
