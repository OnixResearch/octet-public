## Why

Octet now has a fail-closed THIR supplement seam and one explicit fixture-backed THIR selector, `checked-arithmetic`. That proves the selected-fixture contract shape but leaves the operator surface too narrow to show that THIR fixture support can grow by adding closed selectors rather than by relaxing the default THIR boundary.

## What Changes

- Add one new checked-in THIR fixture selector, `call-return`, bound to one exact fixture path/function.
- Extend the THIR fixture inventory, parser/help, runtime supplement emit/check path, docs, tests, and contract metadata to list both selected THIR fixture shapes.
- Preserve default/non-fixture THIR status as `unsupported-planned`; only explicit selected fixture runtime paths may report `pilot-supported`.

## Non-Goals

- No general THIR extraction support.
- No arbitrary THIR fixture paths or implicit shape defaults.
- No correctness, semantic-equivalence, dependency-resolution, pure-cache, or cross-toolchain-stability claim.
- No change to HIR `branch-control-flow`; it remains HIR-only and is not accepted for THIR.

## Capabilities

### Modified Capabilities
- `r[hir-mir-supplement-pilot]`: adds a second closed THIR fixture shape while preserving the fail-closed default THIR family contract.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, `cargo-octet/tests/fixtures/thir-spike/*.rs`, contributor/evidence-boundary docs, and standards OpenSpec text.
- **APIs**: Adds explicit `--fixture-shape call-return` for `--family thir --spike rustc-thir-fixture`; existing checked-arithmetic selector remains supported.
- **Testing**: Focused parser, THIR fixture emit/check, supplement regression, object corpus contract, OpenSpec gates, final strict validation, `git diff --check`, and done-review.
