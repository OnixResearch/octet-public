# Structure Lints — Delta Spec

## ADDED Requirements

### Requirement: Function Length Limit

The lint MUST warn on functions whose body exceeds 70 lines (excluding
blank lines and comment-only lines).

The lint MUST report the actual line count and the 70-line limit.

The lint MUST NOT fire on functions annotated with `#[test]`.

The lint SHOULD NOT fire on generated code (`#[automatically_derived]`,
`include!` bodies, or files in `target/`).

The lint MAY offer a configurable threshold via `#[allow(tigerstyle::function_length)]`
or a config file.

#### Scenario: Long handler function

- GIVEN a function body spanning 85 non-blank, non-comment lines
- WHEN the lint runs
- THEN it warns: "function `handle` is 85 lines (limit: 70)"

#### Scenario: Long function with many comments

- GIVEN a function with 90 total lines but only 65 code lines
- WHEN the lint runs
- THEN it does NOT fire (65 code lines < 70)

#### Scenario: Test function

- GIVEN a 200-line function annotated `#[test]`
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Compound Condition Decomposition

The lint MUST be able to warn on `if` conditions containing 3 or more `&&`/`||`
operators at the same nesting level.

The lint MUST suggest decomposing into early returns or nested `if` blocks.

The lint MUST NOT fire on `match` guard expressions.

The lint MUST NOT fire inside `assert!` macros (compound assertions are
covered by a separate lint).

#### Scenario: Triple conjunction

- GIVEN `if a.is_some() && !a.unwrap().expired() && a.unwrap().holder == id`
- WHEN the lint runs
- THEN it suggests decomposing into nested `if let` / early returns

#### Scenario: Simple binary condition

- GIVEN `if x > 0 && x < 100`
- WHEN the lint runs
- THEN it does NOT fire (only 1 operator)

### Requirement: Ambiguous Same-Type Parameters

The lint MUST be able to warn on function signatures with 2 or more consecutive
parameters of the same primitive type (`u8`–`u128`, `i8`–`i128`, `f32`,
`f64`, `bool`, `&str`, `String`).

The lint MUST suggest wrapping in a named struct.

The lint MUST NOT fire on trait method implementations (the signature
is dictated by the trait).

The lint MUST NOT fire on functions with only 1 parameter of that type.

#### Scenario: Two u64 parameters

- GIVEN `fn compute_deadline(now_ms: u64, timeout_ms: u64) -> u64`
- WHEN the lint runs
- THEN it suggests a `DeadlineParams { now_ms: u64, timeout_ms: u64 }` struct

#### Scenario: Mixed types

- GIVEN `fn set(key: &str, value: &[u8], ttl: u64) -> Result<()>`
- WHEN the lint runs
- THEN it does NOT fire (no consecutive same-type params)

### Requirement: Allow Attribute Without Justification

The lint MUST be able to warn on `#[allow(clippy::...)]` and `#![allow(clippy::...)]`
attributes that are not preceded by a comment explaining why the lint
is suppressed.

The lint MUST accept a comment on the same line or the line immediately
preceding the attribute.

The lint MUST NOT fire on `#[allow(dead_code)]` in `#[cfg(test)]` blocks.

The lint MUST NOT fire on workspace-level lint configuration in
`Cargo.toml` (those are outside Clippy's reach anyway).

This lint exists because verified-logic carefully documents every
`#[allow(...)]` — its `conversion.rs` has a module-level `#![allow(...)]`
block with a comment explaining that "verified casts are the module's
entire purpose", and its `Cargo.toml` documents each allowed lint with
a `# ---` section comment. Unjustified suppression hides real bugs.

#### Scenario: Bare allow without comment

- GIVEN `#[allow(clippy::cast_possible_truncation)]` with no preceding comment
- WHEN the lint runs
- THEN it warns: "`#[allow]` without justification — add a comment explaining why"

#### Scenario: Allow with justification

- GIVEN:
  ```rust
  // Verified casts: Verus checks every narrowing in this module.
  #[allow(clippy::cast_possible_truncation)]
  ```
- WHEN the lint runs
- THEN it does NOT fire

### Requirement: Platform-Dependent Cast

The lint MUST warn on `as usize` and `as isize` casts from fixed-width
integer types (`u32`, `u64`, `i32`, `i64`, etc.).

The lint MUST suggest keeping the fixed-width type or using an explicit
conversion that handles the platform difference.

The lint MUST NOT fire on `as usize` from types that are already
platform-sized (e.g., array `.len()` which returns `usize`).

The lint MUST NOT fire inside indexing expressions (`array[x as usize]`)
when `x` is already bounded by a prior check against `array.len()`.

This lint exists because Tiger Style mandates explicitly sized types
(`u32`, `i64`, not `usize`) for consistent cross-platform behavior.
clankers' `frame.rs` does `u32::from_be_bytes(len_buf) as usize` — on
a 16-bit platform this truncates. The `as usize` idiom scatters
platform assumptions throughout the codebase.

#### Scenario: Fixed-width to usize

- GIVEN `let size = u32::from_be_bytes(buf) as usize;`
- WHEN the lint runs
- THEN it warns: "`as usize` from u32 is platform-dependent — use explicit conversion"

#### Scenario: Len-bounded index

- GIVEN:
  ```rust
  let idx: u32 = header.index;
  assert!((idx as usize) < data.len());
  let val = data[idx as usize];
  ```
- WHEN the lint runs
- THEN it does NOT fire (bounded by len check)
