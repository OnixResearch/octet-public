fn guarded_option(value: Option<u32>) {
    if value.is_some() {
        let _ = value.unwrap();
    }
}

fn guarded_result(value: Result<u32, &'static str>) {
    if value.is_ok() {
        let _ = value.unwrap();
    }
}

fn guarded_expect_still_warns(value: Option<u32>) {
    if value.is_some() {
        let _ = value.expect("guarded expect should still warn");
    }
}

fn else_branch_still_warns(value: Option<u32>) {
    if value.is_some() {
        return;
    } else {
        let _ = value.unwrap();
    }
}

fn main() {}
