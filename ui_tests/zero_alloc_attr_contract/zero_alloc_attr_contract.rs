// zero_alloc_attr_contract: functions marked with #[zero_alloc] that allocate

#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(unused)]

#[tigerstyle::zero_alloc]
fn makes_vec() {
    let _v: Vec<u8> = Vec::new();
}

#[tigerstyle::zero_alloc]
fn makes_box() {
    let _b: Box<u64> = Box::new(42u64);
}

#[tigerstyle::zero_alloc]
fn makes_string() {
    let _s = String::new();
}

#[tigerstyle::zero_alloc]
fn makes_string_from() {
    let _s = String::from("hello");
}

#[tigerstyle::zero_alloc]
fn makes_to_string() {
    let _s = "hello".to_string();
}

// format!() expands to a macro that ultimately produces a String
#[tigerstyle::zero_alloc]
fn makes_format() {
    let _s = format!("value: {}", 42);
}

// Array lengths, repeat counts, and statement-level items are nested bodies;
// walking them must not ICE, and later allocations are still reported.
#[tigerstyle::zero_alloc]
fn makes_vec_after_nested_bodies() {
    const K: u8 = 1;
    let _buf: [u8; 4] = [K; 4];
    let _v: Vec<u8> = Vec::new();
}

const fn seven() -> u8 {
    7
}

// Calls inside nested const items, array-length constants, and nested functions
// belong to other bodies: walking them must not ICE, nested function bodies are
// separate functions, and allocations in the marked body are still reported.
#[tigerstyle::zero_alloc]
fn makes_vec_after_nested_calls() -> u8 {
    const DELAY: std::time::Duration = std::time::Duration::from_secs(1);
    let _buf = [0u8; core::mem::size_of::<u64>()];
    fn helper() -> u8 {
        let _unmarked: Vec<u8> = Vec::new();
        seven()
    }
    let _v: Vec<u8> = Vec::new();
    helper()
}

fn main() {}

fn unmarked_outer_with_marked_helper() -> u8 {
    // Should NOT warn: only the nested helper carries the marker.
    #[tigerstyle::zero_alloc]
    fn helper() -> u8 {
        seven()
    }
    let _v: Vec<u8> = Vec::new();
    helper()
}

fn unmarked_outer_with_marked_allocating_helper() {
    // The nested helper is marked and allocates, so it alone is reported.
    #[tigerstyle::zero_alloc]
    fn allocating_helper() {
        let _b: Box<u64> = Box::new(7u64);
    }
    allocating_helper();
}

fn unmarked_outer_with_marker_text() {
    // Should NOT warn: #[tigerstyle::zero_alloc] in a comment or string is not an attribute.
    let _text = "#[tigerstyle::zero_alloc]";
    let _v: Vec<u8> = Vec::new();
}

#[doc = "Mark helpers #[tigerstyle::zero_alloc] once they stop allocating."]
fn doc_attr_mentions_marker() {
    // Should NOT warn: a doc attribute naming the marker is not the marker.
    let _v: Vec<u8> = Vec::new();
}
