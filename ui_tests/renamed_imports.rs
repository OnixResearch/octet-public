#![allow(dead_code, unused_imports)]
#![allow(non_trait_imports, path_segment_repetition, reason = "fixture targets another lint; the import and path-naming lints have their own fixtures")]

mod transport {
    pub struct Config;
}

mod rustc_hir {
    pub struct Item;
}

mod registry {
    pub struct Registry;
}

// Should NOT warn: the module file sits under a `generated/` source directory, which the
// default `generated_searchable_names` exception class keeps exempt.
#[path = "../policy_inputs/generated_searchable_names/renamed_imports/generated/imports.rs"]
mod generated_imports;

// Should NOT warn: canonical rustc-private shorthand is scoped to lint implementation code.
use crate::rustc_hir as hir;

use crate::transport::Config as TransportConfig;

// Should warn: the alias spelling alone does not document a downstream contract.
pub use crate::transport::Config as IncompatibleTransportConfig;

// Should warn: a `registry` path segment is ordinary source, not generated code.
use crate::registry::Registry as ServiceRegistry;

fn connect(config: TransportConfig) {
    let _ = config;
}

fn lookup(registry: ServiceRegistry) {
    let _ = registry;
}

fn use_rustc_alias(_: hir::Item) {}

fn main() {}
