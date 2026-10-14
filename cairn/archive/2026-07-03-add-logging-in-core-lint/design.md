## Context

Observability calls are useful, but they couple core decisions to global sinks and make function behavior non-local. In functional-core style, core returns structured events and shells decide how to record them.

## Decisions

### 1. Treat observability as a shell concern

**Choice:** `logging_in_core` emits inside configured core scopes for direct printing, debugging, logging, tracing, or metrics emission.

**Rationale:** This makes side effects visible without preventing shells from recording output.

### 2. Match macro and call paths

**Choice:** Implement macro-name detection for standard print/debug/logging macros and call-path detection where crates expose functions.

**Rationale:** Most logging APIs are macro-shaped; path matching alone is insufficient.

### 3. Encourage returned events

**Choice:** Diagnostic help points to returning domain events, audit records, or observations.

**Rationale:** Returned data preserves deterministic core behavior and lets shells own sinks.

## Risks / Trade-offs

- Macro expansion spans can be noisy. Emit only on user-written macro/call sites.
- Some debug output in tests is acceptable. Reuse test-context suppression.

## Validation

Add macro-focused UI fixtures, clean returned-event fixtures, and classifier unit tests for names.
