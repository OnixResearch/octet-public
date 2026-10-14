## Why

The workspace currently builds successfully, but `cargo check --workspace` emits warnings from `cargo-octet/src/evidence_contracts.rs`. Warning-bearing builds make it harder to see new regressions, weaken release-readiness signals, and conflict with Octet's own checked-path story.

A warning-clean baseline lets future changes treat compiler warnings as failures without first paying down unrelated noise.

## What Changes

- Make the workspace warning-clean under the Nix devshell.
- Decide whether currently unused evidence-contract generation code is live product code, test-only code, or dead code, then wire or scope it accordingly.
- Add a focused check rail that treats Rust warnings as regressions for the relevant workspace packages and compatibility binaries.
- Document the warning-clean check in the contributor verification path.

## Impact

- **Files**: `cargo-octet/src/evidence_contracts.rs`, possibly `cargo-octet/src/main.rs` or command wiring, Nix checks in `flake.nix`, and contributor docs.
- **Testing**: `cargo check --workspace`, a warning-deny cargo check rail, and targeted tests for any newly wired evidence-contract command or test-only generator path.
