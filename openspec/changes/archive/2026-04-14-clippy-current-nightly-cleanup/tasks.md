# Tasks: Restore Green Clippy Check On Current Nightly

- [x] Capture the current `checks.clippy` failure set from `nix build .#checks.x86_64-linux.clippy -L` (`evidence/clippy-baseline.txt`)
- [x] Separate mechanical style rewrites from policy decisions or intentional patterns (`evidence/triage-notes.md`)
- [x] Fix or intentionally justify the current clippy failures
- [x] Rerun `nix build .#checks.x86_64-linux.clippy -L` (`evidence/clippy-pass.txt`)
- [x] Rerun full `nix flake check -L` (`evidence/flake-check-pass.txt`)
