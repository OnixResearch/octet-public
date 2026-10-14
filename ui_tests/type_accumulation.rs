#![warn(type_accumulation)]

pub mod crowded {
    pub struct Thing;

    impl Thing { pub fn a(&self) {} }
    impl Thing { pub fn b(&self) {} }
    impl Thing { pub fn c(&self) {} }
    impl Thing { pub fn d(&self) {} }
    impl Thing { pub fn e(&self) {} }
}

fn main() {}
