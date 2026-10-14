## Why

Proof and artifact logic appears in multiple places across `crates/octet-standards` and `cargo-octet`, including BLAKE3 identity helpers, canonical Nix closure graph handling, proof rail relevance, proof receipt binding, and generated artifact validation. Duplicated logic increases the risk that the standards checker and runner disagree about the same evidence.

A shared pure library for proof and artifact decisions will make receipts more trustworthy and reduce future maintenance cost.

## What Changes

- Identify duplicated proof/artifact identity, canonicalization, and validation helpers.
- Move shared deterministic logic into a pure library module with positive and negative tests.
- Route `octet-standards` CLI and `cargo-octet` proof/artifact paths through the shared logic.
- Preserve existing command output, receipt shapes, and diagnostics unless an explicit contract update is made.

## Impact

- **Files**: `crates/octet-standards/src/main.rs`, `crates/octet-standards/src/proof.rs`, `crates/octet-standards/src/artifacts.rs`, `cargo-octet/src/prove.rs`, artifact/provenance tests, and possibly a shared crate/module.
- **Testing**: proof receipt validation tests, artifact manifest tests, runner prove tests, negative stale/tampered receipt tests, and existing Nix proof rails.
