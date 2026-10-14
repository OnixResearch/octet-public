#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::no_unwrap)]

fn canonical_lint_allow_suppresses_no_unwrap() {
    let value: Option<u32> = Some(1);
    let _ = value.unwrap();
}

fn main() {
    canonical_lint_allow_suppresses_no_unwrap();
}
