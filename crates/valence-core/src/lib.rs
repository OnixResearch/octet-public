//! Frozen Octet-hosted Valence compatibility fixture.
//!
//! The package identity is `octet-valence-legacy-fixture`; this crate is excluded
//! from the Octet workspace and MUST NOT be selected as canonical `valence-core`.
//! Production Octet code delegates to the exact standalone Valence dependency.
//!
//! The retained modules support bounded positive and negative drift checks only.
//! They do not prove behavioral correctness, semantic equivalence, external
//! consumer compatibility, release eligibility, or whole-program safety.

pub mod evidence_codebase_index;
pub mod evidence_index;
pub mod function_object;
