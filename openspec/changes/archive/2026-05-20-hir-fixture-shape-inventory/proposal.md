## Why

Octet now supports two closed rustc-backed HIR spike fixture shapes, but operators must infer the available shape names, default shape, source paths, and fixture-boundary wording from docs or source. That increases the chance of drift between CLI behavior, contract metadata, and review evidence.

## What Changes

- Add an operator-visible inventory surface for rustc HIR spike fixture shapes.
- Report each supported shape name with source path, function name, default marker, support status, and non-proof boundary metadata.
- Keep inventory output derived from the same closed fixture registry used by supplement emit/check.
- Add deterministic positive inventory evidence and negative scope evidence.

## Non-Goals

- No arbitrary HIR fixture paths.
- No new fixture shapes.
- No THIR/MIR implementation.
- No claim of correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism.
- No change to Function Object v1 identity or supplement replay semantics.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, focused tests, docs/contract metadata if needed, OpenSpec evidence.
- **APIs**: Adds a read-only fixture inventory command for `object supplement` scoped to `--spike rustc-hir-fixture`.
- **Testing**: parser/unit tests, CLI inventory smoke, docs/metadata assertions, OpenSpec validation.
