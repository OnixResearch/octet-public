## Why

Proof receipts now bind pass signals, source hashes, verifier identity, toolchain identity, and Nix closure graph evidence, but those fields are still represented as loose receipt lines and flattened generated config arrays. Downstream proof-bundle consumers need one versioned schema surface that can be emitted, validated, and rejected deterministically before proof receipts are trusted.

## What Changes

- Add a first-class versioned proof-bundle artifact schema to the policy artifact export surface.
- Extend Nickel contracts and Rust models so verified proof rails export structured receipt obligations instead of only flat TOML summary fields.
- Make `tigerstyle-standards check --policy-artifacts-dir ...` validate the proof-bundle schema and reject missing or unsupported schema versions.
- Add compliant and noncompliant fixtures proving artifact-backed proof rails accept complete bundles and reject schema/version drift.

## Capabilities

### Modified Capabilities
- `standards-codification`: generated policy artifacts and verified proof rails now include a versioned proof-bundle schema gate.

## Impact

- **Files**: `standards/policies/*.ncl`, `crates/tigerstyle-standards/src/*`, standards tests/fixtures, generated-artifact docs/evidence.
- **APIs**: Adds a generated proof-bundle artifact file and validation path; existing registry-backed checks continue to work.
- **Testing**: Nickel export/validation, Rust unit and CLI fixture tests, artifact export/check round trips, OpenSpec validation, and whitespace checks.
