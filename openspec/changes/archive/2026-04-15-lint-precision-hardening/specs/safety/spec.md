# Safety Lint Precision Hardening — Delta Spec

## ADDED Requirements

### Requirement: No Unwrap In Production Code

The lint MUST detect `.unwrap()` and `.expect()` calls on `Option` and
`Result` in production code.

The lint MUST NOT fire in `#[test]` functions.

The lint MUST NOT fire in items or modules guarded by `#[cfg(test)]` or an
inner `#![cfg(test)]` that makes the enclosing item test-only.

The lint MUST NOT fire on compile-time-known infallible constructions such as
`Some(value).unwrap()`, `Ok(value).unwrap()`, or valid literal-string parses
for a finite supported standard set (primitive bool/int/float/char, `String`,
and standard network address types such as `"127.0.0.1".parse().unwrap()`),
including const/static-style initializer code.

The lint MUST NOT fire when the unwrap is dominated by a same-branch
`.is_some()` or `.is_ok()` guard.

In this change, the new dominance / compile-time-known suppression rules apply
only to `.unwrap()`. `.expect()` continues to warn unless a future change
specifies matching suppression behavior.

#### Scenario: Test helper unwrap

- GIVEN a `#[test]` function that calls `value.unwrap()`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Cfg-test module unwrap

- GIVEN a helper inside `#[cfg(test)] mod tests { ... }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Not-test cfg still warns

- GIVEN a helper inside `#[cfg(not(test))] mod prod_only { ... }`
- WHEN the lint runs
- THEN it still warns on `value.unwrap()`

#### Scenario: Compile-time-known unwrap

- GIVEN `let v = Some(1_u32).unwrap();`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Literal parse unwrap

- GIVEN `static ADDR: std::net::IpAddr = "127.0.0.1".parse().unwrap();`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Invalid literal parse still warns

- GIVEN `let addr: std::net::IpAddr = "not an ip".parse().unwrap();`
- WHEN the lint runs
- THEN it still warns

#### Scenario: Guarded unwrap

- GIVEN `if value.is_some() { value.unwrap(); }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Panic Macro In Production Code

The lint MUST detect `panic!()`, `todo!()`, `unimplemented!()`, and
`unreachable!()` in production code.

The lint MUST NOT fire in `#[test]` functions.

The lint MUST NOT fire in items or modules guarded by `#[cfg(test)]` or an
inner `#![cfg(test)]` that makes the enclosing item test-only.

The lint MUST NOT fire on `unreachable!()` in a trailing wildcard arm when the
preceding enum match arms already list the concrete variants exhaustively for a
local, non-`#[non_exhaustive]` enum.

The lint MUST NOT fire on `todo!()` when the call is inside a function or
block explicitly allowed with `#[allow(tigerstyle::no_panic)]` and an adjacent
justification comment.

For this rule, an adjacent justification comment is a `// ...` comment on the
same line as the allow attribute or on the immediately preceding non-empty
line. Blank lines, doc comments, and block comments do NOT count.

#### Scenario: Test-only panic

- GIVEN a `#[test]` helper that calls `panic!("boom")`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Cfg-test module panic

- GIVEN a helper inside `#[cfg(test)] mod tests { ... }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Not-test cfg still warns

- GIVEN a helper inside `#[cfg(not(test))] mod prod_only { panic!(...) }`
- WHEN the lint runs
- THEN it still warns

#### Scenario: Exhaustive wildcard unreachable

- GIVEN a local enum match whose explicit arms already cover every variant and
  whose trailing wildcard uses `unreachable!()`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Justified todo allow

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` with a `//`
  justification comment on the same line or the immediately preceding line
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Justified block todo allow

- GIVEN a block annotated with `#[allow(tigerstyle::no_panic)]` and an
  adjacent `//` justification comment that contains `todo!(...)`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Unjustified todo still warns

- GIVEN `#[allow(tigerstyle::no_panic)] fn stub() { todo!(...) }` without that
  adjacent `//` justification comment
- WHEN the lint runs
- THEN it still warns

## MODIFIED Requirements

### Requirement: Ignored Result Detection

The lint MUST detect `let _ = expr` where `expr` has return type
`Result<_, _>`.

The lint MUST suggest using `let _: () = expr?;` or explicit error
handling.

The lint MUST NOT fire on `let _ = expr` where `expr` returns a
non-Result type (e.g., guard drops).

The lint MUST NOT fire inside `Drop::drop` implementations.

The lint MUST NOT fire on `let _ = tx.send(msg);` when all of the
following hold:
- the method name is exactly `send`
- `tx` has type `std::sync::mpsc::Sender<_>` or `tokio::sync::mpsc::Sender<_>`
- the enclosing function or method name is exactly `shutdown` or `close`

#### Scenario: Drop cleanup swallow

- GIVEN `impl Drop for Cleaner { fn drop(&mut self) { let _ = self.tx.send(msg); } }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Shutdown send swallow

- GIVEN `fn shutdown(&self) { let _ = self.tx.send(msg); }` where `self.tx` has
  type `std::sync::mpsc::Sender<_>`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Tokio close send swallow

- GIVEN `async fn close(&self) { let _ = self.tx.send(msg).await; }` where
  `self.tx` has type `tokio::sync::mpsc::Sender<_>`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Ordinary send still warns

- GIVEN `fn publish(&self) { let _ = self.tx.send(msg); }` with the same sender
  type
- WHEN the lint runs
- THEN it still warns

### Requirement: Unchecked Narrowing Cast

The lint MUST detect `as u32`, `as u16`, `as u8`, `as i32`, `as i16`,
`as i8` casts from a wider integer type without a prior bounds check
on the source value.

A bounds check is any of: `if value <= TYPE::MAX`, a preceding
`assert!(value <= ...)`, wrapping in `TryFrom`/`try_into()`, or calling
a checked narrowing function.

The lint MUST suggest using `u32::try_from(value)` or a checked
narrowing function like `verified_logic::conversion::narrow_u32`.

The lint MUST NOT fire on casts where the source type is the same width
or narrower than the target (e.g., `u16 as u32`).

The lint MUST NOT fire inside `#[allow(clippy::cast_possible_truncation)]`
blocks (explicit opt-out).

The lint MUST NOT fire on bounded forms such as `value.min(u32::MAX as u64) as u32`.

The lint MUST NOT fire on `u32::try_from(value).unwrap_or(u32::MAX)`.

The lint MUST treat an earlier same-block `assert!(value <= u32::MAX as u64)`
as a valid bounds check even when it is not the immediately preceding
statement.

This same-block assert suppression MUST apply only when the asserted
expression matches the casted value and the asserted bound matches the cast
target. Unrelated asserts, shadowed values, mismatched bounds, or later asserts
MUST NOT suppress the warning.

#### Scenario: Min-bounded narrowing

- GIVEN `let id = value.min(u32::MAX as u64) as u32;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Try-from fallback narrowing

- GIVEN `let id = u32::try_from(value).unwrap_or(u32::MAX);`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Same-block assert narrowing

- GIVEN an earlier `assert!(value <= u32::MAX as u64);` in the same block
  before `let id = value as u32;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Unrelated same-block assert still warns

- GIVEN a same-block assert about a different value, a shadowed value, the
  wrong bound shape, or an assert that appears after the cast
- WHEN the lint runs on `value as u32`
- THEN it still warns

### Requirement: Unbounded Loop Detection

The lint MUST warn on `loop { }` blocks that do not contain a visible
iteration counter, `break` condition tied to a counter, or bounded
channel receive.

The lint SHOULD warn on `while let Some(_) = iter.next()` without a
limit check in the body.

The lint MUST NOT fire on `loop` blocks containing Tokio's `select!`
macro, written either as `tokio::select!` or an imported `select!`.

The lint MUST NOT fire on `while let Some(_) = rx.recv().await` when `rx`
has type `tokio::sync::mpsc::Receiver<_>`.

The lint MUST continue warning on `tokio::sync::mpsc::UnboundedReceiver<_>`
receive loops.

#### Scenario: Tokio select loop

- GIVEN `loop { tokio::select! { ... } }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Imported Tokio select loop

- GIVEN `use tokio::select; loop { select! { ... } }`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Bounded receiver loop

- GIVEN `while let Some(msg) = rx.recv().await { handle(msg); }`
- WHEN `rx` has type `tokio::sync::mpsc::Receiver<_>`
- THEN the lint does NOT fire

#### Scenario: Iterator-next loop still warns

- GIVEN `while let Some(msg) = iter.next() { handle(msg); }` without a limit
  check in the body
- WHEN the lint runs
- THEN it still warns

#### Scenario: Unbounded receiver loop still warns

- GIVEN `while let Some(msg) = rx.recv().await { handle(msg); }`
- WHEN `rx` has type `tokio::sync::mpsc::UnboundedReceiver<_>`
- THEN the lint still warns
