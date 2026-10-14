pub fn checked(value: u32, delta: u32) -> Option<u32> {
    value.checked_add(delta)
}
