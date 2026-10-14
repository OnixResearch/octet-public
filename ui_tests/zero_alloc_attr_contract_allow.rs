// zero_alloc_attr_contract_allow: non-allocating or explicitly-allowed functions

#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(unused)]

// These should NOT trigger the lint
#[tigerstyle::zero_alloc]
fn stack_locals() -> u64 {
    let a = 1u64;
    let b = 2u64;
    a + b
}

#[tigerstyle::zero_alloc]
fn array_init() -> [u8; 4] {
    [1, 2, 3, 4]
}

#[allow(tigerstyle::zero_alloc_attr_contract, reason = "shell boundary allocation")]
fn allows_with_justification() {
    let _v: Vec<u8> = Vec::new();
}

fn main() {
    stack_locals();
    array_init();
    allows_with_justification();
}