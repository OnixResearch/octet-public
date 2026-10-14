#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::safety)]

fn canonical_group_allow_suppresses_safety_lints() {
    let value: Option<u32> = Some(1);
    let _ = value.unwrap();
    panic!("suppressed by safety group");
}

fn main() {
    canonical_group_allow_suppresses_safety_lints();
}
