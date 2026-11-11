use std::cell::Cell;

const fn scalar(value: u32) -> u32 {
    value
}
const fn length(value: usize) -> usize {
    value
}
const fn closure_scalar(value: u32) -> u32 {
    value
}

pub const NAMED: u32 = scalar(1);
pub static STATIC: u32 = scalar(2);

pub trait WithConst {
    const VALUE: u32 = scalar(3);
}

pub struct Owner;
impl WithConst for Owner {
    const VALUE: u32 = scalar(4);
}
impl Owner {
    pub const INHERENT: u32 = scalar(5);
    pub fn method(&self) -> u32 {
        scalar(6)
    }
}

pub fn inline_value() -> u32 {
    const { scalar(7) }
}
pub fn anonymous_length() -> [u8; length(8)] {
    [0; length(8)]
}

macro_rules! expanded_constant {
    () => {
        const { scalar(9) }
    };
}
pub fn macro_value() -> u32 {
    expanded_constant!()
}

pub fn constant_with_nested_body() -> u32 {
    const {
        fn nested_file_effect() {
            let _ = std::fs::read("body-owner-fixture-never-executed");
        }
        scalar(10)
    }
}

std::thread_local! {
    static LOCAL: Cell<u32> = const { Cell::new(0) };
}
pub fn hidden_state(value: u32) {
    LOCAL.with(|cell| cell.set(value));
}
pub fn closure_value() -> u32 {
    let value = || closure_scalar(11);
    value()
}
