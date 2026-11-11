## Phase 1: Test-context suppression

- [x] Add shared test-context detection helper used by `no_unwrap` and `no_panic`
- [x] Teach `tests/ui.rs` to pass per-fixture `--cfg test` for cfg-gated test-context fixtures
- [x] Keep `tests/ui.rs` compiling standalone fixtures with `--edition=2024` so async precision fixtures match the crate edition
- [x] `no_unwrap`: skip `#[test]` functions
- [x] `no_unwrap`: skip `#[cfg(test)]` modules / `#![cfg(test)]` items
- [x] `no_unwrap`: still warn for `#[cfg(not(test))]` production code
- [x] `no_panic`: skip `#[test]` functions
- [x] `no_panic`: skip `#[cfg(test)]` modules / `#![cfg(test)]` items
- [x] `no_panic`: still warn for `#[cfg(not(test))]` production code
- [x] UI proof: `ui_tests/no_unwrap.rs` shows production warning + test silence
- [x] UI proof: `ui_tests/no_unwrap_cfg_not.rs` shows `#[cfg(not(test))]` still warns
- [x] UI proof: `ui_tests/no_panic.rs` shows production warning + test silence
- [x] UI proof: `ui_tests/no_panic_cfg_not.rs` shows `#[cfg(not(test))]` still warns

## Phase 2: Pattern-based suppression

- [x] `no_unwrap`: skip `.unwrap()` on compile-time-known `Some` / `Ok` and literal-string `.parse().unwrap()`, including const/static-style initializer contexts
- [x] UI proof: `ui_tests/no_unwrap_compile_time_known.rs` stays silent for compile-time-known `Some` / `Ok` and valid literal-string `.parse().unwrap()` cases while equivalent `.expect()`, invalid literal parses, unsupported literal parse targets, and non-literal `.parse().unwrap()` cases still warn
- [x] `no_unwrap`: skip `.unwrap()` after `.is_some()` / `.is_ok()` guard in same branch
- [x] Implementation guardrail: same-branch unwrap suppression uses condition structure (`&&` may carry the guard, `||` only counts when both sides imply it) instead of raw snippet matching
- [x] UI proof: `ui_tests/no_unwrap_guarded.rs` stays silent in the guarded `.unwrap()` branch while guarded `.expect()` and the unguarded `else` path still warn
- [x] `no_panic`: skip `unreachable!()` in exhaustive trailing-wildcard enum matches for local, non-`#[non_exhaustive]` enums
- [x] UI proof: `ui_tests/no_panic_exhaustive.rs` stays silent for exhaustive wildcard `unreachable!()` and still warns for incomplete variant coverage
- [x] UI proof: `ui_tests/no_panic_exhaustive_indirect.rs` shows a nested / non-direct wildcard-arm `unreachable!()` still warns
- [x] UI proof: `ui_tests/no_panic_exhaustive_external.rs` shows an external enum wildcard `unreachable!()` still warns
- [x] UI proof: `ui_tests/no_panic_exhaustive_non_exhaustive.rs` shows a local `#[non_exhaustive]` enum wildcard `unreachable!()` still warns
- [x] `no_panic`: skip `todo!()` when an enclosing function or block has `#[allow(tigerstyle::no_panic)]` plus an adjacent `//` justification comment
- [x] UI/setup proof: `ui_tests/no_panic_todo_allow.rs` covers a tool-registered function-level fixture where the immediately preceding `//` justification suppresses `todo!()`
- [x] UI/setup proof: `ui_tests/no_panic_todo_allow.rs` covers a tool-registered block-scoped fixture where the immediately preceding `//` justification suppresses `todo!()`
- [x] UI/setup proof: `ui_tests/no_panic_todo_allow.rs` covers a same-line `//` justification case that also suppresses `todo!()`
- [x] UI/setup proof: `ui_tests/no_panic_todo_allow.rs` shows blank lines, doc comments, block comments, or missing `//` comments do NOT suppress `todo!()`
- [x] UI/setup proof: `ui_tests/no_panic_todo_allow.rs` shows the same justified allow shape does NOT suppress `panic!()`, `unimplemented!()`, or `unreachable!()`
- [x] `ignored_result`: skip `let _ = expr` inside `Drop::drop` impls
- [x] `ignored_result`: detect real `Drop::drop` via lang-item-backed trait impl matching, not trait-name text
- [x] UI proof: `ui_tests/ignored_result_drop.rs` stays silent in `Drop::drop` and still warns for an inherent `drop` method
- [x] `ignored_result`: skip `let _ = tx.send(msg);` only for `std::sync::mpsc::Sender<_>` / `tokio::sync::mpsc::Sender<_>` in functions or methods named exactly `shutdown` or `close`
- [x] Implementation guardrail: match Tokio sender / receiver types by concrete definition path, not by the public re-export spelling alone
- [x] UI proof: `ui_tests/ignored_result_shutdown_send.rs` shows `std::sync::mpsc::Sender<_>` inside a free `shutdown` function stays silent
- [x] UI proof: `ui_tests/ignored_result_shutdown_send.rs` shows a supported sender inside an impl `close` method stays silent
- [x] UI proof: `ui_tests/ignored_result_shutdown_send.rs` shows an unsupported sender type in `shutdown` / `close` still warns
- [x] Implementation guardrail: async `ignored_result` teardown suppression unwraps await-desugared `send(...).await` before checking sender type and enclosing name
- [x] UI proof: `ui_tests/tokio_async/ignored_result_shutdown_send_tokio.rs` shows `tokio::sync::mpsc::Sender<_>` inside `close` and `shutdown` stays silent
- [x] UI proof: `ui_tests/tokio_async/ignored_result_try_send_tokio.rs` shows an exact `shutdown` / `close` function still warns when the supported sender uses a method other than exact `send` (`try_send`)
- [x] UI proof: `ui_tests/ignored_result.rs` keeps ordinary ignored-`Result` warnings and their existing `let _: () = expr?;` / explicit-error-handling help text
- [x] UI proof: `ui_tests/ignored_result_shutdown_send.rs` shows inexact names or ordinary `publish`-style sends still warn

## Phase 3: Bounds-check recognition

- [x] `unchecked_narrowing`: recognize `x.min(T::MAX as u64) as T`
- [x] UI proof: `ui_tests/unchecked_narrowing_min.rs` stays silent for `min(... ) as T` and still warns for a plain unchecked cast
- [x] `unchecked_narrowing`: recognize `T::try_from(x).unwrap_or(T::MAX)`
- [x] UI proof: `ui_tests/unchecked_narrowing_try_from.rs` stays silent for `try_from(...).unwrap_or(...)` and still warns for a plain unchecked cast
- [x] `unchecked_narrowing`: recognize an earlier same-block `assert!(value <= T::MAX as WIDE)` before `value as T`
- [x] UI proof: `ui_tests/unchecked_narrowing_same_block_assert.rs` stays silent for a matching earlier assert and still warns when the earlier assert is about a different value, a shadowed binding, the wrong bound shape, or appears later in the block
- [x] Async UI setup: `tests/ui.rs` now builds fixture-local Cargo/Tokio support for async fixtures without dropping `--edition=2024`
- [x] Async UI verification: `cargo test --test ui -- --ignored --nocapture` proves the Tokio/Cargo fixture path in `tests/ui.rs` still compiles with `--edition=2024`
- [x] `unbounded_loop`: recognize `loop { tokio::select! { ... } }` as bounded
- [x] Implementation guardrail: recover Tokio `select!` macro identity from `expr.span.source_callsite()` plus macro backtrace inspection rather than relying on pre-expansion registration
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_tokio_select.rs` covers fully qualified `tokio::select!` loops
- [x] `unbounded_loop`: recognize imported Tokio `select!` loops as bounded
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_imported_select.rs` covers imported Tokio `select!` loops
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_bare_async.rs` keeps warning without either Tokio `select!` form
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_plain_await.rs` shows plain `.await` outside a user-written loop stays silent
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_non_tokio_select.rs` shows a non-Tokio macro named `select!` still warns
- [x] `unbounded_loop`: recognize `while let Some(_) = rx.recv().await` only for `tokio::sync::mpsc::Receiver<_>`
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_bounded_receiver.rs` covers bounded `tokio::sync::mpsc::Receiver<_>`
- [x] UI proof: `ui_tests/tokio_async/unbounded_loop_unbounded_receiver.rs` distinguishes `UnboundedReceiver<_>` and still warns
- [x] Regression proof: `ui_tests/unbounded_loop.rs` keeps existing visible-counter and counter-break bounded loops silent while the bare loop still warns
- [x] UI proof: `ui_tests/unbounded_loop_iter_next.rs` keeps `while let Some(_) = iter.next()` warning when there is no limit check in the body

## Phase 4: Downstream validation

- [x] Re-run against Aspen and record before/after finding counts
- [x] Save reviewer-visible Aspen comparison artifact at `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/aspen-comparison.md` with before/after counts and short notes per affected lint family
- [x] Save ignored UI harness transcript at `openspec/changes/archive/2026-04-15-lint-precision-hardening/evidence/ui-ignored-test-run.txt`, including `[ui fixture] unbounded_loop_iter_next` and the passing `cargo test --test ui -- --ignored --nocapture` result
- [x] Confirm no real findings were lost in the precision changes (`aspen-comparison.md`: no owned-package count deltas; supplementary workspace-only `unbounded_loop +8` limited to vendored `openraft/*`)
- [x] Accept the supplementary workspace-only `openraft/*` `unbounded_loop +8` as non-blocking context because the completion gate for this change is the owned Aspen rollout scope, not vendored workspace members outside consumer-owned code
- [x] Review `docs/downstream-rollout.md` noise-level notes and keep them unchanged because this rerun did not make any additional family promotable
