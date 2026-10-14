## Why

Octet contract metadata already names THIR supplemental evidence as an unsupported-planned family, but the runtime supplement CLI only accepts HIR and MIR. Operators need a concrete fail-closed THIR command path so future Valence/HIR/MIR work can test schema/boundary behavior without implying extractor support.

## What Changes

- **THIR family parsing**: Accept `--family thir` for `cargo octet object supplement emit|check` in pilot scope.
- **Fail-closed THIR receipts**: Emit THIR supplement records with schema `octet.function-object-thir-supplement.v1`, status `unsupported-planned`, explicit unsupported observations/caveats, and preserved no-correctness/no-semantic-equivalence boundaries.
- **Replay check**: Allow `object supplement check --family thir` to compare saved THIR receipts against freshly emitted fail-closed records.

## Out of Scope

- No THIR extractor, rustc THIR capture, semantic equivalence claim, pure-cache claim, cross-toolchain stability claim, or general HIR/MIR support expansion.
- No change to fixture inventory; it remains scoped to the rustc HIR fixture spike.

## Capabilities

### Modified Capabilities
- `standards-codification`: Runtime supplement family support gains a fail-closed THIR path matching the existing contract metadata.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, standards-codification OpenSpec delta.
- **APIs**: CLI accepts `--family thir` for `object supplement emit|check` only.
- **Dependencies**: None.
- **Testing**: Focused cargo tests for THIR emit/check fail-closed boundaries, strict OpenSpec validation, and diff whitespace check.

## Obligation cross-check

- Runtime support must remain fail-closed: the THIR record status is `unsupported-planned` and names missing extractor support.
- Replay support must compare deterministic THIR fields without treating hash agreement as correctness or semantic equivalence.
- HIR fixture inventory scope must not be broadened by this change.
