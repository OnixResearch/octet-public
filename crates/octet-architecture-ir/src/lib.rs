#![forbid(unsafe_code)]

mod contract;
mod core;
mod identity;
mod membership;
mod model;
mod roles;
mod selection;
mod stamp;

pub use core::compilation_unit_identity;
pub use core::coverage_identity;
pub use core::fact_identity;
pub use core::ir_identity;
pub use core::normalize_architecture_shards;
pub use core::receipt_identity;
pub use core::shard_identity;
pub use core::source_span_identity;
pub use core::unknown_fact_identity;
pub use core::verify_ir_unit_bindings;
pub use core::verify_observation_set;

pub use contract::render_nickel_contract;
pub use identity::canonical_identity;
pub use identity::canonical_json;
pub use identity::framed_blake3;
pub use membership::verify_ir_unit_memberships;
pub use model::*;
pub use roles::UnitRoleError;
pub use roles::select_compilation_unit_role;
pub use selection::CompilerUnitObservation;
pub use selection::UnitSelectionError;
pub use selection::select_compilation_unit;
pub use selection::validate_collector_units;
pub use stamp::compilation_unit_stamp_directory;
pub use stamp::compilation_unit_stamp_name;
pub use stamp::compilation_unit_stamp_path;

#[cfg(test)]
mod tests;
