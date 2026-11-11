## Why

Octet now treats deterministic JSON as the v1 machine-readable receipt and fixture format, while Rust DTOs remain the runtime source of truth. The object corpus receipt/check surfaces need a small Rust-derived contract metadata rail so reviewers and CI can verify required schema names, version constants, required fields, boundary fields, and checked-in fixture compatibility without hand-reading DTOs.

## What Changes

- Add an Octet-native object corpus contract metadata surface generated from Rust constants/DTO knowledge.
- Cover `octet.function-object-corpus-receipt.v1` and the corpus check report shape.
- Validate the checked-in internal corpus receipt fixture against the contract metadata.
- Add positive and negative tests for required fields, schema/version compatibility, and boundary non-claims.

## Capabilities

### Modified Capabilities
- `standards-codification`: Object corpus JSON receipts/check reports gain reviewable Rust-derived contract metadata and fixture validation.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/src/cli.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec artifacts/evidence.
- **APIs**: Adds a read-only `cargo octet object corpus contract --output-format human|json` metadata command.
- **Dependencies**: No new dependencies.
- **Testing**: Focused cargo tests for object corpus contract metadata, then full cargo/OpenSpec/diff verification.
