## Why

Octet currently has only one concurrency lint and one performance lint. The review guides already describe hazards such as lock ordering, blocking work, backpressure, hot-path allocation, and task lifecycle management, but the static lint catalog only covers a small slice.

The next lint-growth area should target narrow, high-signal concurrency and performance source shapes that map directly to Tiger Style outcomes.

## What Changes

- Define an expansion contract for concurrency/performance lint candidates.
- Prioritize narrow first lints such as lock held across `.await`, blocking calls in async contexts, ignored `JoinHandle` / unbounded task spawn, and allocation in declared hot loops.
- Require each candidate to ship with scope controls, positive and negative fixtures, and rollout guidance before default-level decisions.

## Impact

- **Files**: new lint modules under `src/concurrency/` and `src/performance/`, `src/lib.rs`, lint metadata, docs, templates, UI fixtures, and optional config keys.
- **Testing**: async/Tokio UI fixture support, positive/negative fixtures, catalog freshness tests, and downstream profile updates.
