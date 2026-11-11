#![allow(dead_code, unused_imports)]
#![allow(path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

mod addr {
    pub struct Addr;
}

mod rustc_hir {
    pub struct Item;
}

mod registry {
    pub struct Registry;
}

// Should NOT warn: the module file sits under a `generated/` source directory, which the
// default `generated_searchable_names` exception class keeps exempt.
#[path = "../policy_inputs/generated_searchable_names/non_trait_imports/generated/imports.rs"]
mod generated_imports;

// Should NOT warn: canonical rustc-private shorthand keeps lint implementation code readable.
use crate::rustc_hir as hir;

use crate::addr::Addr;

// Should warn: a `registry` path segment is ordinary source, not generated code.
use crate::registry::Registry;

fn route(addr: Addr) {
    let _ = addr;
}

fn lookup(registry: Registry) {
    let _ = registry;
}

fn use_rustc_alias(_: hir::Item) {}

fn main() {}
