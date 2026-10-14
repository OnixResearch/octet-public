pub fn clamp_to_limit(input: u32, limit: u32) -> u32 {
    if input > limit {
        limit
    } else {
        input
    }
}
