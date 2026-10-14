## Why

Pure cores only pay off when their laws are explicit and tested. Octet already has many deterministic transforms: config precedence, lint classification, diagnostic ordering, evidence identity, receipt validation, artifact manifests, and boundary checks. Example tests and UI fixtures catch important cases, but they do not prove algebraic expectations such as idempotence, stable sorting, precedence monotonicity, or fail-closed behavior across generated inputs.

The Haskell practice worth adapting is QuickCheck-style law testing. In Rust, that means bounded, reproducible property tests around pure functions, paired with concrete positive and negative examples.

## What Changes

- Create a law inventory for selected pure cores whose behavior should be stable across refactors.
- Add property/law tests for config precedence, deterministic sorting/deduplication, receipt/artifact identity, and non-trivial lint classifiers.
- Require generated cases to be reproducible and bounded, with named case budgets and saved failing inputs when available.
- Keep UI fixtures, golden output tests, and transcript evidence as complementary rails rather than replacing them with property tests.

## Impact

- **Files**: selected tests under `tests/`, core-specific test modules, `src/config.rs`, `src/diagnostics.rs`, selected lint pure-core helpers, `cargo-octet` pure validation modules, and possibly dev-dependency configuration if property testing support is added.
- **Testing**: positive examples, negative malformed-input cases, reproducible property/law suites, existing UI/runner rails, and Cairn validate/gates.
