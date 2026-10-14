## Why

Unison code identity survives renames and moves. Tigerstyle should support refactor evidence that maps symbols across file moves or renames using symbol paths, fingerprints, and old/new identity records.

## What Changes

- Define rename/move evidence records.
- Tie refactor records to function/finding fingerprints where available.
- Support review output showing unchanged identity across path changes.

## Capabilities

### New Capabilities
- `semantic-refactor-evidence`: Semantic Refactor Evidence support for Tigerstyle standards and tooling.

### Modified Capabilities
- `consumer-integration`: Adds Unison-inspired guidance and future implementation scope.

## Impact

- **Files**: Documentation, policy registry entries, CLI/checker code where applicable, fixtures, and OpenSpec evidence.
- **APIs**: Future commands or output fields are specified only where required by this change.
- **Dependencies**: No new dependency is accepted without implementation evidence and fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax or runtime semantics into Rust.
- Do not hard-gate weak static signals before positive and negative fixtures exist.
- Do not weaken existing Tigerstyle FCIS, boundedness, or explicitness rules.
