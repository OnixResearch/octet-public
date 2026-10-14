## Why

MIR now has two selected fixture shapes (`checked-arithmetic` and `call-return`) and focused runtime tests, but the aggregate Valence evidence surface still exposes MIR only as a generic supplement fixture gate. THIR already has an inventory-focused rail in the aggregate gate. MIR should gain the same operator-visible inventory/roundtrip rail so selected MIR support cannot drift between the fixture inventory, emit/check commands, and aggregate evidence output.

## What Changes

- **MIR inventory aggregate rail**: Add a focused Nix check that proves the MIR fixture inventory lists exactly `checked-arithmetic` and `call-return`, and that every inventory-listed MIR emit/check command roundtrips selected-fixture evidence.
- **Aggregate composition**: Compose the focused MIR inventory rail into `checks.x86_64-linux.valence-object-evidence` and print a distinct PASS line for it.
- **Docs/spec wording**: Update contributor/operator wording to say MIR aggregate coverage is selected-fixture-only and does not imply general MIR extraction.

## Scope

### In scope
- MIR `rustc-mir-fixture` inventory and command-roundtrip evidence for the already-supported selectors `checked-arithmetic` and `call-return`.
- A named Nix rail and aggregate `valence-object-evidence` composition/output update.
- Focused tests and saved evidence for inventory, aggregate composition, docs boundaries, and no-overclaim behavior.

### Out of scope
- Adding a new MIR fixture shape.
- Changing MIR default/non-fixture behavior from `unsupported-planned`.
- Claiming general MIR extraction, correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, cross-machine determinism, or cross-toolchain determinism.

## Impact

- **Files**: `flake.nix`, `cargo-octet/tests/fingerprint_consumer.rs`, contributor/current evidence docs, and `openspec/specs/standards-codification/spec.md` after archive.
- **APIs**: No new CLI user command is expected; the change adds/renames Nix check surfaces and aggregate output lines.
- **Dependencies**: None expected.
- **Testing**: Focused `mir_fixture`/inventory tests, focused Nix MIR inventory rail, aggregate `valence-object-evidence`, strict OpenSpec validation, and diff whitespace checks.
