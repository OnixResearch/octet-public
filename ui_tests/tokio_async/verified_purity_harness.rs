#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![allow(tigerstyle::ambient_clock)]
#![allow(non_trait_imports, path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

// The crate root sits outside `verified/`, as `src/lib.rs` does in a normal
// layout; only the file-backed module below is a verified module. The fixture
// compiles with `--test`, so harness test functions survive expansion.
#[path = "src/verified/mod.rs"]
mod verified;
