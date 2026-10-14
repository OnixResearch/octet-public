## Why

Interior mutability lets code mutate state through shared references, which hides side effects from pure-function signatures. Core data types should expose state transitions explicitly unless a shell boundary owns the mutable cell or lock.

## What Changes

- Define an `interior_mutability_in_core` lint for `Cell`, `RefCell`, `UnsafeCell`, locks, and atomics in configured core scopes.
- Focus the first wave on fields and local construction sites with clear remediation toward explicit state-transition values.
- Require catalog metadata, docs, rollout notes, and positive/negative fixtures.

## Impact

- **Files**: `src/purity/interior_mutability_in_core.rs`, purity registration, metadata catalog, lint reference, templates, and fixtures.
- **Testing**: fixtures for interior-mutable fields/construction in core scopes and guardrails for shell-owned synchronization and explicit immutable state.
