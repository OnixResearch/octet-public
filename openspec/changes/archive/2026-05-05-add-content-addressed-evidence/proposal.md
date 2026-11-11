## Why

Function fingerprints bind individual functions, but Tigerstyle also needs stable identity for findings, snippets, configs, commands, fixtures, and transcripts so evidence survives line moves and review artifacts prove exact inputs.

## What Changes

- Add hashes for finding snippets, config, commands, tool revisions, and fixtures.
- Define evidence headers for saved transcripts.
- Support baseline entries by stable finding fingerprint.

## Capabilities

### New Capabilities
- `content-addressed-evidence`: Content Addressed Evidence support for Tigerstyle standards and tooling.

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
