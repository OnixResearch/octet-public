## Why

Unison abilities make computational effects explicit in type signatures. Tigerstyle can import the discipline without recreating algebraic effects by requiring Rust core logic to expose effects as data or named capabilities and keeping shells as interpreters.

## What Changes

- Document Rust effect sets for IO, clock, randomness, env, process, spawn, network, storage, and remote work.
- Define pure-core admissibility rules for effect descriptions and capability traits.
- Classify first enforcement as docs/policy with future lint candidates for obvious ambient effects.

## Capabilities

### New Capabilities
- `effect-capability-boundaries`: Effect Capability Boundaries support for Tigerstyle standards and tooling.

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
