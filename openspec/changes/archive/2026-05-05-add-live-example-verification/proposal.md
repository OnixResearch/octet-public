## Why

Unison live docs keep examples connected to executable code. Tigerstyle should make standards examples fixture-backed so docs claims cannot drift away from lints, templates, or expected diagnostics.

## What Changes

- Add an example manifest tying docs snippets to fixtures and expected output.
- Specify `cargo tigerstyle verify-examples` or equivalent checker.
- Require hashes for source and expected output examples.

## Capabilities

### New Capabilities
- `live-example-verification`: Live Example Verification support for Tigerstyle standards and tooling.

### Modified Capabilities
- `documentation`: Adds Unison-inspired guidance and future implementation scope.

## Impact

- **Files**: Documentation, policy registry entries, CLI/checker code where applicable, fixtures, and OpenSpec evidence.
- **APIs**: Future commands or output fields are specified only where required by this change.
- **Dependencies**: No new dependency is accepted without implementation evidence and fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax or runtime semantics into Rust.
- Do not hard-gate weak static signals before positive and negative fixtures exist.
- Do not weaken existing Tigerstyle FCIS, boundedness, or explicitness rules.
