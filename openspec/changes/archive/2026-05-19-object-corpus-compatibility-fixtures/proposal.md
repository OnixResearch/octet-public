# Object Corpus Compatibility Fixtures

## Summary

Add checked-in v1 object corpus receipt compatibility fixtures that make accepted and rejected schema evolution behavior durable. The fixtures preserve Octet's boundary that object corpus receipts identify review/provenance evidence and do not prove correctness or semantic equivalence.

## Motivation

Octet now emits and checks deterministic object corpus receipts, exposes Rust-derived contract metadata, and runs the internal corpus gate in CI. The remaining compatibility gap is that schema evolution examples are embedded in generated test mutations rather than represented as named, checked-in positive and negative fixtures. Reviewers and future maintainers need explicit fixtures for what v1 accepts and what fails closed.

## Scope

- Add a small static compatibility corpus source fixture.
- Add a v1 positive receipt fixture that must replay cleanly.
- Add negative receipt fixtures for missing required fields and future schema versions.
- Add tests that validate fixture names, expected status, expected error kinds, and boundary claims.
- Keep future schema versions rejected unless an explicit future compatibility mechanism is introduced later.

## Non-goals

- Do not add v2 compatibility or migration behavior.
- Do not broaden the internal object corpus.
- Do not claim correctness, semantic equivalence, or cross-toolchain determinism.
