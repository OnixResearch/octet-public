fn bad_names() {
    // Should warn: bool without predicate prefix
    let active = true;
    let _ = active;

    let ready = false;
    let _ = ready;
}

fn good_names() {
    // Should NOT warn: has predicate prefix
    let is_active = true;
    let _ = is_active;

    let has_data = false;
    let _ = has_data;

    let should_retry = true;
    let _ = should_retry;
}

fn non_bool() {
    // Should NOT warn: not a bool type
    let enabled: u32 = 1;
    let _ = enabled;
}

fn main() {
    bad_names();
    good_names();
    non_bool();
}
