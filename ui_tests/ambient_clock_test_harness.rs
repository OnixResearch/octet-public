#![allow(non_trait_imports, path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

// Compiled with `--test`: rustc consumes `#[test]` and registers each test
// through a same-named sibling const, so the function itself has no test
// attribute in HIR.
use std::time::Instant;

trait Restamp {
    fn now(&self) -> Instant;
}

impl Restamp for Instant {
    fn now(&self) -> Instant {
        *self
    }
}

fn restamped_start() -> Instant {
    // Should warn: a test harness crate still contains non-test helpers
    let start = Instant::now();
    // Should warn: method-call shape in the same helper
    start.now()
}

#[test]
fn clock_reads_in_harness_test_are_allowed() {
    let start = Instant::now();
    let _ = start.now();
    let _ = restamped_start();
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn clock_read_in_cfg_test_module_is_allowed() -> std::time::Instant {
        std::time::Instant::now()
    }
}
