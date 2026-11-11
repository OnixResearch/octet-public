#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

// Compiled with `--test`: rustc consumes `#[test]` and registers each test
// through a same-named sibling const, so the function itself has no test
// attribute in HIR.

fn helper_outside_tests() -> u32 {
    // Should warn: a test harness crate still contains non-test helpers
    let val: Option<u32> = Some(1);
    val.expect("helper")
}

#[test]
fn expect_in_harness_test_is_allowed() {
    let val: Option<u32> = Some(1);
    let _ = val.expect("test");
    let _ = helper_outside_tests();
}

#[test]
fn unwrap_in_fn_nested_in_harness_test_is_allowed() {
    fn nested() -> u32 {
        let val: Option<u32> = Some(1);
        val.unwrap()
    }
    let _ = nested();
}

fn production_entry() -> u32 {
    fn expect_in_harness_test_is_allowed() -> u32 {
        // Should warn: shares only its name with a test registered elsewhere
        let val: Result<u32, &str> = Ok(1);
        val.expect("nested production")
    }
    expect_in_harness_test_is_allowed()
}

mod production {
    pub(super) fn expect_in_harness_test_is_allowed() -> u32 {
        // Should warn: shares only its name with a test in another module
        let val: Result<u32, &str> = Ok(1);
        val.expect("production")
    }
}
