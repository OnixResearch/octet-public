## Why

The roadmap slice is archived and the full flake gate is clean, but the newly added operator surfaces are still easier to discover from tests/specs than from the release-facing README/help path. Before treating the current Slotcar surface as release-ready, operators should have one obvious smoke route that mentions the `fingerprint`, `migrate`, effect-summary, cache, and transcript workflows together.

## What Changes

- Add release-facing README/help discoverability for the current operator commands and outputs.
- Add a small smoke transcript that exercises or demonstrates the release-critical operator surfaces in one place.
- Gate the polish with deterministic docs/transcript verification so the release checklist cannot drift back into prose-only claims.

## Capabilities

### Modified Capabilities
- `documentation`: Adds release/operator discoverability and smoke-transcript expectations to the existing documentation surface.

## Impact

- **Files**: README/help docs, transcript fixtures, and docs/example verification tests.
- **APIs**: No new runtime API is intended; this change documents and smoke-tests existing operator surfaces.
- **Dependencies**: No new runtime dependency expected.
- **Testing**: Scenario rows map to deterministic docs/transcript checks plus one human operator-review checkpoint.

## Non-Goals

- Do not add new lint semantics or new CLI subcommands.
- Do not block release-readiness on broad downstream consumer rollout evidence.
- Do not replace focused smoke coverage with a full end-to-end release process.
