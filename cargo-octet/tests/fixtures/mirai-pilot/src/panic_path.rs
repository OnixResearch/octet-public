pub fn require_nonzero(value: u32) -> u32 {
    assert!(value != 0, "zero is outside the selected fixture domain");
    value
}
