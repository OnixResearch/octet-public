//! Pure Valence evidence helpers for bounded evidence identity and linkage surfaces.
//!
//! # Current public modules
//!
//! - `artifact_ref`: validates typed artifact references, digest spelling profiles, confined path
//!   metadata, declared hash/size metadata, and public-field compatibility adapters over
//!   already-loaded in-memory metadata.
//! - `evidence_adapters`: validates parsed Cairn, ChaosControl, state-machine, and canonical-core
//!   cutover rows as bounded Evidence IR identity/linkage inputs.
//! - `evidence_codebase_index`: builds, queries, renders, and boundary-checks Octet-owned
//!   evidence-codebase indexes from in-memory artifacts with iddqd-backed normalized source lookup.
//! - `evidence_index`: renders and checks the current-head Valence evidence page from typed inputs.
//!   Octet retains command execution, repository-state capture, freshness orchestration, and
//!   publication decisions.
//! - `formal_proof_chain`: validates frozen formal-proof stack-chain fixtures for role-preserving
//!   linkage and Cairn readiness-input counts.
//! - `function_address_profile`: validates Octet/Kamacite function-address evidence profiles as
//!   recorded-only Evidence IR with typed digest domains, graph rows, and non-claim boundaries.
//! - `function_object`: constructs Function Object v1 DTOs, canonical hash payloads, caveat
//!   summaries, and pure-cache eligibility from source facts. Octet retains scanning, source-marker
//!   mutation, corpus orchestration, and CLI/status reporting.
//! - `oxide_primitives`: records reference-only Oxide primitive intake for typed IDs, derived-key
//!   collections, semantic diffs, and bounded attestation vocabulary.
//! - `preserves_evidence`: validates Preserves evidence envelopes and Preserves-to-Valence bridge
//!   rows over already-loaded canonical bytes/text.
//! - `shared_identity`: normalizes BLAKE3 spellings, digest roles, canonical hashes, self-hash
//!   exclusion profiles, and reusable non-claim boundaries.
//! - `stack_role_registry`: validates stack schema/role rows, verification-role counting
//!   vocabulary, compatibility aliases, and consumer constant drift.
//!
//! # Boundary
//!
//! This crate intentionally contains deterministic core logic only. CLI I/O,
//! filesystem reads, artifact byte measurement, path resolution, source scanning,
//! object marker mutation, corpus orchestration, supplement orchestration,
//! stdout/stderr, process exits, repository tags, package publication, and
//! release ownership remain in shells or producer repositories. Standalone
//! Valence owns Evidence IR semantics; Octet remains an evidence producer and
//! source-scanning/proof-rail orchestrator.
//!
//! # Evidence boundary
//!
//! `valence-core` currently supports bounded local evidence replay for selected
//! Octet/Valence surfaces. It does not claim behavioral correctness, semantic
//! equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine
//! determinism, cross-toolchain determinism, external consumer compatibility, or
//! general HIR/THIR/MIR extraction.

pub mod artifact_ref;
pub mod ast_grep_evidence;
pub mod capability_plan;
pub mod evidence_adapters;
pub mod evidence_chain;
pub mod evidence_codebase_index;
pub mod evidence_harness;
pub mod evidence_index;
pub mod evidence_ir;
pub mod evidence_model;
pub mod formal_proof_chain;
pub mod function_address_preserves;
pub mod function_address_profile;
pub mod function_object;
pub mod hegel_property;
pub mod oxide_primitives;
pub mod preserves_evidence;
pub mod proof_carrying_ir;
pub mod provenance;
pub mod shared_identity;
pub mod stack_provenance_adapter;
pub mod stack_role_registry;
pub mod trellis_proof_evidence;
pub mod wasm_component_evidence;
