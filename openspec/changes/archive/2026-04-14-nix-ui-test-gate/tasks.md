# Tasks: Prove UI Lint Fixtures Under Nix

## Phase 1: Harness split

- [x] Replace the silent skip in `tests/ui.rs` with a real fixture harness
- [x] Teach the harness to accept prebuilt driver/library paths from nix
- [x] Add a dedicated nix `checks.ui` derivation so UI proof no longer depends
  on `checks.nextest`

## Phase 2: Real fixture coverage

- [x] Add representative UI fixtures for a pre-expansion/syntax lint and a
  late/type-aware lint
- [x] Add repo-local support code for the prebuilt UI driver path
- [x] Refresh README/developer docs so the UI-validation path is explicit

## Phase 3: Validation

- [x] Run targeted UI tests inside the nix devshell (`evidence/ui-devshell.txt`)
- [x] Run `nix build .#checks.x86_64-linux.nextest` (`evidence/ui-nextest.txt`, detailed log in `evidence/ui-nextest-detailed.txt`)
- [x] Run `nix build .#checks.x86_64-linux.ui` (`evidence/ui-check.txt`)
- [x] Prove the nix UI path builds against repo-local Dylint support crates (`evidence/ui-check-local-support.txt`, `evidence/ui-validation-check.txt`)
- [x] Run full `nix flake check` without unrelated failures (`evidence/flake-check-pass.txt`)
