## Why

`no_panic` now runs as a true tool lint, but the documented temporary-stub
workflow still depends on `#[allow(tigerstyle::no_panic)]` around `todo!()`.
That contract fights rustc: once the attr suppresses `no_panic`, tigerstyle has
to recover the intended policy through a separate audit lint.

The repo already proved that the current workaround is awkward, and the main
safety spec now carries a follow-on note that a redesign is still needed before
full tool-lint migration is clean. Keeping `todo!()` inside `no_panic` also
forces downstream docs to describe a stub escape hatch in terms of the wrong
lint name.

Splitting `todo!()` into its own lint gives the escape hatch an honest target,
keeps `no_panic` focused on real panic-family aborts, and removes the biggest
semantic blocker on finishing the consumer-facing tool-lint/group-attr story.

## What Changes

- Add a dedicated safety lint, `no_todo`, for `todo!()` in production code
- Narrow `no_panic` so it covers only `panic!()`, `unimplemented!()`, and
  `unreachable!()`
- Move the documented temporary-stub escape hatch onto
  `#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` with a companion audit
  lint, `unjustified_no_todo_allow`
- Update rollout/docs/metadata contracts so downstream maintainers know how to
  migrate existing justified `no_panic` stub escapes, including bare source
  attrs such as `#![allow(no_panic)]` seen in Aspen
- Treat the saved Aspen usage inventory as required migration evidence, not a
  best-effort follow-up

## Non-Goals

- Changing the `no_panic` contract for `panic!()`, `unimplemented!()`, or
  `unreachable!()` beyond removing `todo!()` from that lint
- Broadening the temporary-stub escape hatch to container-level attrs or
  undocumented spellings
- Providing an automatic downstream source rewrite in the same change
- Reclassifying starter-profile rollout lints beyond the `no_todo` split
- Adding structured autofix suggestions for the new lint pair in this change

## Capabilities

### Modified Capabilities
- `safety`: split `todo!()` enforcement from panic-family enforcement and move
  the temporary-stub escape hatch onto a dedicated lint contract
- `documentation`: update lint inventory, rollout guidance, and migration notes
  for the new `no_todo` / `unjustified_no_todo_allow` pair and the narrowed
  `no_panic` scope

## Impact

- **Files**: `src/safety/no_panic.rs`, new `src/safety/no_todo.rs`, `src/lib.rs`,
  `src/diagnostics.rs`, `README.md`, `docs/downstream-rollout.md`,
  `templates/dylint.toml`, `ui_tests/no_panic*.rs`, new `ui_tests/no_todo*.rs`,
  fixture metadata snapshots, and the safety/documentation specs
- **APIs**: new consumer-facing lint `no_todo`; new companion lint
  `unjustified_no_todo_allow`; `no_panic` no longer covers `todo!()`; the
  supported temporary-stub attr changes from `allow(no_panic)` /
  `allow(tigerstyle::no_panic)` to the matching `no_todo` form, with both bare
  and tool-scoped `no_todo` attrs supported during the documented migration
  path
- **Dependencies**: none
- **Testing**: update unit tests; refresh ignored UI fixtures/snapshots for
  direct `no_todo`, narrowed `no_panic`, legacy `no_panic` escape spellings,
  justified bare/tool-scoped `no_todo` allows, and the three distinct
  `unjustified_no_todo_allow` firing cases (missing comment, wrong macro,
  orphaned attr / `cfg_attr` expansion); prove the `cfg_attr` path through the
  existing expanded late-pass/source-callsite recovery path rather than a new
  pre-expansion pass; refresh diagnostic metadata snapshots; and save a docs
  grep artifact plus a downstream usage-inventory artifact proving the
  migration note matches real consumer spellings and carries an explicit
  breaking-change callout
