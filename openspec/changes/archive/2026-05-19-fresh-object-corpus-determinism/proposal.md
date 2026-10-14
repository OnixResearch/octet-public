## Why

Object corpus receipts now have deterministic JSON, replay checks, fixture gates, and Rust-derived contract metadata. The remaining local validation gap is fresh-target repeatability: a receipt should not drift merely because the surrounding Cargo target/cache directory changes.

## What Changes

- Add a fresh-target object corpus determinism test rail.
- Generate repeated internal corpus receipts under scrubbed environments with distinct fresh `CARGO_TARGET_DIR` values.
- Assert stable receipt bytes and stable compared receipt fields: object count, source paths, object-set hash, caveat counts, pure-cache blocker count, and boundary fields.
- Save deterministic evidence for the fresh-target rail.

## Capabilities

### Modified Capabilities
- `standards-codification`: Object corpus receipts gain bounded local fresh-target repeatability coverage.

## Impact

- **Files**: `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec artifacts/evidence.
- **APIs**: No CLI/API changes.
- **Dependencies**: No new dependencies.
- **Testing**: Focused fresh-target cargo test plus full cargo/OpenSpec/diff verification.
