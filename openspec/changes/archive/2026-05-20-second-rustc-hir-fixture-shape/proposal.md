## Why

The rustc-backed HIR extractor spike currently has deterministic evidence for one pinned checked fixture. That proves the narrow extraction/replay seam, but it leaves fixture-shape confidence weak: a trivial arithmetic function cannot exercise branch/control-flow HIR shape changes or prove the replay path rejects shape drift beyond the original fixture.

## What Changes

- Add a second explicitly named, checked-in rustc HIR fixture shape for a small branch/control-flow function.
- Extend the rustc HIR spike path so operators can select the second fixture shape without broadening support beyond checked-in fixtures.
- Add deterministic positive replay and negative mismatch evidence for the new shape.
- Preserve Function Object v1 identity, exact rustc/toolchain/extractor/schema identity, fail-closed checks, and all non-proof boundaries.

## Non-Goals

- No general HIR extraction support for arbitrary Rust files.
- No THIR or MIR implementation.
- No claim of behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism.
- No replacement of `octet.function-object.v1` as the canonical review/provenance identity.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fixtures/hir-spike/`, focused tests, docs/contract metadata if the selector is operator-visible.
- **APIs**: The rustc HIR spike selector remains fixture-scoped; any new flag/value must name a checked-in fixture shape rather than implying arbitrary path support.
- **Testing**: focused rustc HIR spike tests, positive CLI replay smoke, negative mismatch smoke, docs/metadata boundary checks, OpenSpec validation.
