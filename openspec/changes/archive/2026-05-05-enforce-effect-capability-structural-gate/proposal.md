## Why

Tigerstyle already describes effect-capability boundaries and the architecture checker has a narrow hard-coded scan for ambient operations in `crates/tigerstyle-core`. The next Tigerstyle-only enforcement step is to move that policy into the same generated artifact seam used by bounded queues, so construction-mechanism obligations are typed in Nickel, exported deterministically, and consumed by Rust checkers without downstream Aspen proof work.

## What Changes

- Promote `tigerstyle.architecture.effect-capability-boundaries` from advisory lint metadata toward an artifact-backed structural gate for architecture-family checks.
- Extend the capability-api mechanism contract with exported obligations for core source scopes, forbidden ambient authority constructors, approved injected capability names, and shell-owned constructor scopes.
- Make `tigerstyle-standards check --policy-artifacts-dir` reject fixture core code that reads ambient IO/time/env/network/process authority directly, while accepting injected data/capability traits and shell-owned construction.
- Keep this slice Tigerstyle-local: standards fixtures and artifact checks prove behavior; no Aspen mutation or downstream lock update is in scope.

## Capabilities

### Modified Capabilities
- `standards-codification`: effect capability boundary policy becomes a generated-artifact-backed structural checker with positive and negative local fixtures.

## Impact

- **Files**: `standards/policies/contracts.ncl`, `standards/policies/registry.ncl`, `crates/tigerstyle-standards/src/model.rs`, `crates/tigerstyle-standards/src/artifacts.rs`, `crates/tigerstyle-standards/src/architecture.rs`, `standards/check-fixtures/architecture/**`, `flake.nix`, docs/evidence.
- **APIs**: no public cargo-tigerstyle lint API change; `tigerstyle-standards` artifact JSON gains capability-api obligation fields.
- **Testing**: Nickel fixture/export assertions, architecture compliant/noncompliant fixtures, policy-artifact consumer check, focused Rust tests, standards fixtures, docs examples, strict OpenSpec validation.
