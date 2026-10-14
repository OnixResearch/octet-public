pub fn choose_branch(input: u32, limit: u32) -> u32 {
    if input > limit {
        limit
    } else {
        input
    }
}
