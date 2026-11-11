#![feature(register_tool)]
#![register_tool(tigerstyle)]
// justified crate-wide stubs while API lands
#![allow(tigerstyle::no_todo)]

fn crate_allow_covers_stub() {
    todo!("allowed crate-level stub");
}

fn crate_allow_still_rejects_panic() {
    panic!("panic should still warn");
}

fn main() {
    crate_allow_covers_stub();
    crate_allow_still_rejects_panic();
}
