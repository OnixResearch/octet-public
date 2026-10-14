## Why

The repository and first-class cargo subcommand are still named `tigerstyle`, but the project is being renamed to `slotcar`. The implementation needs a reviewable migration plan so public package names, binaries, docs, examples, Nix outputs, and generated evidence change together instead of drifting across consumers.

## What Changes

- Rename the project-facing repository/tool identity from `tigerstyle` to `slotcar` across manifests, Nix outputs, CLI help, docs, templates, examples, and consumer-facing receipts.
- Introduce `cargo slotcar` / `cargo-slotcar` as the canonical tool entry point while keeping any deliberate `cargo tigerstyle` compatibility path explicit and tested during the transition.
- Preserve the underlying Tiger Style coding-principle language where it describes the philosophy, not the project/tool identity.
- Save deterministic rename evidence showing canonical `slotcar` surfaces and any remaining `tigerstyle` strings are categorized as philosophy text or compatibility aliases.

## Non-Goals

- Changing lint semantics, diagnostic precision, severity policy, or OpenSpec domain coverage.
- Removing compatibility aliases without a separate deprecation decision.
- Renaming third-party references to TigerBeetle/Tiger Style when they describe the source philosophy.

## Capabilities

### New Capabilities

- `slotcar-public-identity`: Users can install, invoke, and read docs for the tool under the `slotcar` name.

### Modified Capabilities

- `consumer-integration`: Consumer-facing commands, outputs, and templates identify the tool as `slotcar` while retaining explicit transition compatibility where implemented.
- `documentation`: Docs distinguish canonical product/tool identity (`slotcar`) from retained philosophy references (`Tiger Style`).

## Impact

- **Files / artifacts**: `Cargo.toml`, `cargo-tigerstyle/`, `crates/tigerstyle-standards/`, `flake.nix`, `README.md`, `CHANGELOG.md`, `docs/**`, `templates/**`, `tests/**`, `ui_tests/**`, `openspec/config.yaml`, generated evidence under `openspec/changes/archive/2026-05-15-rename-to-slotcar/evidence/`.
- **Ordering / dependencies**: Rename manifests/Nix outputs before docs and fixtures; update tests after canonical command names settle; categorize remaining `tigerstyle` strings before compatibility decisions.
- **Verification / evidence**: Saved grep/classification transcript for remaining names, CLI/package surface checks for `cargo slotcar`, and focused integration tests proving canonical and compatibility command behavior.
