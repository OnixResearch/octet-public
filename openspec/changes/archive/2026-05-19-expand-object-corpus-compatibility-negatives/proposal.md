## Why

Object corpus compatibility fixtures currently prove one accepted v1 receipt and two rejected shapes: a missing `object_set_hash` and a future schema version. The checker already fails closed for additional important compatibility boundaries, but those behaviors are not preserved as named fixtures.

## What Changes

- Add checked-in negative compatibility fixtures for malformed receipt JSON, duplicate receipt `source_paths`, boundary claim tampering, empty source selection replay, and unsupported non-Rust source selection.
- Extend the compatibility test rail so each fixture or pinned source-selection case asserts the expected fail-closed status, error kind, field, or mismatch field.
- Save focused and full verification transcripts under the change evidence directory.

## Out of Scope

- No v2 migration support.
- No new object identity algorithm.
- No correctness, semantic-equivalence, or cross-toolchain determinism claim.
- No downstream/Aspen pilot.

## Impact

- Files: `cargo-octet/tests/fixtures/object-corpus/compatibility/*`, `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec artifacts.
- APIs: no CLI or JSON schema changes.
- Testing: focused `object_corpus_compatibility` rail plus full Octet verification.
