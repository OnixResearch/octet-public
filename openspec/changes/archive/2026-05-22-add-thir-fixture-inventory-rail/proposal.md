## Why

Octet now has two selected rustc THIR fixture shapes (`checked-arithmetic` and `call-return`), but the inventory rail still needs explicit reviewable coverage showing THIR selectors are discoverable through `cargo octet object supplement fixtures` with the same bounded/no-proof semantics as HIR.

## What Changes

- Strengthen the supplement fixture inventory contract for THIR.
- Add focused tests/evidence that `--family thir --spike rustc-thir-fixture` lists exactly `checked-arithmetic` and `call-return`.
- Keep `branch-control-flow` HIR-only and absent from THIR inventory output.
- Preserve default/non-fixture THIR family status as `unsupported-planned`; inventory entries are selected-fixture runtime metadata only.

## Capabilities

### Modified Capabilities
- `standards-codification`: THIR fixture inventory has deterministic selector discovery and bounded operator evidence for the HIR/MIR/THIR supplement pilot contract metadata requirement.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, docs/specs if wording needs sync.
- **APIs**: No new broad API; existing `object supplement fixtures` THIR path gets explicit tests and evidence.
- **Testing**: Focused unit/integration tests, strict OpenSpec validation, diff hygiene, and done-review.

## Out of Scope

- No third THIR fixture shape.
- No general THIR extraction, semantic correctness proof, dependency resolution, pure-cache eligibility, or cross-toolchain equivalence claim.
- No change to default/non-fixture THIR `unsupported-planned` status.
