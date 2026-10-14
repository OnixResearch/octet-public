## Why

Octet's selected rustc THIR fixture contract already covers both `checked-arithmetic` and `call-return`, but the selected rustc MIR fixture contract only covers `checked-arithmetic`. Adding one bounded MIR `call-return` selector lets the Valence evidence ladder compare a call/return shaped function at the MIR supplement seam without claiming general MIR extraction.

## What Changes

- **Selected MIR fixture expansion**: Add exact selector `call-return` for `cargo-octet/tests/fixtures/mir-spike/call_return.rs::call_then_return`.
- **Runtime and inventory sync**: Update parser, fixture inventory, supplement emission/check replay, contract metadata, and docs to list exactly `checked-arithmetic` and `call-return` for MIR fixture support.
- **Negative boundaries**: Keep alias rejection and unsupported shapes fail-closed; default/non-fixture MIR remains `unsupported-planned`.

## Out of Scope

- General MIR extraction for arbitrary Rust paths.
- Semantic equivalence, correctness, pure-cache eligibility, or cross-toolchain stability claims.
- HIR or THIR selector expansion.
- Accepting aliases such as `checked` for MIR selectors.

## Capabilities

### Modified Capabilities
- `r[hir-mir-supplement-pilot]`: Extends the selected MIR fixture set from one exact selector to two exact selectors while preserving fail-closed boundaries.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fixtures/mir-spike/`, `cargo-octet/tests/fingerprint_consumer.rs`, docs, flake-backed gates, and synced OpenSpec specs.
- **APIs**: CLI accepts `--family mir --spike rustc-mir-fixture --fixture-shape call-return` only for the selected fixture path/function.
- **Dependencies**: No new dependencies.
- **Testing**: Focused parser, MIR fixture runtime/check replay, supplement contract, named MIR fixture gate, aggregate Valence evidence gate, OpenSpec validation, and diff check.

## Obligation cross-check

- MIR `call-return` support must be selected-fixture-only: exact selector, exact fixture path, and exact function.
- MIR selector inventory must list exactly `checked-arithmetic` and `call-return` after this change.
- MIR alias rejection remains: `checked` is not accepted as a MIR selector alias.
- Default/non-fixture MIR contract metadata remains `unsupported-planned`; the fixture runtime may report `pilot-supported` only for selected fixtures.
- All docs/spec/tasks must retain non-proof language for correctness, semantic equivalence, pure-cache eligibility, general MIR extraction, and cross-toolchain stability.
