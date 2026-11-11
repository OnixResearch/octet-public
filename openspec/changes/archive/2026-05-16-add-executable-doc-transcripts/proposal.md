## Why

Unison transcripts make docs executable and reproducible. Slotcar examples, rollout snippets, and evidence recipes should similarly run as deterministic transcript fixtures instead of drifting as prose.

## What Changes

- Define Slotcar markdown transcript blocks for commands, fixtures, and expected output assertions.
- Add a transcript runner command surface for docs/examples.
- Require deterministic transcript artifacts with content hashes.

## Capabilities

### New Capabilities
- `executable-doc-transcripts`: Executable Documentation Transcripts support for Slotcar.

### Modified Capabilities
- `documentation`: Adds Unison-inspired behavior to the existing Slotcar surface.

## Impact

- **Files**: CLI/docs/specs/tests in the affected domain.
- **APIs**: New or extended command/output fields as specified by the delta spec.
- **Dependencies**: No new runtime dependency without fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax, UCM, or runtime semantics into Rust.
- Do not replace Git/text source with a code database.
- Do not claim semantic equivalence beyond the explicitly recorded hashes and status fields.
