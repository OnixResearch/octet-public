#[cfg(not(test))]
mod prod_only {
    pub(super) fn clock_read_in_cfg_not_test_module_warns() -> std::time::Instant {
        std::time::Instant::now()
    }
}

fn main() {}
