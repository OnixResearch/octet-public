## Why

HIR already has a bounded `branch-control-flow` selected fixture, while THIR selected-fixture evidence is limited to `checked-arithmetic` and `call-return`. Adding one THIR branch-control-flow fixture closes the next highest-ROI evidence gap without broadening THIR support claims.

## What Changes

- Add a checked-in THIR `branch-control-flow` fixture selector and replayable emit/check evidence.
- Update THIR fixture inventory and CLI selector acceptance to include exactly `checked-arithmetic`, `call-return`, and `branch-control-flow`.
- Update docs/evidence boundaries so this remains selected-fixture-only and does not imply general THIR extraction, correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-toolchain stability.

## Out of Scope

- General THIR extraction/support for arbitrary Rust input.
- MIR `branch-control-flow` support; MIR remains exactly `checked-arithmetic` and `call-return`.
- Semantic equivalence, behavioral correctness, pure-cache, or cross-toolchain proof claims.

## Impact

- **Files**: `cargo-octet/src/{cli,object}.rs`, THIR fixture tests, docs, and OpenSpec evidence.
- **APIs**: No new public schema version; existing selected fixture inventory gains one THIR selector.
- **Testing**: Focused THIR inventory/fixture tests, THIR Nix rail, aggregate Valence object evidence rail, docs boundary check, strict OpenSpec validation, and whitespace checks.
