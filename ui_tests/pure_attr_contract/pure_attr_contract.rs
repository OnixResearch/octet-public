#![feature(register_tool)]
#![register_tool(tigerstyle)]

mod rand {
    pub fn random() -> u64 {
        0
    }
}

#[tigerstyle::pure]
fn pure_reads_file() {
    let _bytes = std::fs::read("state.bin");
}

#[tigerstyle::pure]
fn pure_reads_env() {
    let _home = std::env::var("HOME");
}

#[tigerstyle::pure]
fn pure_random() -> u64 {
    rand::random()
}

#[tigerstyle::pure]
fn pure_logs(value: u64) {
    println!("{value}");
}

#[tigerstyle::pure]
fn pure_spawns() {
    let _handle = std::thread::spawn(|| {});
}

#[tigerstyle::pure]
fn pure_global_state() {
    static mut VALUE: u64 = 0;
    let _value = core::ptr::addr_of!(VALUE);
}

#[tigerstyle::pure]
fn pure_interior_mutability() {
    let _cell = std::cell::RefCell::new(0);
}

#[tigerstyle::pure]
fn pure_mutating_input(counter: &mut u64) -> u64 {
    *counter
}

#[tigerstyle::pure]
fn pure_panics() {
    panic!("pure panic");
}

#[tigerstyle::pure]
fn pure_returns_file() -> std::fs::File {
    loop {}
}

fn main() {}
