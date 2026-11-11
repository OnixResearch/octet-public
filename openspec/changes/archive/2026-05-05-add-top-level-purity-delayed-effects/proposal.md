## Why

Unison requires top-level definitions to be pure unless effectful computation is delayed. Tigerstyle should prevent Rust globals, constructors, Default impls, and builders from hiding IO/time/env/network/spawn effects.

## What Changes

- Document top-level purity expectations.
- Define delayed effect construction patterns for Rust.
- Stage lint candidates for LazyLock, OnceLock, new/default/build effect calls.

## Capabilities

### New Capabilities
- `top-level-purity-and-delayed-effects`: Top Level Purity and Delayed Effects support for Tigerstyle standards and tooling.

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
