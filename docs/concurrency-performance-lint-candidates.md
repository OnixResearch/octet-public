# Concurrency and performance lint candidate matrix

This matrix records the first-wave ranking required before adding new concurrency
or performance lints. Ratings are qualitative and review-oriented: `high`,
`medium`, or `low` for signal and remediation clarity; `low`, `medium`, or
`high` for false-positive risk and fixture complexity.

| Candidate | Domain | Signal | False-positive risk | Type-info needs | Fixture complexity | Default level | Remediation clarity | Status | Decision rationale |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `ignored_join_handle` | concurrency | high | low | return type resolves to supported `JoinHandle` | low | `warn` | high | accepted | Discarded thread/task handles hide lifecycle, cancellation, and failure handling. The first implementation flags statement and wildcard discards while named handles remain clean. |
| `allocation_in_loop` | performance | medium | medium | return type resolves to supported allocation types | low | `allow` | high | accepted | Allocation constructors inside explicit loop bodies are locally visible, but workload context still matters, so the lint starts advisory and opt-in. |
| `lock_held_across_await` | concurrency | high | medium | lock guard type plus await span ownership | high | `allow` | high | deferred | Needs more precise guard-lifetime facts to avoid warning after `drop(guard)` or scoped blocks. |
| `blocking_call_in_async` | concurrency | medium | medium | async context plus known blocking call table | medium | `allow` | medium | deferred | The blocking call list and runtime context need a maintained policy table before default rollout. |
| `unbounded_task_spawn` | concurrency | medium | high | loop/task-spawn context and bounded semaphore facts | high | `allow` | medium | deferred | Useful but noisy without structural backpressure recognition. |
| `clone_in_hot_loop` | performance | medium | high | declared hot scope or loop context plus clone type | medium | `allow` | medium | deferred | Clones in loops may be acceptable for small or copy-like values; defer until hot-scope declarations exist. |

Accepted first-wave scope:

- `ignored_join_handle` emits by default because the discarded-handle shape is
  narrow, type-backed, and has a clear lifecycle remediation: bind, join, await,
  abort, store, or document an intentional boundary.
- `allocation_in_loop` is cataloged as `allow` because the shape is useful for
  hot-path reviews but still needs workload context before default enforcement.

Promotion blocker policy: any deferred candidate remains inventory-only until a
saved dogfood or consumer summary names raw findings, accepted debt, fixed
findings, residual findings, and a `ready`, `deferred`, or `blocked` decision.
