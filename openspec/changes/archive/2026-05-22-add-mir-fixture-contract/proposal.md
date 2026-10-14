## Why

Octet's Function Object v1 supplement contract already names MIR as a future evidence family, but current runtime support stops at generic `unsupported-planned` pilot records. HIR and THIR now have bounded rustc fixture rails, inventory, and aggregate evidence gates; MIR needs the same selector-bounded contract before implementation starts so future Valence evidence does not overclaim arbitrary MIR extraction.

## What Changes

- **MIR fixture scope**: Add a closed `rustc-mir-fixture` spike scope for `cargo octet object supplement emit|check|fixtures`.
- **Selected MIR fixture**: Bind initial MIR support to exactly one checked-in fixture selector, `checked-arithmetic`, with a pinned fixture path/function pair.
- **Fail-closed boundaries**: Preserve default/non-fixture MIR behavior as `unsupported-planned`; only the explicit selected fixture path may report `pilot-supported`.
- **Aggregate evidence**: Extend focused tests and the aggregate Valence object evidence gate once the implementation lands.

## Out of Scope

- No general MIR extractor, arbitrary-path MIR support, correctness proof, semantic-equivalence proof, pure-cache claim, cross-toolchain stability claim, or exhaustive Rust coverage.
- No HIR or THIR selector expansion.
- No downstream Cairn/Trellis consumer contract changes beyond Octet's own receipt/evidence surface.

## Capabilities

### Modified Capabilities
- `standards-codification`: MIR supplement evidence gains a selected-fixture pilot path with explicit non-proof boundaries.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, MIR fixture files under `cargo-octet/tests/fixtures/`, docs, flake aggregate gate, and standards-codification OpenSpec delta.
- **APIs**: CLI accepts `--spike rustc-mir-fixture` where paired with `--family mir`; selected fixture commands use `--fixture-shape checked-arithmetic`.
- **Dependencies**: None expected.
- **Testing**: Focused cargo tests for MIR parser/inventory/emit/check behavior, negative non-fixture boundaries, aggregate `valence-object-evidence`, strict OpenSpec validation, and diff whitespace check.

## Obligation cross-check

- Selected MIR fixture support must be closed over selector, path, and function; arbitrary MIR paths remain unsupported-planned.
- Default corpus contract metadata must continue to report MIR as `unsupported-planned` outside explicit selected fixture runtime commands.
- Every new scenario must have a deterministic verification row and evidence path before implementation is marked complete.
