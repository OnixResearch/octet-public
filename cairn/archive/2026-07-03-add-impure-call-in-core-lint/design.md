## Context

`verified_purity` is intentionally tied to `verified/` modules. Product code also has functional-core zones declared through `core_source_scopes`; those zones need a configurable effect boundary without pretending to prove whole-program purity.

## Decisions

### 1. Scope by configured core paths

**Choice:** `impure_call_in_core` only emits inside `core_source_scopes`.

**Rationale:** Boundary code legitimately owns IO and platform calls. A scoped lint keeps the signal tied to pure-function intent.

### 2. Use a pure classifier for effect paths

**Choice:** Implement a pure matching core that maps resolved def paths or syntactic paths to an effect family such as `fs`, `io`, `net`, `process`, `env`, `random`, or `runtime`.

**Rationale:** The lint pass should be a thin imperative shell over deterministic classification helpers that can be unit-tested without rustc session setup.

### 3. Keep the first wave narrow

**Choice:** Start with direct calls to known APIs and macros; defer transitive call-graph inference.

**Rationale:** Direct platform-effect calls are explainable and fixture-friendly. Transitive purity needs a separate design.

## Risks / Trade-offs

- Path deny-lists can miss facade crates. Add new paths only with fixtures.
- Broad IO matching can flag helper wrappers. Remediation should point to ports/capabilities, not deletion.

## Validation

Ship positive and negative UI fixtures, pure classifier unit tests, metadata/docs freshness updates, and `cargo test --lib` plus the ignored UI rail.
