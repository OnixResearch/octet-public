# Valence Function Object core crate seam

## Why

Phase 1 proved that `valence-core` can own a small deterministic evidence-index seam while Octet remains the CLI/filesystem shell. The next extraction step is to move the core Function Object v1 DTO/hash/caveat construction into `valence-core`, because Function Object v1 is the substrate for Valence object evidence and future standalone boundaries.

## What changes

- Add a `valence_core::function_object` module for pure Function Object v1 DTOs, canonical object hashing, source-token caveat summaries, and pure-cache eligibility classification.
- Keep source scanning, filesystem reads, CLI parsing, marker updates, corpus orchestration, supplements, stdout/stderr, and process execution in `cargo-octet`.
- Have `cargo-octet` build Function Object v1 records by passing source-scanner inputs into `valence-core`.
- Preserve object hashes, existing command output, fixtures, named Nix gates, and bounded non-claims.

## Out of scope

- No standalone repository creation or package publication.
- No broad supplement/HIR/THIR/MIR migration.
- No change to marker syntax, corpus receipt schema, bundle schema, or evidence-index page contract.
- No correctness, semantic-equivalence, exhaustive Rust coverage, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction claims.
