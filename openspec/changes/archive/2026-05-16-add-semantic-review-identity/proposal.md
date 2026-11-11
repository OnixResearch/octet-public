## Why

Slotcar fingerprints currently prove selected function text, but review state and baselines still depend too heavily on names, paths, and lines. Unison's content-addressed definitions show a better target: stable semantic identity with names treated as metadata.

## What Changes

- Extend fingerprint and finding output contracts with normalized item identity fields.
- Bind review manifests to tool/config/dependency identity instead of only stored body hash.
- Define rename/move-stable baseline matching semantics.

## Capabilities

### New Capabilities
- `semantic-review-identity`: Semantic Review Identity support for Slotcar.

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
