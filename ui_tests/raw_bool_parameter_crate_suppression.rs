#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::raw_bool_parameter)]

pub fn wrongly_exempted(enabled: bool) {
    let _ = enabled;
}

fn main() {
    wrongly_exempted(true);
}
