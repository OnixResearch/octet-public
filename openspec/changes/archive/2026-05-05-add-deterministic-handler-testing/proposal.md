## Why

Unison ability handlers make effects testable. Tigerstyle should require deterministic handlers for clock, random, store, remote, and error-path behavior so invalid paths and cleanup can be verified without ambient services.

## What Changes

- Add deterministic handler testing guidance.
- Require invalid-path and cleanup evidence for effect handlers.
- Define fixture patterns for recording/failing handlers.

## Capabilities

### New Capabilities
- `deterministic-handler-testing`: Deterministic Handler Testing support for Tigerstyle standards and tooling.

### Modified Capabilities
- `standards-codification`: Adds Unison-inspired guidance and future implementation scope.

## Impact

- **Files**: Documentation, policy registry entries, CLI/checker code where applicable, fixtures, and OpenSpec evidence.
- **APIs**: Future commands or output fields are specified only where required by this change.
- **Dependencies**: No new dependency is accepted without implementation evidence and fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax or runtime semantics into Rust.
- Do not hard-gate weak static signals before positive and negative fixtures exist.
- Do not weaken existing Tigerstyle FCIS, boundedness, or explicitness rules.
