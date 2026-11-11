# Restore Green Clippy Check On Current Nightly

## Why

After the nix UI split landed, targeted UI and nextest checks passed, but full
`nix flake check` still failed in `checks.clippy` with current-nightly style
findings across existing repo source files. Those failures are unrelated to the
new UI harness, but they still block a fully green flake check.

## What Changes

- inventory the current clippy failures under the pinned nightly toolchain
- decide which suggestions should be accepted versus explicitly documented or
  configured around
- land the cleanup needed to make `checks.clippy` pass again

## Impact

- **Files**: likely existing lint implementation files under `src/`
- **APIs**: no planned public API changes
- **Testing**: rerun `nix build .#checks.x86_64-linux.clippy` and full
  `nix flake check`
