#[cfg(not(test))]
mod prod_only {
    pub(super) fn unwrap_in_cfg_not_test_module_warns() {
        let val: Option<u32> = Some(1);
        let _ = val.unwrap();
    }
}

fn main() {}
