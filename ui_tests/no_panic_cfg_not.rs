#[cfg(not(test))]
mod prod_only {
    pub(super) fn panic_in_cfg_not_test_module_warns() {
        panic!("cfg-not-test panic");
    }
}

fn main() {}
