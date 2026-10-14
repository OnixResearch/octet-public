## Context

This fixture intentionally omits one proposal/spec obligation from design coverage.

## Goals / Non-Goals

**Goals:**
- improve `no_unwrap` precision

**Non-Goals:**
- broad lint rewrites

## Decisions

### 1. Treat compile-time-known unwraps as safe

**Choice:** suppress unwraps when the compiler can prove the value is always present.

**Covers:** `Scenario: Disjunctive guard still warns`

**Rationale:** this preserves real-bug diagnostics.

**Alternative considered:** suppress all unwraps in conditionals. Rejected as too broad.

**Implementation:** inspect enclosing condition structure for disjunctions before suppressing.

**Boundary cases:** disjunctive conditions still warn.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Disjunctive guard still warns`
- artifact path:
  - `openspec/changes/design-obligation-missing-blocked/evidence/no-unwrap-disjunction.txt`
- command / helper:
  - `cargo test --test ui no_unwrap_guarded -- --ignored --nocapture`
- exact expected output / acceptance signal:
  - fixture output includes a `no_unwrap` warning for disjunctive guards
- ordering / prerequisites:
  - run after implementation
- cleanup / postcondition:
  - no temp files remain

## Risks / Trade-offs

- Intentional fixture risk -> the same-branch guard suppression obligation is missing from design text.
