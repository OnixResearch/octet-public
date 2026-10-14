## Why

Unison user-defined abilities give effects domain names. Tigerstyle should encourage Rust capability traits and tokens such as Clock, Entropy, ObjectStore, CredentialReader, and RemoteExecutor with deterministic test handlers.

## What Changes

- Document capability naming and ownership conventions.
- Define admissible capability traits/tokens for shell-adjacent code.
- Require deterministic fake/recording handlers in examples.

## Capabilities

### New Capabilities
- `capability-vocabulary-guidance`: Capability Vocabulary Guidance support for Tigerstyle standards and tooling.

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
