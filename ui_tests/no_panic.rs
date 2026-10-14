fn will_panic() {
    // Should warn: panic! in production code
    panic!("something went wrong");
}

fn will_unimplemented() {
    // Should warn: unimplemented!
    unimplemented!("not done");
}

fn main() {
    // Don't actually call the panicking functions
}

#[test]
fn test_only_panic_is_allowed() {
    panic!("test-only panic");
}

#[cfg(test)]
mod cfg_test_only {
    pub(super) fn panic_in_cfg_test_module_is_allowed() {
        panic!("cfg-test-only panic");
    }
}
