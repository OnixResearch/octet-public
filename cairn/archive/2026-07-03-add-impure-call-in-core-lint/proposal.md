## Why

Pure Rust functions are weakened when declared core modules can call platform effects directly. Octet already checks wall-clock reads and verified-module purity, but `core_source_scopes` need a broader boundary lint for filesystem, IO, networking, process, environment, random, and runtime APIs.

## What Changes

- Define the `impure_call_in_core` lint contract for configured core source scopes.
- Classify narrow, statically visible platform-effect calls and route the diagnostic toward injected ports or shell-boundary helpers.
- Require catalog metadata, docs, rollout notes, and positive/negative UI fixtures before the lint enters the default catalog.

## Impact

- **Files**: `src/purity/impure_call_in_core.rs`, `src/purity/mod.rs`, `src/lib.rs`, metadata catalog, lint reference, templates, and UI fixtures.
- **Testing**: positive fixtures for effect calls in configured core scopes, negative fixtures for shell code and injected capability use, catalog freshness, and the ignored UI rail.
