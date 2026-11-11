## Why

The standards/checker work moved from a standalone legacy repo into `tigerstyle`, and the old project naming no longer matches the product boundary. The repo now needs one OpenSpec change whose names, paths, commands, and tasks match the combined Tiger Style project.

The goal is to keep the seams explicit: Dylint lints stay in `src/` and `cargo-tigerstyle/`; standards policy, source-shape checks, and traceability live in `standards/`, `docs/standards/`, and `crates/tigerstyle-standards/`.

## What Changes

- Rename the standards codification change to `codify-tigerstyle-standards` and use `tigerstyle.*` policy IDs.
- Move source traceability from the legacy source document and section directory to `docs/standards/source/*.adoc`.
- Add `tigerstyle-standards` as the standards checker command with a pure core and thin CLI shell.
- Add Nix apps/checks for policy export, policy validation, and standards fixture validation.
- Replace the old safety-contract filename with `docs/standards/policy-contract.md`.

## Non-Goals

- Do not merge Dylint lint implementation and standards policy checking into one crate.
- Do not claim all future rule families, fitness reports, safety-evidence prompts, or final single-command verification are complete in this change wave.
- Do not keep legacy project names in Tiger Style repo artifacts.

## Capabilities

### New Capabilities

- `standards-policy-registry`: Machine-readable Tiger Style rule catalog with source traceability and enforcement metadata.
- `standards-check-runner`: Reusable standards checker command for static checks, fixture validation, and CI integration.
- `standards-rule-families`: Codified rule families covering boundaries, resources, suppressions, and planned future checks.

### Modified Capabilities

- `documentation`: README and contributor docs name the standards checker seam and command surfaces.
- `verification-rails`: Flake checks include policy registry validation and standards fixtures alongside existing lint/UI checks.

## Impact

- **Files / artifacts**: `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/**`, `crates/tigerstyle-standards/**`, `standards/**`, `docs/standards/**`, `README.md`, `docs/contributor-guide.md`, `Cargo.toml`, `Cargo.lock`, `flake.nix`.
- **Ordering / dependencies**: Standards source paths, Nickel policy IDs, Rust checker constants, fixtures, and Nix command names must stay renamed in lockstep.
- **Verification / evidence**: Saved evidence under `openspec/changes/archive/2026-04-25-codify-tigerstyle-standards/evidence/` covers rename hygiene, policy validation, standards tests, fixture flake checks, and command-surface exposure.
