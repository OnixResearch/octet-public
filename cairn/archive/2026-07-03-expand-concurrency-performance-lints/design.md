## Context

Concurrency and performance lint expansion should start with patterns that are statically visible, high impact, and explainable. The goal is not a broad static analyzer; it is a set of narrow source-shape checks that surface review questions early.

## Decisions

### 1. Rank candidates before implementation

**Choice:** Add a candidate matrix scoring signal strength, false-positive risk, required type info, fixture complexity, default level, and remediation clarity.

**Rationale:** The domains are under-covered, but adding noisy lints would reduce trust.

### 2. Start with async/concurrency lifecycle shapes

**Choice:** Prioritize `lock_held_across_await`, `blocking_call_in_async`, and `ignored_join_handle` / unbounded spawn shapes because they map directly to deadlock, starvation, and lifecycle leaks.

**Rationale:** These are common Rust systems failure modes with concrete remediations.

### 3. Keep performance checks scoped by declaration

**Choice:** Hot-loop allocation or clone checks require declared hot scopes or clear syntactic loops; otherwise they remain advisory or inventory-only.

**Rationale:** Performance rules need workload context. Scope controls prevent generic style policing.

## Risks / Trade-offs

- Async fixtures require dependency support. Reuse the existing Tokio UI fixture harness and root dev-dependencies.
- Blocking-call lists can become incomplete. Start with a narrow known list and expose candidate additions through tests.

## Validation

Each implemented lint gets metadata, docs, positive and negative UI fixtures, catalog freshness coverage, and downstream rollout guidance before default-level decisions.
