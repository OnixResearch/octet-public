# Prove UI Lint Fixtures Under Nix

## Why

`tests/ui.rs` currently skips when `rustup` is absent. In this repo that means
`nix flake check` can stay green without exercising any Dylint UI fixture at
all. That leaves a blind spot exactly on the nix-managed toolchain path we
publish and use for CI.

## What Changes

- replace the skip-only UI path with a repo-owned UI harness plus checked-in
  fixtures
- split UI proof out of `checks.nextest` into a dedicated nix `checks.ui`
  derivation that injects a prebuilt driver and lint library
- keep local developer runs available through
  `cargo test --test ui -- --ignored --nocapture`
- refresh developer docs so the nix validation path is explicit

## Capabilities

### Modified Capabilities

- `nix-validation`: `nix flake check` now exercises real Dylint UI fixtures
  instead of accepting a skipped test

## Impact

- **Files**: `flake.nix`, `tests/ui.rs`, `ui_tests/`, `README.md`, OpenSpec
  docs/specs
- **APIs**: no public Rust API changes
- **Dependencies**: no new published crates; add repo-local UI driver support under `tests/support/`
- **Testing**: run targeted UI tests in nix devshell plus dedicated nix `checks.ui` and `checks.nextest`
