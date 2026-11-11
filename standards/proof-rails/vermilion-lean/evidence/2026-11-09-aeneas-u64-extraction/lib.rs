pub fn increment_checked(value: u64) -> Option<u64> {
    if value < u64::MAX { Some(value + 1) } else { None }
}
