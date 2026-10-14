## Why

Unison abilities make effects visible in types. Slotcar cannot retrofit Rust algebraic effects, but it can present lint output as a reviewable effect surface covering time, failure, allocation, concurrency, IO, remote execution, randomness, and unsafe boundaries.

## What Changes

- Add effect-family metadata to diagnostics and summary output.
- Group existing lints by reviewable effect surface in JSON/SARIF/human summaries.
- Document effect-family coverage and unknown-effect behavior.

## Capabilities

### New Capabilities
- `effect-surface-reporting`: Effect Surface Reporting support for Slotcar.

### Modified Capabilities
- `consumer-integration`: Adds Unison-inspired behavior to the existing Slotcar surface.

## Impact

- **Files**: CLI/docs/specs/tests in the affected domain.
- **APIs**: New or extended command/output fields as specified by the delta spec.
- **Dependencies**: No new runtime dependency without fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax, UCM, or runtime semantics into Rust.
- Do not replace Git/text source with a code database.
- Do not claim semantic equivalence beyond the explicitly recorded hashes and status fields.
