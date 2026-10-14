## Why

Spawning threads or tasks from core logic creates hidden scheduling, lifecycle, and cancellation effects. Pure functions should return commands, plans, or work descriptions while shells own executors and joins.

## What Changes

- Define a `spawn_in_core` lint for direct thread/task spawn calls inside configured core scopes.
- Cover common spawn surfaces such as `std::thread::spawn`, `tokio::spawn`, `tokio::task::spawn_blocking`, `async_std::task::spawn`, and scoped executor facades as fixtures justify.
- Require remediation toward command values, injected executor ports, or shell-boundary helpers.

## Impact

- **Files**: `src/purity/spawn_in_core.rs`, purity/concurrency registration touchpoints, metadata catalog, docs, templates, and UI fixtures.
- **Testing**: async/Tokio fixtures for spawn calls and clean fixtures for returned commands or injected executors.
