## Why

Unison avoids repeated work by caching immutable content-addressed definitions. Slotcar can use the same principle narrowly: cache deterministic lint results by tool/config/item identity so large consumers rerun only changed work.

## What Changes

- Specify cache keys from Slotcar version, rustc/toolchain, config hash, crate graph hash, and item identity.
- Define cache hit/miss reporting and invalidation behavior.
- Constrain the cache to deterministic findings only.

## Capabilities

### New Capabilities
- `content-identity-lint-cache`: Content Identity Lint Cache support for Slotcar.

### Modified Capabilities
- `performance`: Adds Unison-inspired behavior to the existing Slotcar surface.

## Impact

- **Files**: CLI/docs/specs/tests in the affected domain.
- **APIs**: New or extended command/output fields as specified by the delta spec.
- **Dependencies**: No new runtime dependency without fixture-backed need.
- **Testing**: Each scenario has a deterministic verification task and saved evidence path.

## Non-Goals

- Do not copy Unison syntax, UCM, or runtime semantics into Rust.
- Do not replace Git/text source with a code database.
- Do not claim semantic equivalence beyond the explicitly recorded hashes and status fields.
