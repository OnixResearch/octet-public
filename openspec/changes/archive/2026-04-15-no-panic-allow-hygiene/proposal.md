## Why

`unjustified_no_panic_allow` now preserves the `no_panic` escape-hatch contract
for justified `todo!()` stubs, but it only fires when a panic-family macro is
still present under the allow. After a cleanup removes the panic macro, an
orphaned `#[allow(tigerstyle::no_panic)]` can remain in production code with no
warning.

That leaves dead panic escapes behind and weakens review of the migration's new
companion-lint path.

## What Changes

- Audit `#[allow(tigerstyle::no_panic)]` even when no panic-family macro remains
  under the annotated item or block
- Warn on orphaned panic escapes so dead `allow` attrs do not accumulate after
  cleanup
- Add UI proof for orphaned item-level and block-level escapes

## Non-Goals

- Expanding the hygiene audit to bare compatibility attrs like `#[allow(no_panic)]`
- Diagnosing overly broad but still-live nested or container-level panic escapes
- Auditing inactive `cfg_attr(...)` branches or non-executable container attrs on
  `mod`, `impl`, or trait declarations

## Capabilities

### New Capabilities
- `no-panic-allow-hygiene`: detect orphaned `#[allow(tigerstyle::no_panic)]`
  after the panic-family code it once covered is gone

## Impact

- **Files**: `src/safety/no_panic.rs`,
  `openspec/changes/no-panic-allow-hygiene/design.md`,
  `ui_tests/no_panic_todo_allow.rs`, new `ui_tests/no_panic_orphan_allow.rs`,
  `openspec/specs/safety/spec.md`
- **APIs**: no new public lint names; tighter auditing of the existing escape
  hatch
- **Testing**: keep `ui_tests/no_panic_todo_allow.rs` as the justified-live
  regression fixture, add `ui_tests/no_panic_orphan_allow.rs` for orphaned
  item/block coverage, then run `cargo test --lib` and
  `cargo test --test ui -- --ignored --nocapture`
