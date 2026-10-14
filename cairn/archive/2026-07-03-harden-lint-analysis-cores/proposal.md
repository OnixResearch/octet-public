## Why

Several shipped lints still classify code using source snippets or string markers where rustc HIR, type information, or structured syntax is available. That keeps the lints simple, but it makes false positives and false negatives harder to reason about and harder to test as pure logic.

The most visible examples are `public_unsafe_api`, `multi_lock_ordering`, `unbounded_loop`, and `magic_number`. Each should move toward a functional core that receives normalized facts and returns a classification, with the rustc visitor acting only as the imperative shell.

## What Changes

- Define a lint-analysis-core hardening contract for snippet-heavy lints.
- Refactor target lints to prefer HIR/type-derived facts and limit source snippets to diagnostics or documented fallback cases.
- Add pure unit tests for classification cores plus UI fixtures for rustc integration.

## Impact

- **Files**: `src/safety/public_unsafe_api.rs`, `src/concurrency/multi_lock_ordering.rs`, `src/safety/unbounded_loop.rs`, `src/structure/magic_number.rs`, shared helper modules if extracted, and UI fixtures.
- **Testing**: pure positive/negative core tests, UI guardrail fixtures, and targeted `cargo test --lib` / UI rails inside the nix devshell.
