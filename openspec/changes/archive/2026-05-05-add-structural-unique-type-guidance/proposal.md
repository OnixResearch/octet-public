## Why

Unison distinguishes structural and unique types. Tigerstyle can map that distinction to Rust guidance: nominal newtypes for identities, units, secrets, capabilities, and protocol states; plain structural records only where interchangeability is intentional.

## What Changes

- Strengthen newtype guidance around identity and authority.
- Document when structural records/tuples are acceptable.
- Stage future lint candidates around primitive aliases and public primitive parameters.

## Capabilities

### New Capabilities
- `structural-and-unique-type-guidance`: Structural and Unique Type Guidance support for Tigerstyle standards and tooling.

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
