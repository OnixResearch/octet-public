# Verification: per-lint configuration

## Changed files

- `src/config.rs`
- `src/lib.rs`
- `tests/ui.rs`
- `tests/fixtures/lint_inventory_names.txt`
- `tests/fixtures/lint_inventory_default_levels.txt`
- `ui_tests/no_unwrap_allow_custom/`
- `ui_tests/no_unwrap_warn_custom/`
- `ui_tests/no_unwrap_deny_custom/`
- `ui_tests/no_unwrap_default_custom/`
- `ui_tests/no_unwrap_disabled_deny_custom/`
- `ui_tests/no_unwrap_disabled_source_attr_custom/`
- `templates/dylint.toml`
- `docs/downstream-rollout.md`
- `flake.nix`
- `openspec/changes/archive/2026-04-14-per-lint-configuration/proposal.md`
- `openspec/changes/archive/2026-04-14-per-lint-configuration/design.md`
- `openspec/changes/archive/2026-04-14-per-lint-configuration/tasks.md`

## Automated evidence

- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/cargo-test-lib.txt`
  - command: `nix develop -c bash -lc 'export RUSTUP_TOOLCHAIN=nightly; export CARGO_TARGET_DIR="$(mktemp -d /tmp/tigerstyle-lib-levels.XXXXXX)"; trap '\''rm -rf "$CARGO_TARGET_DIR"'\'' EXIT; cargo test --lib'`
  - result: 17 lib tests passed
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/cargo-test-ui.txt`
  - command: `nix develop -c bash -lc 'export RUSTUP_TOOLCHAIN=nightly; export CARGO_TARGET_DIR="$(mktemp -d /tmp/tigerstyle-ui-levels.XXXXXX)"; trap '\''rm -rf "$CARGO_TARGET_DIR"'\'' EXIT; cargo test --test ui -- --ignored --nocapture'`
  - result: custom UI harness passed and prints each fixture name during the run
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/ui-fixtures-and-snapshots.txt`
  - command: `python3 openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/check-ui-fixtures-and-snapshots.py`
  - result: names the exact per-lint UI fixtures wired in `tests/ui.rs` and prints the full contents of `tests/fixtures/lint_inventory_names.txt` and `tests/fixtures/lint_inventory_default_levels.txt`
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/nix-flake-check.txt`
  - command: staged snapshot replay of `nix flake check -L`
  - result: `all checks passed!`

## Downstream replay evidence

- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-validation-checkpoint.txt`
  - records pinned Aspen revision `909e5e9fa`, the canonical phase-1 archive path `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/`, wrapper evidence path, authoritative template path, and exact replay command
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-replay.txt`
  - exact replay command plus sandbox/worktree setup transcript, including explicit archive/rollout-notes/counts references used for the phase-1 reconstruction claim
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-boundary-first.txt`
  - boundary-first replay using the copied `templates/tigerstyle-check.sh` and per-lint disabled-lints config
  - result: ambient-clock findings reproduced, no `no_unwrap` bleed into the starter path
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-smoke.txt`
  - smoke replay scoped to `aspen-time`
  - result: ambient-clock findings reproduced on one owned crate
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-inventory-only.txt`
  - inventory-only replay with only `no_unwrap` enabled from per-lint config
  - transcript header and wrapper banner both identify the inventory-only `no_unwrap` profile
  - result: clean run on the pinned phase-1 default scope
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-inventory-only.dylint.toml`
  - saved config snapshot for the inventory-only replay
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/check-ui-fixtures-and-snapshots.py`
  - deterministic helper that asserts the per-lint UI fixtures are present in `tests/ui.rs` and emits the snapshot contents used by review
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/check-phase4-and-spec-sync.py`
  - deterministic helper that asserts the archived Phase 4 evidence files contain the expected results and that the synced per-lint requirement block is present in `openspec/specs/documentation/spec.md`
- `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/phase4-and-spec-sync.txt`
  - review bundle that prints the `cargo test --lib` result, `nix flake check` summary, Aspen replay/checkpoint summaries, and the synced main-spec requirement block

## Task-to-evidence map

- Phase 1 schema and validation tasks: `src/config.rs`, `src/lib.rs`, `tests/fixtures/lint_inventory_names.txt`, `tests/fixtures/lint_inventory_default_levels.txt`, `cargo-test-lib.txt`, `ui-fixtures-and-snapshots.txt`
- Phase 2 registration and UI behavior tasks: `tests/ui.rs`, `ui_tests/no_unwrap_*`, `cargo-test-ui.txt`, `ui-fixtures-and-snapshots.txt`, `nix-flake-check.txt`
- Phase 3 template/docs tasks: `templates/dylint.toml`, `docs/downstream-rollout.md`
- Phase 4 external validation tasks: `nix-flake-check.txt`, `aspen-validation-checkpoint.txt`, `aspen-replay.txt`, `aspen-smoke.txt`, `aspen-boundary-first.txt`, `aspen-inventory-only.txt`, `aspen-inventory-only.dylint.toml`, `phase4-and-spec-sync.txt`
