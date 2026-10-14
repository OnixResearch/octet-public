## Why

The HIR/MIR supplement pilot now provides bounded source-token supplement records, but it deliberately does not implement rustc-backed HIR extraction. The next valuable step is a tiny, fixture-backed HIR extractor spike that proves Octet can collect toolchain-scoped HIR evidence without changing Function Object v1 identity or claiming correctness.

## What Changes

- Add a narrow rustc-backed HIR extraction path for one selected fixture function.
- Emit a deterministic HIR observation supplement that references the existing Function Object v1 `object_hash`.
- Record exact rustc/toolchain identity, extractor identity, schema version, fixture context, and bounded caveats.
- Add fail-closed replay checks for toolchain, context, extractor, schema, and observation mismatches.

## Non-Goals

- No full HIR coverage for arbitrary Rust crates.
- No THIR or MIR implementation.
- No semantic-equivalence, behavioral-correctness, pure-cache, cross-toolchain-stability, or exhaustive Rust coverage claim.
- No replacement of `octet.function-object.v1` as the canonical review/provenance identity.

## Capabilities

### New Capabilities
- `rustc-hir-extractor-spike`: Fixture-backed HIR observation extraction and replay evidence for selected Function Object v1 records.

### Modified Capabilities
- `standards-codification`: Extends HIR supplement requirements from source-token pilot status to a bounded rustc-backed spike with fail-closed identity checks.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/`, docs, and OpenSpec evidence.
- **APIs**: May add a clearly named experimental HIR extractor path under existing `object supplement` surfaces or a dedicated internal helper; public naming must remain pilot/spike-scoped.
- **Dependencies**: Any rustc-private or toolchain dependency must be pinned through the Nix devshell and recorded in receipts.
- **Testing**: Focused positive fixture, negative mismatch fixtures, docs/contract checks, OpenSpec validation, and a bounded aggregate evidence rerun.
