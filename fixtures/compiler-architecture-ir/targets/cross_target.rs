// `no_core` compiles this fixture for any built-in target without that
// target's standard library, so the collector can observe a target whose Rust
// tuple differs from its LLVM target name.
#![feature(no_core, lang_items)]
#![no_core]
#![allow(internal_features)]

#[lang = "pointee_sized"]
pub trait PointeeSized {}

#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}

#[lang = "sized"]
pub trait Sized: MetaSized {}

pub fn identity() {}
