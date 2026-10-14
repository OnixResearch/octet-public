# Valence in-repo core crate

## Why

Phase 0 established that Valence is ready for extraction scaffolding but not a full repository cutover. The next low-risk step is an in-repository crate boundary that gives Valence-owned deterministic evidence logic a standalone package identity while preserving Octet as the CLI shell and operator entrypoint.

## What changes

- Add a workspace crate named `valence-core` for pure deterministic Valence evidence logic.
- Move the current evidence-index render/check core into `valence-core` without changing the generated Markdown contract or pass/fail semantics.
- Keep filesystem reads, CLI parsing, stdout/stderr, and process orchestration in `cargo-octet`.
- Preserve existing named Nix gates and bounded non-claims.

## Out of scope

- No standalone repository creation.
- No package publication, release tag, or cutover.
- No broad Function Object/HIR/THIR/MIR migration.
- No correctness, semantic-equivalence, exhaustive coverage, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction claims.
