## Why

`slotcar` successfully communicates constrained motion, but the project is growing beyond a lint runner toward a Rust fork / language distribution with explicit composition, capability, and effect-boundary rules. `Octet` preserves the same philosophy at language scale through the stable-shell metaphor: code is inert by default, and interaction requires complete, explicit interfaces.

## What Changes

- Rename the canonical project/toolchain identity from `slotcar` to `octet` across public docs, package surfaces, command help, Nix outputs, templates, generated receipts, and OpenSpec metadata.
- Define `Octet` as the language/distribution identity: Rust-compatible systems code with stable closed interfaces for effects, capabilities, APIs, proofs, and review surfaces.
- Keep `Tiger Style` references where they describe the source philosophy, but stop using `tigerstyle`/`slotcar` as canonical product identity except for deliberate compatibility aliases during migration.
- Make compatibility explicit: existing `cargo slotcar` and `cargo tigerstyle` paths may remain only as transition aliases that route to the canonical `cargo octet` behavior and are documented/tested.
- Save deterministic rename evidence that classifies remaining `slotcar` and `tigerstyle` occurrences as philosophy, compatibility, namespace, archived evidence, or needs-fix.

## Non-Goals

- Changing lint semantics, policy severity, diagnostics, or enforcement scope as part of the rename.
- Removing compatibility aliases in the same change without a separate deprecation decision.
- Renaming archived historical evidence except where an active/current spec must stop pointing at dead active paths.
- Choosing a final upstream repository URL or publishing/tagging a release.

## Capabilities

### New Capabilities

- `octet-public-identity`: users can install, invoke, configure, and read docs for the project under the Octet name.
- `octet-composition-model`: docs explain the chemistry metaphor as stable shells / inert-by-default interfaces rather than a cosmetic rename.

### Modified Capabilities

- `consumer-integration`: command names, config sections, outputs, artifacts, and compatibility aliases distinguish canonical Octet surfaces from transition-only legacy names.
- `documentation`: current user-facing docs present `Octet` as the project/toolchain identity while preserving explicit Tiger Style philosophy references.
- `standards-codification`: policy registry and generated artifacts use Octet identity for current tool surfaces while retaining stable policy IDs only when compatibility requires it.

## Impact

- **Files / artifacts**: `Cargo.toml`, `cargo-slotcar/` or successor runner path, `crates/slotcar-standards/` or successor standards path, `flake.nix`, `README.md`, `CHANGELOG.md`, `docs/**`, `templates/**`, `tests/**`, `ui_tests/**`, `standards/policies/**`, `openspec/config.yaml`, generated examples and rename evidence under `openspec/changes/archive/2026-05-18-rename-slotcar-to-octet/evidence/`.
- **Ordering / dependencies**: settle the canonical naming map before broad mechanical renames; update manifests/Nix outputs before docs and tests; update generated Nickel-derived artifacts after `.ncl` changes; classify remaining legacy strings before claiming completion.
- **Verification / evidence**: focused cargo metadata/build smoke for renamed manifests, command/help/output checks for `cargo octet`, policy artifact export/validation if Nickel changes, deterministic remaining-name classification, and doc/example checks that prove the Octet stable-shell metaphor and compatibility boundaries are visible.
