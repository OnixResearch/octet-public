#![feature(register_tool, stmt_expr_attributes)]
#![register_tool(tigerstyle)]
#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

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
fn pure_prints(value: u64) {
    print!("{value}");
}

#[tigerstyle::pure]
fn pure_eprints(value: u64) {
    eprint!("{value}");
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

fn unmarked_outer_with_marked_helper() -> u64 {
    // Should NOT warn: only the nested helper carries the marker.
    #[tigerstyle::pure]
    fn helper(value: u64) -> u64 {
        value
    }
    let _bytes = std::fs::read("state.bin");
    helper(1)
}

fn unmarked_outer_with_marked_impure_helper() {
    // The nested helper is marked and impure, so it alone is reported.
    #[tigerstyle::pure]
    fn impure_helper() {
        let _home = std::env::var("HOME");
    }
    impure_helper();
}

fn unmarked_outer_with_marked_closure() -> u64 {
    // Should NOT warn: only the closure carries the marker.
    let double = #[tigerstyle::pure]
    |value: u64| value * 2;
    let _bytes = std::fs::read("state.bin");
    double(1)
}

fn unmarked_outer_with_marker_text() {
    // Should NOT warn: #[tigerstyle::pure] in a comment or string is not an attribute.
    let _text = "#[tigerstyle::pure]";
    let _bytes = std::fs::read("state.bin");
}

#[doc = "Mark helpers #[tigerstyle::pure] once they stop reading files."]
fn doc_attr_mentions_marker() {
    // Should NOT warn: a doc attribute naming the marker is not the marker.
    let _bytes = std::fs::read("state.bin");
}
