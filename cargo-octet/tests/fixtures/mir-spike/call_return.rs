pub fn callee(value: u32) -> u32 {
    value.saturating_add(1)
}

pub fn call_then_return(input: u32) -> u32 {
    let next = callee(input);
    next
}
