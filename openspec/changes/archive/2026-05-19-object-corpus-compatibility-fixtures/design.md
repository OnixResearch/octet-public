# Design

## Fixture layout

Compatibility fixtures live under `cargo-octet/tests/fixtures/object-corpus/compatibility/`:

- `src/lib.rs` is the tiny replay corpus.
- `positive-v1-receipt.json` is a generated v1 receipt for `src/lib.rs`.
- `negative-missing-object-set-hash.json` removes the required `object_set_hash` field.
- `negative-future-schema-version.json` keeps the v1 schema id but sets an unsupported future `schema_version`.

The checked-in source plus receipts are intentionally small so a failed compatibility check is easy to inspect.

## Test behavior

Tests invoke `cargo octet object corpus check --receipt <fixture> --output-format json src/lib.rs` with the compatibility fixture directory as the working directory. The positive fixture must return `status=match`. Negative fixtures must return `status=invalid`, no comparisons, and the expected typed error kind/field.

## Boundary handling

The positive fixture test asserts the receipt boundary fields remain false for `claims_correctness` and `claims_semantic_equivalence`. Negative fixtures are compatibility examples only; they do not relax the fail-closed schema/version behavior.

## Evidence

Final evidence records the focused compatibility test, full `cargo-octet` tests, OpenSpec validation, and whitespace checks.
