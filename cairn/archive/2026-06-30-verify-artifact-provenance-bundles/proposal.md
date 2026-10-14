## Why

Octet now emits `provenance.jsonl`, but release operators still need a first-class way to verify that an artifact directory is complete, internally consistent, and bound to the exact artifact bytes it claims. Without a verifier, provenance remains descriptive rather than an enforceable release gate.

The next useful step is a local command that checks an artifact bundle before CI or release promotion.

## What Changes

- Add a verifier for `cargo octet check --artifact-dir` output that reads the bundle, parses `provenance.jsonl`, recomputes BLAKE3 hashes, validates evidence tiers and checker IDs, and checks manual-review actions.
- Define deterministic machine-readable and human-readable verification output for release gates.
- Fail closed for missing files, stale extras, malformed provenance, hash mismatches, unknown checker IDs, invalid tiers, and missing manual-review actions.
- Document the command as the release/operator check for local artifact evidence.

## Impact

- **Files**: `cargo-octet/src/cli.rs`, `cargo-octet/src/engine.rs` or a new artifact verification module, `cargo-octet/src/evidence.rs`, `tests/consumer_integration.rs`, README/operator docs, and Cairn specs.
- **Testing**: positive complete-bundle verification plus negative cases for partial bundles, stale extras, malformed provenance, hash mismatch, unknown checker, invalid tier, and missing human action.
