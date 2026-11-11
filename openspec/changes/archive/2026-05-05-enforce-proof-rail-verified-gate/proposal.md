## Why

Tigerstyle can already describe verified-mode proof rails in Nickel, but the Rust standards checker does not yet reject missing local proof artifacts when a policy targets verified enforcement. That leaves verified mode as metadata instead of a hard gate.

## What Changes

- Add an artifact-backed verified proof-rail checker path in `tigerstyle-standards`.
- Export proof-rail obligations into generated check config summaries.
- Add Tigerstyle-local compliant and noncompliant verification fixtures.
- Promote the deterministic effect-handler policy into a real verified-mode gate backed by local source/proof receipt evidence.

## Impact

- **Files**: `crates/tigerstyle-standards`, `standards/policies`, `standards/check-fixtures/verification`, `flake.nix`, and this OpenSpec package.
- **APIs**: no public CLI flag change; existing `check --policy-artifacts-dir` consumes proof rails from generated artifacts.
- **Testing**: focused Rust tests, policy validation, standards fixture Nix check, artifact export assertions, and OpenSpec validation.
