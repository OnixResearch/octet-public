pub fn helper(input: u32) -> u32 {
    input.saturating_add(1)
}

pub fn call_then_return(input: u32) -> u32 {
    helper(input)
}
