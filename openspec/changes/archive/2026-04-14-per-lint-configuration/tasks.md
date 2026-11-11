## Phase 1: Config schema extension

- [x] Add `disabled_lints: Vec<String>` to `Config` with `#[serde(default)]` empty default
- [x] Add `lint_levels: HashMap<String, String>` to `Config` with `#[serde(default)]` empty default
- [x] Normalize validated `lint_levels` values into an internal representation used by registration
- [x] Add validation: warn on unknown `disabled_lints` names at load time
- [x] Add validation: warn on unknown `lint_levels` names at load time
- [x] Add validation: warn on invalid `lint_levels` values and ignore the override
- [x] Add validation: warn when a lint appears in both `disabled_lints` and `lint_levels`, then ignore the level override
- [x] Introduce a shared `LINT_INVENTORY` (or equivalent canonical table) that stores every canonical lint name and is the sole source for both config validation and registration wiring
- [x] Eliminate out-of-band lint registration wiring so every shipped lint is declared through `LINT_INVENTORY` and registration iterates only that inventory
- [x] Refactor config validation and registration to derive accepted lint names from that shared inventory rather than separate hand-maintained string lists
- [x] Deterministic check: unit test asserts the shared inventory's lint names are unique and that the lint-name slice passed to config validation is derived from the same inventory used by registration
- [x] Deterministic check: snapshot-test the exact canonical lint-name set exported by `LINT_INVENTORY` so adding or removing a shipped lint requires updating the inventory snapshot in the same change
- [x] Unit test: omitted `disabled_lints` and `lint_levels` deserialize to empty defaults
- [x] Unit test: explicit `disabled_lints` and `lint_levels` entries deserialize successfully
- [x] Unit test: unknown `disabled_lints` name warns without failing config load
- [x] Unit test: mixed `disabled_lints` config ignores an unknown name while a separate valid disabled lint still takes effect
- [x] Unit test: unknown `lint_levels` entry warns without failing config load
- [x] Unit test: mixed `lint_levels` config ignores an unknown name while a separate valid override still takes effect
- [x] Unit test: invalid `lint_levels` value warns and keeps the default lint level
- [x] Unit test: mixed `lint_levels` config ignores an invalid value while a separate valid override still takes effect
- [x] Unit test: overlap between `disabled_lints` and `lint_levels` warns and keeps the lint disabled

## Phase 2: Registration gating

- [x] Refactor `register_lints` to check `disabled_lints` before each registration
- [x] Implement lint-level override during registration
- [x] Integration test: disabled lint produces no diagnostics
- [x] Integration test: disabled lint stays inert even when `lint_levels` tries to raise it
- [x] Integration test: disabled lint stays inert even when source-level attrs try to raise it
- [x] Fixture-local UI test with local `Cargo.toml` (including empty `[workspace]`), `dylint.toml`, and `src/lib.rs`, using unscoped lint names, proving consumer-owned config is discovered from the fixture root
- [x] Keep UI/config verification on the repo custom UI harness / `checks.ui` path; do not rely on `dylint_testing` or rustup-only UI paths
- [x] Integration test: `allow` level suppresses diagnostics
- [x] Integration test: explicit `warn` level emits a warning
- [x] Integration test: default lint level still emits a warning when no override is present
- [x] Deterministic check: snapshot-test each `LINT_INVENTORY` entry's default severity so the no-override path preserves shipped defaults across the full inventory
- [x] Integration test: `deny` level produces an error instead of a warning

## Phase 3: Templates and docs

- [x] Update `templates/dylint.toml` with commented `disabled_lints` and `lint_levels` examples
- [x] Document that omitted `disabled_lints` / `lint_levels` keys default to empty collections
- [x] Document that unknown lint names and invalid level values warn and are ignored
- [x] Document that `disabled_lints` takes precedence over source-level lint attributes
- [x] Update `docs/downstream-rollout.md` to reference per-lint config for phased rollout
- [x] Show how the `docs/downstream-rollout.md` smoke, boundary-first, and inventory-only profiles map to config examples

## Phase 4: Validation

- [x] Save reviewable transcripts for `cargo test --lib` (`openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/cargo-test-lib.txt`) and the targeted custom UI harness run (`openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/cargo-test-ui.txt`)
- [x] Save deterministic evidence naming the per-lint UI fixtures and inventory snapshot contents at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/ui-fixtures-and-snapshots.txt` via `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/check-ui-fixtures-and-snapshots.py`
- [x] Save a deterministic Phase 4 + spec-sync review bundle at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/phase4-and-spec-sync.txt` via `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/check-phase4-and-spec-sync.py`, so archived validation and spec-sync claims do not rely on file existence alone
- [x] Verify config changes in `nix flake check` and the targeted UI fixture gate, after staging any newly added UI fixture paths so the flake source snapshot includes them, and save the transcript at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/nix-flake-check.txt`
- [x] Human/oracle checkpoint: before marking Aspen validation complete, capture the pinned Aspen revision (`909e5e9fa`), the wrapper evidence path (`openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt`), the authoritative repo template path (`templates/tigerstyle-check.sh`, copied into a consumer-local wrapper), and the exact replay command in `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-validation-checkpoint.txt`
- [x] Verification task: exercise the documented `smoke` and `boundary-first` rollout examples against the canonical inventory and save transcripts at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-smoke.txt` and `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-boundary-first.txt`
- [x] Human validation: reconstruct Aspen phase 1 from `openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` at Aspen commit `909e5e9fa`, use wrapper evidence from `openspec/changes/archive/2026-04-13-custom-clippy-lints/evidence/aspen-check-path-and-config.txt` together with `docs/downstream-rollout.md` section `Check-path recipe for nonstandard toolchains` to copy `templates/tigerstyle-check.sh` into that sandbox, apply the inventory-only profile from the same doc, confirm a clean run, and save the replay transcript at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-replay.txt`, the clean-run log at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-inventory-only.txt`, and the applied config snapshot at `openspec/changes/archive/2026-04-14-per-lint-configuration/evidence/aspen-inventory-only.dylint.toml`
