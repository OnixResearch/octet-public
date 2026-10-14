pub fn divide_checked(numerator: u32, denominator: u32) -> Option<u32> {
    if denominator == 0 {
        return None;
    }
    Some(numerator / denominator)
}
