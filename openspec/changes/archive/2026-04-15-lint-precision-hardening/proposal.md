## Why

The downstream rollout playbook explicitly names five lint families as
inventory-first because they produce too many false positives for blind
rollout: `ignored_result`, `no_unwrap`, `no_panic`, `unchecked_narrowing`,
and `unbounded_loop`. Aspen phase-2 widening confirmed that `no_panic`
and `no_unwrap` are among the noisiest. Reducing false positives in these
families makes them promotable into default downstream paths sooner.

## What Changes

### `ignored_result`
- Skip `let _ = tx.send(msg)` only for narrow fire-and-forget teardown
  paths: supported `std::sync::mpsc::Sender` / `tokio::sync::mpsc::Sender`
  sends inside functions or methods named exactly `shutdown` or `close`
- Skip inside `Drop::drop` implementations where propagation is impossible

### `no_unwrap`
- Skip `.unwrap()` on compile-time-known `Some` / `Ok` constructions,
  including const/static-style initializers (e.g., `"127.0.0.1".parse().unwrap()`)
- Skip `.unwrap()` immediately after a `.is_some()` / `.is_ok()` guard in
  the same `if` branch
- Skip in `#[cfg(test)]` and `#[test]` contexts

### `no_panic`
- Skip `unreachable!()` inside exhaustive match arms that the compiler
  already proves unreachable (e.g., after a complete enum match with a
  trailing wildcard that names the variant)
- Skip `todo!()` when annotated with `#[allow(tigerstyle::no_panic)]` and
  a justification comment (stub-in-progress pattern)
- Skip in `#[cfg(test)]` and `#[test]` contexts

### `unchecked_narrowing`
- Recognize `u32::try_from(x).unwrap_or(u32::MAX)` and
  `x.min(u32::MAX as u64) as u32` as bounded patterns
- Recognize prior `assert!(x <= u32::MAX as u64)` within the same block,
  not just immediately preceding

### `unbounded_loop`
- Recognize `loop { tokio::select! { ... } }` as a bounded event loop
  (already spec'd but implementation may be incomplete)
- Recognize `while let Some(_) = rx.recv().await` as bounded only for
  `tokio::sync::mpsc::Receiver<_>` and keep warning on
  `tokio::sync::mpsc::UnboundedReceiver<_>`

## Capabilities

### Modified Capabilities
- `ignored_result`: Fewer false positives in Drop and fire-and-forget paths
- `no_unwrap`: Skips compile-time-known infallible unwraps and test code
- `no_panic`: Skips exhaustive-match unreachable arms and test code
- `unchecked_narrowing`: Recognizes more bounded-narrowing patterns
- `unbounded_loop`: Recognizes tokio select loops and bounded receivers

## Impact

- **Files**: `src/safety/ignored_result.rs`, `src/safety/no_unwrap.rs`,
  `src/safety/no_panic.rs`, `src/safety/unchecked_narrowing.rs`,
  `src/safety/unbounded_loop.rs`, `tests/ui.rs`, async fixture support under
  `ui_tests/tokio_async/`, and downstream evidence at
  `openspec/changes/lint-precision-hardening/evidence/aspen-comparison.md`
- **APIs**: No public API changes
- **Dependencies**: None
- **Testing**: New UI fixtures for each precision improvement (including async
  Tokio-backed standalone fixtures), plus re-validation against Aspen to
  confirm reduced noise
