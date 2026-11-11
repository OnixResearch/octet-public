//! Bounded four-version semantic merge relation prototype.
//!
//! This crate is an opt-in research surface. It is not wired into starter
//! `cargo octet check`, does not synthesize merges, and does not prove Rust
//! source-to-IR semantic correspondence; the bounded frontend records only a
//! syntactic mapping with explicit non-claims. Every receipt carries the reviewed profile,
//! role-tagged identities, assumptions, bounds, deterministic outcome, and
//! consumer non-claims.

pub mod checker;
pub mod corpus;
pub mod evaluate;
pub mod external_lean;
pub mod frontend;
pub mod identity;
pub mod model;
pub mod receipt;

/// Reviewed profile identity.
pub const PROFILE_ID: &str = receipt::PROFILE_ID;
/// Reviewed profile version.
pub const PROFILE_VERSION: &str = receipt::PROFILE_VERSION;
/// Claim scope bound into admissions and receipts.
pub const CLAIM_SCOPE: &str = receipt::CLAIM_SCOPE;
