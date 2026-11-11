#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

fn will_panic() {
    // Should warn: panic! in production code
    panic!("something went wrong");
}

fn will_unimplemented() {
    // Should warn: unimplemented!
    unimplemented!("not done");
}

fn zero_arg_panic() {
    // Should warn: zero-argument panic! expands to a plain call, not a block
    panic!();
}

fn zero_arg_unreachable() {
    // Should warn: zero-argument unreachable!
    unreachable!();
}

fn zero_arg_unimplemented() {
    // Should warn: zero-argument unimplemented!
    unimplemented!();
}

fn path_qualified_std_panic() {
    // Should warn: path-qualified panic!
    std::panic!("qualified panic");
}

fn path_qualified_core_unreachable() {
    // Should warn: path-qualified unreachable!
    core::unreachable!("qualified unreachable");
}

fn assert_is_not_a_panic_macro(value: u32) {
    // Should NOT warn: assert! expands to panic! internally
    assert!(value > 0, "value must be positive");
}

macro_rules! local_panic_wrapper {
    () => {
        panic!("wrapped")
    };
}

fn user_macro_wrapping_panic_is_not_reported() {
    // Should NOT warn: the outermost invocation is the user macro, unchanged behavior
    local_panic_wrapper!();
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

struct CfgTestMethodHost;

impl CfgTestMethodHost {
    #[cfg(test)]
    fn panic_in_cfg_test_impl_method_is_allowed() {
        panic!("cfg-test-only method panic");
    }
}

trait CfgTestTraitHost {
    #[cfg(test)]
    fn panic_in_cfg_test_trait_method_is_allowed() {
        panic!("cfg-test-only method panic");
    }
}
