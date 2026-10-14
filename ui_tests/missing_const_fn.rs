#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![warn(tigerstyle::missing_const_fn)]

const SCALE_FACTOR: u64 = 2;

fn scale_value(value: u64) -> u64 {
    value * SCALE_FACTOR
}

fn main() {
    let _scaled = scale_value(SCALE_FACTOR);
}
