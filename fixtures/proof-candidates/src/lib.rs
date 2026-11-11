//! Fixture crate for `cargo octet prove --candidates`.
//!
//! The crate is standalone so its checked source root is the fixture directory,
//! which keeps candidate paths stable (`src/codec.rs`) in the walkthrough.

#![feature(register_tool)]
#![register_tool(tigerstyle)]

pub mod codec;
pub mod codec_mutant;
