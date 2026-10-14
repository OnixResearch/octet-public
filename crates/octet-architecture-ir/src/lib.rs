#![forbid(unsafe_code)]

mod contract;
mod core;
mod identity;
mod model;

pub use core::compilation_unit_identity;
pub use core::coverage_identity;
pub use core::fact_identity;
pub use core::ir_identity;
pub use core::normalize_architecture_shards;
pub use core::receipt_identity;
pub use core::shard_identity;
pub use core::source_span_identity;
pub use core::unknown_fact_identity;

pub use contract::render_nickel_contract;
pub use identity::canonical_identity;
pub use identity::canonical_json;
pub use identity::framed_blake3;
pub use model::*;

#[cfg(test)]
mod tests;
