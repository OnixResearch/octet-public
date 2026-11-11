#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
//! Pure bounded filesystem-tree admission and deterministic copy planning.
//!
//! This crate accepts owned facts. It performs no filesystem I/O, hashing,
//! logging, process execution, clock access, or environment access.

extern crate alloc;

mod admission;
mod assembly;
mod model;
mod policy;
mod segments;
mod validation;

pub use admission::{Blocker, BlockerKind, PlanError, TreeRequest, plan_tree};
pub use model::{
    Blake3Digest, CopyOperation, EntryKind, FileMode, MemberFact, RawObservation, SymlinkPolicy,
    TreePlan,
};
pub use policy::{LimitKind, LimitValues, TreeLimits, TreeLimitsError};
pub use segments::{PathError, RelativePath};
