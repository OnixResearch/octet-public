## Why

Logging, printing, tracing, and metrics emission are side effects. Pure core logic should return decisions, events, or observations that the shell records, rather than writing to process-global sinks directly.

## What Changes

- Define a `logging_in_core` lint for direct diagnostic/telemetry output inside configured core scopes.
- Cover `println!`, `eprintln!`, `dbg!`, `log::*`, `tracing::*`, and `metrics::*` macro/call shapes.
- Require diagnostics that suggest returned events or shell-owned observability ports.

## Impact

- **Files**: `src/purity/logging_in_core.rs`, registration, metadata, lint reference, rollout docs, templates, and UI fixtures.
- **Testing**: positive macro fixtures and negative fixtures for returned events, shell logging, and test-only debug output.
