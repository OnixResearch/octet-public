## Why

Unison treats distributed execution as an explicit Remote ability. Tigerstyle should classify remote execution as a first-class effect requiring bounds, idempotency, retries, timeouts, receipts, and replay semantics.

## What Changes

- Add remote execution as an effect family.
- Require timeout/retry/idempotency/batch/receipt fields in guidance.
- Keep enforcement checklist-first until static signal is proven.

## Capabilities

### New Capabilities
- `remote-effect-contracts`: Remote Effect Contracts support for Tigerstyle standards and tooling.

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
