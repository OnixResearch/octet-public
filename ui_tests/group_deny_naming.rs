#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![deny(tigerstyle::naming)]

fn naming_group_is_denied() {
    let ready = true;
    let _ = ready;
}

fn main() {
    naming_group_is_denied();
}
