## Why

Global mutable state creates hidden inputs and outputs that make functions hard to test, replay, and verify. Core modules should model state transitions explicitly instead of reaching through process-wide singletons.

## What Changes

- Define a `global_state_in_core` lint for global mutable state declarations and process-wide lazy state inside configured core scopes.
- Cover `static mut`, `thread_local!`, `OnceLock`, `LazyLock`, global locks, and global atomics where they are declared in core code.
- Require diagnostics that steer maintainers to explicit state values, stores, or shell-owned singletons.

## Impact

- **Files**: `src/purity/global_state_in_core.rs`, registration, metadata, docs, templates, and UI fixtures.
- **Testing**: positive fixtures for global mutable declarations in core scopes and negative fixtures for constants, immutable statics, explicit state parameters, and shell-owned state.
