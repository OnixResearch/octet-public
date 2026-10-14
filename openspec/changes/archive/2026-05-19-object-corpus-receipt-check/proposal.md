## Why

Octet can emit deterministic function-object corpus receipts, but operators do not yet have a first-class command that replays a saved receipt against current source and reports whether the reviewed object set still matches. Without that check, receipt evidence is harder to enforce in local gates or Cairn-style replay flows.

## What Changes

- Add an object-corpus receipt check mode that reads a saved receipt and recomputes the current corpus receipt for selected source paths.
- Report pass/fail status with deterministic JSON/human output and fail closed when object-set identity, selected source paths, caveat distribution, pure-cache blocker count, or boundary claims differ.
- Preserve the existing boundary: receipt checks validate evidence identity and caveat summaries, not correctness or semantic equivalence.

## Capabilities

### Modified Capabilities
- `standards-codification`: Extends function-object corpus receipts from emit-only evidence to replayable local gate evidence.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/object.rs`, `cargo-octet/tests/fingerprint_consumer.rs`.
- **APIs**: Adds `cargo octet object corpus check --receipt RECEIPT.json [PATHS...]`.
- **Dependencies**: No new dependencies expected.
- **Testing**: Add pass, drift, and boundary-tamper tests; save deterministic evidence under this change.
