## Why

Proof receipts now bind source, command, receipt schema, runner, backend, environment, and trust-boundary metadata. They still do not machine-check which verifier/tooling identity and toolchain identity the receipt claims, so a stale receipt can preserve current source/command hashes while omitting the verifier/toolchain accountability needed for reproducible verified rails.

## What Changes

- Add typed proof receipt verifier metadata to Nickel proof rails.
- Export verifier/toolchain identity and digest obligations into generated artifacts.
- Require verified receipt-only checks and opt-in command-generated receipts to include matching verifier/toolchain lines when configured.
- Add fixtures, regressions, documentation, and evidence proving stale receipts without verifier/toolchain digests fail.

## Capabilities

### Modified Capabilities
- `standards-codification`: Verified proof rails bind receipts to source, command, provenance, verifier identity, and toolchain identity.

## Impact

- **Files**: `crates/tigerstyle-standards`, `standards/policies`, proof fixtures, docs, flake checks, OpenSpec evidence.
- **APIs**: Nickel `ProofRailContract` gains verifier receipt metadata with default-empty compatibility.
- **Dependencies**: Reuses existing `blake3`; no hidden network or credential dependencies.
- **Testing**: Focused Rust tests, Nickel policy validation, generated artifact assertions, docs examples, strict OpenSpec validation.
