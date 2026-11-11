## Why

Octet already has a bounded real-code object corpus over three implementation modules. The next highest-signal pilot is internal: broaden the Octet corpus before involving downstream projects, and make the pilot evidence machine-readable instead of transcript-only.

## What Changes

- Add an Octet-native object corpus receipt command for deterministic JSON summaries of function-object coverage.
- Broaden the internal corpus from the original three object/fingerprint modules to a larger set of Octet implementation modules.
- Preserve the current boundary: object receipts are review/provenance evidence, not callable content-addressed functions or correctness proofs.

## Capabilities

### Modified Capabilities
- `standards-codification`: Adds an internal corpus-pilot receipt surface for function-object evidence.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`, OpenSpec evidence.
- **APIs**: New `cargo octet object corpus receipt --output RECEIPT.json [PATHS...]` command.
- **Dependencies**: No new dependencies expected.
- **Testing**: Targeted CLI tests for receipt determinism, coverage, caveat counts, and no-correctness boundary; saved evidence transcript.
