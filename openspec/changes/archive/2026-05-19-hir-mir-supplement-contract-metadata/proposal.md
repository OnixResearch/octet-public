## Why

Octet now has an accepted HIR/MIR supplemental evidence contract, but the machine-readable object corpus contract metadata still only describes current receipt and check-report shapes. Operators need a fail-closed contract surface that says HIR/MIR supplements are planned/unsupported until real extractor fixtures exist.

## What Changes

- Extend `cargo octet object corpus contract --output-format json|human` with planned HIR and MIR supplement metadata.
- Mark each supplement family as unsupported before extractor implementation.
- Preserve Function Object v1 source-token identity as the review/provenance anchor.
- Preserve explicit no-correctness, no semantic-equivalence, and no cross-toolchain stability boundaries.

## Capabilities

### Modified Capabilities
- `standards-codification`: Adds machine-readable fail-closed HIR/MIR supplement contract metadata.

## Impact

- **Files**: `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec standards codification.
- **APIs**: Additive JSON/human fields in object corpus contract metadata.
- **Dependencies**: None.
- **Testing**: Focused contract metadata tests plus full Octet verification.
