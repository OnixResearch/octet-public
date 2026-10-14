fn helper(value: u32) -> u32 {
    value.saturating_add(1)
}

pub fn root(value: u32) -> u32 {
    helper(value)
}
