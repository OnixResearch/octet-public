# Custom Clippy Lints for Tiger Style

## Why

Tiger Style rules in aspen are enforced through code review and convention.
This works until it doesn't — reviewers miss things, new contributors don't
know the rules, and violations accumulate quietly. A scan of aspen's crates
found:

- 15+ non-test files with `.unwrap()` (one file had 219 occurrences)
- 47 files with `panic!()`/`todo!()` in production paths
- 15 instances of `assert!(a && b)` compound assertions
- 20+ functions exceeding 70 lines (worst: 546 lines)
- Boolean variables named `active`, `enabled`, `ready` without `is_`/`has_` prefix
- `RWLock`, `RPCError`, `IDs` acronym style instead of `RwLock`, `RpcError`, `Ids`

These are all mechanically detectable. A lint crate catches them at
`cargo check` time, before review.

## What Changes

A new Rust crate providing custom lints, runnable via `dylint` or
`cargo-marker`. Each lint maps to one Tiger Style rule with a clear
error message and fix suggestion.

## Capabilities

### New Capabilities

- `compound-assertion`: Detect `assert!(a && b)`, suggest split assertions
- `bool-naming`: Detect bool bindings without `is_`/`has_`/`should_`/`can_` prefix
- `function-length`: Detect functions over 70 lines
- `acronym-style`: Detect multi-char uppercase runs in type/variant names
- `numeric-units`: Detect numeric variables missing unit suffix (`_ms`, `_bytes`)
- `verified-purity`: Detect async/IO/time in `verified/` modules
- `contradictory-time`: Detect functions with time param that also call `Instant::now()`
- `ambient-clock`: Detect direct wall-clock reads outside explicit shell-boundary helpers
- `unbounded-loop`: Detect `loop {}`/`while` without explicit iteration bound
- `assertion-density`: Warn on non-trivial functions with fewer than 2 assertions
- `ignored-result`: Detect `let _ = fallible_expr()`
- `unchecked-narrowing`: Detect bare `as u32`/`as u16`/`as u8` without bounds check
- `raw-arithmetic-overflow`: Detect unprotected `+`/`-`/`*` on unsigned types
- `unchecked-division`: Detect `a / b` where `b` is a parameter without zero guard
- `ambiguous-params`: Detect 2+ same-type primitive params without wrapper struct
- `unjustified-allow`: Detect `#[allow(clippy::...)]` without an explanatory comment
- `catch-all-on-enum`: Detect `_ => false`/`_ => None` catch-all in security-relevant match arms
- `float-for-currency`: Detect `f64` in cost/price/budget struct fields
- `platform-dependent-cast`: Detect `as usize`/`as isize` from fixed-width integers

## Impact

- **Files**: New crate under `tigerstyle/`
- **Dependencies**: `dylint_linting` (or `marker_api`), `syn`, `quote`
- **Testing**: Each lint has pass/fail `.rs` fixture files
- **Integration**: Runs in aspen's CI via `cargo dylint` or marker
