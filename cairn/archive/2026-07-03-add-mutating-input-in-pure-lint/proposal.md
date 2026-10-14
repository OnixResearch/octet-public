## Why

Pure-function signatures should make state change explicit. `&mut` parameters, raw pointers, and references to locks or atomics let a function mutate caller-owned state without returning a new value.

## What Changes

- Define a `mutating_input_in_pure` lint for mutable/capability-bearing parameters in declared pure contexts.
- Treat configured core scopes and future pure markers as declared pure contexts.
- Require diagnostics that suggest owned input/output state transitions or imperative-shell placement.

## Impact

- **Files**: `src/purity/mutating_input_in_pure.rs`, registration, metadata catalog, lint reference, templates, and UI fixtures.
- **Testing**: positive fixtures for mutable references/raw pointers/interior-mutable parameters and negative fixtures for owned state transitions and shell APIs.
