## Context

Octet already flags ignored join handles. A separate purity rule should prevent core code from creating task lifecycles in the first place when core scopes are declared.

## Decisions

### 1. Scope to declared core paths

**Choice:** `spawn_in_core` emits only inside `core_source_scopes`.

**Rationale:** Shell code and adapters legitimately own executors.

### 2. Match direct spawn APIs

**Choice:** Start with direct calls to known thread and async runtime spawn APIs.

**Rationale:** These calls are statically visible and have clear remediations.

### 3. Prefer command-returning remediation

**Choice:** Diagnostics should suggest returning a command/work item or accepting an executor port instead of spawning directly.

**Rationale:** That keeps functional core logic deterministic while preserving orchestration in the shell.

## Risks / Trade-offs

- Runtime-specific APIs are numerous. Keep the first call table narrow and fixture-backed.
- Some libraries expose spawn-like methods. Add facade support only after consumer evidence.

## Validation

Use async-capable UI fixtures where needed, plus unit tests for the spawn path classifier.
