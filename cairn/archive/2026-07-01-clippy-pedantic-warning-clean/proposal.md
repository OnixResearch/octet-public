## Why

Octet now has a warning-clean release-readiness rail, but the existing Clippy check only denies the default lint set. Pedantic Clippy catches additional mechanical maintainability issues before they become review noise, and the current baseline already shows actionable owned-code findings rather than policy disputes.

Adding pedantic coverage makes the warning-clean story stricter while keeping the check local to owned workspace code.

## What Changes

- Make the flake Clippy rail run with `clippy::pedantic` enabled and warnings denied.
- Fix the current owned-code pedantic findings instead of suppressing them broadly.
- Document that release-readiness Clippy coverage includes pedantic lints.
- Keep the warning-clean cargo-check rail separate from the Clippy rail so compiler-warning and Clippy-warning failures stay diagnosable.

## Impact

- **Files**: `flake.nix`, `docs/contributor-guide.md`, Rust source files with current pedantic findings, and `cairn/specs/build-hygiene/spec.md`.
- **Testing**: `nix develop -c cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`, `nix build .#checks.x86_64-linux.clippy --no-link -L`, targeted unit checks for touched logic, and Cairn validation/gates.
