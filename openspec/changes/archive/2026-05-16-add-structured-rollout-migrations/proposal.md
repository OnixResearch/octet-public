## Why

Downstream Slotcar rollouts still turn into ad hoc warning triage. Unison's structured refactoring sessions suggest a better migration state model: keep the consumer workspace usable while Slotcar tracks remaining work as deterministic migration tasks.

## What Changes

- Add a migration session command surface for starter/profile rollouts.
- Persist deterministic migration manifests with accepted, fixed, deferred, and manual-review findings.
- Expose status/apply/finish phases for automation and human review.

## Capabilities

### New Capabilities
- `structured-rollout-migrations`: Structured Rollout Migration Sessions support for Slotcar.

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
