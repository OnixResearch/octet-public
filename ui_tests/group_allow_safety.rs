#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::safety)]

fn safety_group_is_suppressed() {
    let value: Option<u32> = Some(1);
    let _ = value.unwrap();
}

fn naming_group_still_warns() {
    let ready = true;
    let _ = ready;
}

fn main() {
    safety_group_is_suppressed();
    naming_group_still_warns();
}
