## Why

The lint fixture manifests currently prove that every registered lint has at least one emitting fixture. That catches half-registered lints, but it does not prove each lint also has a non-emitting guardrail for valid code, boundary cases, disabled configuration, and known false-positive traps.

For a lint distribution meant to be promoted gradually, missing negative coverage is product risk: a lint can be fixture-complete while still being too noisy for consumers.

## What Changes

- Add a guardrail-fixture contract requiring positive and negative coverage for every lint.
- Introduce a checked manifest for non-emitting fixtures and boundary cases.
- Extend library validation so new or changed lints fail freshness tests unless both positive and negative cases are represented.

## Impact

- **Files**: `tests/fixtures/*`, `src/lib.rs` fixture validation tests, `ui_tests/`, and lint-specific fixture-local configs.
- **Testing**: manifest freshness tests, UI rail, representative disabled/configured negative fixtures, and Cairn validation/gates.
