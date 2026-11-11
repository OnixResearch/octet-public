## Why

Object corpus receipts can now be emitted and checked, but malformed or future receipt shapes still need a deterministic failure contract before the checker becomes a CI/replay gate. Operators should get a machine-readable failure report for bad JSON, missing fields, unsupported schema versions, invalid input selections, and empty corpora rather than relying on ad hoc stderr text.

## What Changes

- Harden `cargo octet object corpus check --receipt RECEIPT.json [PATHS...]` with schema-aware validation before evidence comparison.
- Emit deterministic JSON/human failure reports for malformed receipts and invalid corpus selections.
- Add negative tests for malformed JSON, missing fields, wrong schema, future version, empty corpus, missing paths, duplicate paths, and non-Rust paths.

## Capabilities

### Modified Capabilities
- `standards-codification`: Function-object corpus receipt checks become safe to use as replay/CI gates because invalid inputs fail closed with typed reports.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec specs/evidence.
- **APIs**: No new command; hardens output/exit behavior for `cargo octet object corpus check`.
- **Testing**: Focused negative integration tests, OpenSpec validation, full cargo-octet test suite, formatting, and whitespace checks.
