#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

mod addr {
    pub struct Addr;
}

/// Public owner-type re-export for downstream callers.
pub use crate::addr::Addr;

fn main() {}
