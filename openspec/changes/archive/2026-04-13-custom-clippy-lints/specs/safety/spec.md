# Safety Lints — Delta Spec

## ADDED Requirements

### Requirement: Compound Assertion Detection

The lint MUST detect `assert!(expr1 && expr2)` and `assert!(expr1 || expr2)`
where the macro argument contains a top-level binary `&&` or `||` operator.

The lint MUST suggest splitting into separate `assert!()` calls.

The lint MUST NOT fire on `assert_eq!` or `assert_ne!` (those already provide
per-operand diagnostics).

The lint SHOULD NOT fire on `debug_assert!` compound expressions in test code.

#### Scenario: Simple compound

- GIVEN `assert!(a > 0 && b > 0)`
- WHEN the lint runs
- THEN it emits a warning suggesting `assert!(a > 0); assert!(b > 0);`

#### Scenario: Nested boolean in method call

- GIVEN `assert!(vec.iter().all(|x| x > 0 && x < 100))`
- WHEN the lint runs
- THEN it does NOT fire (the `&&` is inside a closure, not top-level)

#### Scenario: Conditional assertion (implication pattern)

- GIVEN `if is_leader { assert!(has_quorum); }`
- WHEN the lint runs
- THEN it does NOT fire (this is the approved Tiger Style pattern)

### Requirement: Assertion Density

The lint MUST be able to warn on functions exceeding 20 lines that contain fewer
than 2 assertion macros (`assert!`, `assert_eq!`, `assert_ne!`,
`debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`).

The lint MUST NOT fire on functions annotated `#[test]`, `#[cfg(test)]`,
or inside `mod tests` blocks.

The lint MUST NOT fire on functions whose body is a single expression or
a pure delegation (`self.inner.method(args)`).

#### Scenario: Function with no assertions

- GIVEN a 30-line function with zero assertions
- WHEN the lint runs
- THEN it emits a suggestion to add assertions documenting invariants

#### Scenario: Short function

- GIVEN a 10-line function with zero assertions
- WHEN the lint runs
- THEN it does NOT fire (under the 20-line threshold)

### Requirement: Unbounded Loop Detection

The lint MUST warn on `loop { }` blocks that do not contain a visible
iteration counter, `break` condition tied to a counter, or bounded
channel receive.

The lint SHOULD warn on `while let Some(_) = iter.next()` without a
limit check in the body.

The lint MUST NOT fire on `loop` blocks containing `tokio::select!`
(event loops are a valid pattern when bounded by channel capacity).

#### Scenario: Bare retry loop

- GIVEN `loop { if try_connect() { break; } }`
- WHEN the lint runs
- THEN it warns about missing iteration bound

#### Scenario: Bounded retry

- GIVEN `for _ in 0..MAX_RETRIES { if try_connect() { break; } }`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Ignored Result Detection

The lint MUST detect `let _ = expr` where `expr` has return type
`Result<_, _>`.

The lint MUST suggest using `let _: () = expr?;` or explicit error
handling.

The lint MUST NOT fire on `let _ = expr` where `expr` returns a
non-Result type (e.g., guard drops).

#### Scenario: Silenced fallible call

- GIVEN `let _ = file.sync_all();`
- WHEN the lint runs
- THEN it warns that a `Result` is being silently discarded

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

This lint exists because verified-logic dedicates an entire module
(`conversion.rs`) to proving narrowing casts are safe — unchecked
`as` casts silently truncate and are a common source of subtle bugs.

#### Scenario: Bare narrowing cast

- GIVEN `let id: u32 = some_u64 as u32;`
- WHEN the lint runs
- THEN it warns: "narrowing cast from u64 to u32 without bounds check — use u32::try_from()"

#### Scenario: Cast with prior guard

- GIVEN `if value <= u32::MAX as u64 { let v = value as u32; }`
- WHEN the lint runs
- THEN it does NOT fire (bounds checked by the `if` guard)

#### Scenario: Same-width cast

- GIVEN `let x: i64 = y as i64;` where `y: u32`
- WHEN the lint runs
- THEN it does NOT fire (widening cast, not narrowing)

### Requirement: Raw Arithmetic Overflow Detection

The lint MUST be able to warn on raw `+`, `-`, `*` operations on unsigned integer
types (`u8`–`u128`) in production code where the operands are not
compile-time constants and no overflow protection is present.

Overflow protection includes: `checked_add`/`checked_sub`/`checked_mul`,
`saturating_add`/`saturating_sub`/`saturating_mul`, a preceding
`assert!` or `if` guard that bounds the operands, or being inside a
`verus! { }` block.

The lint MUST NOT fire on `+= 1` or `i + 1` loop increments where `i`
is bounded by a `for` range or `while i < LIMIT` guard.

The lint MUST NOT fire in test code.

This lint exists because verified-logic uses `checked_mul`, `saturating_add`,
and `saturating_sub` for every arithmetic operation — `backoff.rs` proves
overflow safety with Verus, `rate_limit.rs` splits multiplications to
avoid u64 overflow, and `expiration.rs` uses saturating arithmetic for
all TTL calculations.

#### Scenario: Unprotected multiplication

- GIVEN `let total = count * item_size;` where both are `u64`
- WHEN the lint runs
- THEN it suggests `count.checked_mul(item_size)` or `count.saturating_mul(item_size)`

#### Scenario: Constant arithmetic

- GIVEN `const MAX: u64 = 1024 * 1024;`
- WHEN the lint runs
- THEN it does NOT fire (compile-time constants are checked by the compiler)

### Requirement: Unchecked Division by Parameter

The lint MUST warn on `a / b` or `a % b` where `b` is a function
parameter or local variable (not a constant) and there is no prior
`assert!(b > 0)`, `assert!(b != 0)`, `if b == 0 { return }`,
or `requires b > 0` (Verus) guard in the function.

The lint MUST suggest adding an assertion or early return for the
zero-division case.

The lint MUST NOT fire when the divisor is a literal or named constant.

This lint exists because every division in verified-logic has an explicit
`requires b > 0` precondition — `div_ceil` requires it, `rate_limit`
requires `rate_per_sec > 0`, `hash_bucket` requires `bucket_count > 0`.

#### Scenario: Division by parameter

- GIVEN `fn avg(total: u64, count: u64) -> u64 { total / count }`
- WHEN the lint runs
- THEN it warns: "division by `count` without zero check"

#### Scenario: Division by constant

- GIVEN `let seconds = ms / 1000;`
- WHEN the lint runs
- THEN it does NOT fire

#### Scenario: Guarded division

- GIVEN `assert!(count > 0); let avg = total / count;`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Catch-All Match Arm on Enum

The lint MUST be able to warn on `match` expressions over enum types where
the last arm is a wildcard (`_`) that returns a "deny" value:
`false`, `None`, `Err(...)`, or `0`.

The lint MUST suggest replacing the wildcard with explicit variant
listing so the compiler forces updates when new variants are added.

The lint MUST NOT fire on `match` over non-enum types (`&str`,
integers, tuples).

The lint MUST NOT fire when the wildcard arm has a comment explaining
the catch-all intent (e.g., `// All other variants are unauthorized`).

This lint exists because clankers' `Capability::authorizes()` uses
`_ => false` — if a new `Capability` variant is added, it silently
gets denied authorization instead of forcing the developer to make
an explicit decision. In security-relevant code, this is a class of
bug that leads to silent privilege denial (or worse, silent grant if
the default were `true`).

#### Scenario: Security match with catch-all deny

- GIVEN:
  ```rust
  match (self, op) {
      (Capability::Read, Operation::Read) => true,
      _ => false,
  }
  ```
- WHEN the lint runs
- THEN it warns: "catch-all `_ => false` on enum match — new variants will be silently denied"

#### Scenario: Catch-all with justification comment

- GIVEN:
  ```rust
  match event {
      Event::Click(p) => Some(p),
      // Non-positional events have no coordinates
      _ => None,
  }
  ```
- WHEN the lint runs
- THEN it does NOT fire (comment justifies the catch-all)

#### Scenario: Match on string

- GIVEN `match cmd.as_str() { "start" => true, _ => false }`
- WHEN the lint runs
- THEN it does NOT fire (not an enum match)
