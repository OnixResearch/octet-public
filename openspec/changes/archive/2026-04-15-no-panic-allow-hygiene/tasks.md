## Phase 1: Escape-hatch hygiene

- [x] Extend `src/safety/no_panic.rs` so `unjustified_no_panic_allow` can detect orphaned direct `#[allow(tigerstyle::no_panic)]` attrs on supported item bodies and block expressions even when no panic-family macro remains
- [x] Add `ui_tests/no_panic_orphan_allow.rs(.stderr)` coverage for orphaned item-level and block-level allows while keeping `ui_tests/no_panic_todo_allow.rs(.stderr)` as the justified-live `todo!()` non-regression fixture
- [x] Run `cargo test --lib`
- [x] Run `cargo test --test ui -- --ignored --nocapture`
