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

fn guarded_option_reference(value: &Option<u32>) {
    if value.is_some() {
        let _ = value.unwrap();
    }
}

fn guarded_result_reference(value: &Result<u32, &'static str>) {
    if value.is_ok() {
        let _ = value.unwrap();
    }
}

fn unguarded_option_reference_still_warns(value: &Option<u32>) {
    if value.is_none() {
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

fn guarded_ufcs_option(value: Option<u32>) {
    // Should NOT warn: same guard as the method form
    if value.is_some() {
        let _ = Option::unwrap(value);
    }
}

fn unguarded_ufcs_option_still_warns(value: Option<u32>) {
    if value.is_none() {
        let _ = Option::unwrap(value);
    }
}
