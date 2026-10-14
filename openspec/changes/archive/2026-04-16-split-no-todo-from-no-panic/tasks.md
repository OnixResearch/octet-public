## Phase 1: Implementation

- [x] I1 Add `no_todo` as a dedicated safety lint and narrow `no_panic` to `panic!()`, `unimplemented!()`, and `unreachable!()`
- [x] I2 Replace the temporary-stub allow audit with `unjustified_no_todo_allow`, supporting both `allow(tigerstyle::no_todo)` and `allow(no_todo)` while leaving legacy `no_panic` attrs as migration work
- [x] I3 Refresh `src/diagnostics.rs`, lint inventory fixtures, and full metadata snapshot artifacts for `no_todo`, `unjustified_no_todo_allow`, and the narrowed `no_panic`
- [x] I4 Add and refresh ignored UI fixtures/snapshots for `no_todo`, including the required `register_tool` preamble on tool-scoped fixtures and any mixed-phase churn on existing `no_panic*.stderr` artifacts
- [x] I5 Update `README.md`, `docs/downstream-rollout.md`, and `templates/dylint.toml` comments with the `no_todo` migration guidance and explicit breaking-change callout for old `no_panic` stub escapes

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "`todo!()` emits the dedicated lint" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-routing.txt`
- [x] V2 Verify scenario "panic-family macros stay on `no_panic`" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-routing.txt`
- [x] V3 Verify scenario "legacy tool-scoped `no_panic` allow does not hide `todo!()`" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_legacy_no_panic_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-legacy-no-panic.txt`
- [x] V4 Verify scenario "legacy bare `no_panic` allow does not hide `todo!()`" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_legacy_no_panic_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-legacy-no-panic.txt`
- [x] V5 Verify scenario "justified tool-scoped `no_todo` allow stays quiet" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- [x] V6 Verify scenario "justified bare `no_todo` allow stays quiet" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- [x] V7 Verify scenario "unjustified `no_todo` allow warns" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- [x] V8 Verify scenario "orphaned or misapplied `no_todo` allow warns" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_orphan_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- [x] V9 Verify scenario "`cfg_attr`-expanded `no_todo` allow is audited" with shared `cargo test --test ui ui -- --ignored --nocapture` output covering `ui_tests/no_todo_orphan_allow.rs`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- [x] V10 Verify scenario "Maintainer sees the `no_todo` migration path" with `bash -lc 'rg -n "no_todo|unjustified_no_todo_allow|breaking-change|allow\\((tigerstyle::)?no_panic\\)|allow\\((tigerstyle::)?no_todo\\)" README.md docs/downstream-rollout.md templates/dylint.toml'`; save output at `openspec/changes/split-no-todo-from-no-panic/evidence/docs-migration.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`bash -lc 'rg -n "allow\\((tigerstyle::)?no_panic\\)" /path/to/git/aspen'`; input=`aspen@bd486f3dc`; asserts=`saved docs migration note covers the real downstream no_panic source-attr spellings still present in Aspen`; evidence=`openspec/changes/split-no-todo-from-no-panic/evidence/aspen-no-panic-usage.txt`

## Verification Coverage

- `Scenario: `todo!()` emits the dedicated lint` -> `V1` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-routing.txt`
- `Scenario: panic-family macros stay on `no_panic`` -> `V2` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-routing.txt`
- `Scenario: legacy tool-scoped `no_panic` allow does not hide `todo!()`` -> `V3` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-legacy-no-panic.txt`
- `Scenario: legacy bare `no_panic` allow does not hide `todo!()`` -> `V4` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-legacy-no-panic.txt`
- `Scenario: justified tool-scoped `no_todo` allow stays quiet` -> `V5` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- `Scenario: justified bare `no_todo` allow stays quiet` -> `V6` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- `Scenario: unjustified `no_todo` allow warns` -> `V7` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- `Scenario: orphaned or misapplied `no_todo` allow warns` -> `V8` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- `Scenario: `cfg_attr`-expanded `no_todo` allow is audited` -> `V9` -> `openspec/changes/split-no-todo-from-no-panic/evidence/ui-no-todo-allow.txt`
- `Scenario: Maintainer sees the `no_todo` migration path` -> `V10` -> `openspec/changes/split-no-todo-from-no-panic/evidence/docs-migration.txt`
