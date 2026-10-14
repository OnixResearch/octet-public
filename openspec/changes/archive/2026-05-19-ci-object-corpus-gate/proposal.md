## Why

Octet now has checked-in object corpus receipts, replay checks, Rust-derived contract metadata, and a fresh-target determinism rail. Those protections should be available as an explicit CI/check target, not only as ad hoc developer commands.

## What Changes

- Add a named Nix flake check that runs the internal object corpus gate tests.
- Document the check as the CI/operator entry point for object corpus evidence replay.
- Preserve the bounded claim: the check proves local replay/fixture/contract/fresh-target stability for the selected corpus, not correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage.

## Capabilities

### Modified Capabilities
- `standards-codification`: CI-visible validation for function object corpus evidence receipts.

## Impact

- **Files**: `flake.nix`, contributor docs, standards-codification OpenSpec.
- **APIs**: Adds a flake check attribute; no CLI schema change.
- **Testing**: Focused Nix check, full Rust/OpenSpec verification, diff hygiene.
