## Why

`valence-core::function_object` is the most important extraction seam because Function Object v1 is the stable review/provenance identity for selected Rust functions. The current API still lets the Octet shell pass raw source, normalized source, dependency status strings, and hash-material decisions as loosely coupled strings. That keeps the seam working, but it makes future extraction riskier because callers can accidentally provide inconsistent normalization inputs or reviewers must infer exactly which fields are hashed.

## What Changes

- Move normalization ownership behind the `valence-core` Function Object API or wrap normalized input in an explicit typed contract that records the normalization algorithm.
- Replace stringly internal error/status/issue plumbing with typed enums and newtypes while preserving serialized Function Object v1 compatibility.
- Introduce an explicit private hash-material DTO for Function Object v1 object hashing.
- Add positive and negative tests proving hash stability, fail-closed invalid inputs, caveated purity, and no hidden correctness claims.
- Preserve existing object hashes unless a reviewed schema/hash-material migration explicitly updates fixtures and compatibility notes.

## Impact

- **Files**: `crates/valence-core/src/function_object.rs`, `cargo-octet/src/object.rs`, object corpus fixtures/tests as needed, `cairn/specs/valence-project-extraction/spec.md`.
- **Testing**: `cargo test -p valence-core function_object`, focused `cargo-octet` object tests, object corpus gates, `valence-object-evidence`, and Cairn validation/gates.
