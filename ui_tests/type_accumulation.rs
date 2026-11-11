#![warn(type_accumulation)]
#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

pub mod crowded {
    pub struct Thing;

    impl Thing { pub fn a(&self) {} }
    impl Thing { pub fn b(&self) {} }
    impl Thing { pub fn c(&self) {} }
    impl Thing { pub fn d(&self) {} }
    impl Thing { pub fn e(&self) {} }
}

// File-backed module with 13 public items (limit: 12).
#[path = "type_accumulation/src/filebacked.rs"]
pub mod filebacked;

// Public fields, impl members, nested-module contents, re-export stems, and comments are not
// the module's own items: this module exposes 4 public items (limit: 12).
pub mod members_are_not_module_items {
    // pub pub pub pub pub pub pub pub pub pub pub pub pub
    pub struct Settings {
        pub first: u8,
        pub second: u8,
        pub third: u8,
        pub fourth: u8,
    }

    impl Settings {
        pub fn first(&self) -> u8 { self.first }
        pub fn second(&self) -> u8 { self.second }
        pub fn third(&self) -> u8 { self.third }
        pub fn fourth(&self) -> u8 { self.fourth }
    }

    pub mod nested {
        pub fn one() {}
        pub fn two() {}
    }

    pub use nested::{one, two};
}

// Same-named types in different modules are different types: 3 + 2 impl blocks, no warning.
pub mod same_name_alpha {
    pub struct Shared;

    impl Shared { pub fn a(&self) {} }
    impl Shared { pub fn b(&self) {} }
    impl Shared { pub fn c(&self) {} }
}

pub mod same_name_beta {
    pub struct Shared;

    impl Shared { pub fn d(&self) {} }
    impl Shared { pub fn e(&self) {} }
}

// Different spellings of one type count together: 5 impl blocks (limit: 4).
pub mod spelled {
    pub struct Spelled;

    impl Spelled { pub fn a(&self) {} }
    impl Spelled { pub fn b(&self) {} }
    impl self::Spelled { pub fn c(&self) {} }
    impl crate::spelled::Spelled { pub fn d(&self) {} }
    impl super::spelled::Spelled { pub fn e(&self) {} }
}

// The crate root counts like any other module: it exposes 13 public items (limit: 12).
pub fn root_one() {}
pub fn root_two() {}
pub fn root_three() {}
pub fn root_four() {}
pub fn root_five() {}
pub fn root_six() {}
pub fn root_seven() {}

fn main() {}
