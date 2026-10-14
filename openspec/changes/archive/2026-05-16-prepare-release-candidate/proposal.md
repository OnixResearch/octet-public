## Why

Slotcar now has clean full-gate, archived roadmap, operator help, and a release smoke transcript. The next release-risk gap is not feature behavior; it is whether a maintainer can cut a candidate from the current tree with deterministic package outputs, version visibility, release notes, and a saved install-smoke receipt.

## What Changes

- Add a release candidate checklist/receipt document that records version, package outputs, gates, and smoke commands.
- Add CLI version output so release smoke can assert the shipped binary identity without parsing Cargo metadata by hand.
- Add a Nix check that verifies release candidate docs and installed package smoke paths for `cargo-slotcar`, `cargo-tigerstyle`, and the lint library package.

## Capabilities

### New Capabilities
- `release-candidate-readiness`: Maintainers can verify the exact candidate package outputs and release note checklist before tagging.

### Modified Capabilities
- `documentation`: Release-facing docs include a deterministic candidate receipt path.
- `consumer-integration`: The installed Cargo subcommand surface exposes version output for smoke verification.

## Impact

- **Files**: `cargo-slotcar/src/cli.rs`, `cargo-slotcar/src/main.rs`, `README.md`, `docs/release-candidate.md`, `flake.nix`, OpenSpec artifacts.
- **APIs**: Adds `--version` / `version` CLI handling for the cargo-slotcar binary aliases.
- **Dependencies**: No new runtime dependencies.
- **Testing**: `nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link`, `openspec validate --all --strict`, and full `nix flake check -L`.
