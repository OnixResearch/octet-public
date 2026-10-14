## Context

This fixture explicitly maps the proposal/spec obligation that the blocked fixture omits: same-branch guard suppression.

## Goals / Non-Goals

**Goals:**
- improve `no_unwrap` precision

**Non-Goals:**
- broad lint rewrites

## Decisions

### 1. Model same-branch guard suppression structurally

**Choice:** suppress unwraps only when the same branch condition structurally requires `.is_some()` or `.is_ok()` for the same value.

**Covers:** `Scenario: Same-branch guard suppression`; `Scenario: Disjunctive guard still warns`

**Obligation cross-check:** carries forward the proposal testing promise for same-branch guard suppression, the delta-spec same-branch guard suppression scenario, and the disjunctive continued-warning boundary.

**Rationale:** structural condition matching avoids broad snippet searches that would hide unsafe unwraps.

**Alternative considered:** search the whole condition text for `value.is_some()`. Rejected because `value.is_some() || other_condition` would be misclassified.

**Implementation:** compare the unwrap receiver with the condition predicate and treat `&&` as preserving domination while requiring both sides of `||` to imply the guard before suppression.

**Boundary cases:** same-branch guard suppression is allowed for direct `&&` dominated guards; disjunctive guards still warn unless every disjunct implies the guard.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Same-branch guard suppression`
- artifact path:
  - `openspec/changes/design-obligation-covered-pass/evidence/no-unwrap-same-branch.txt`
- command / helper:
  - `cargo test --test ui no_unwrap_guarded -- --ignored --nocapture`
- exact expected output / acceptance signal:
  - fixture output contains no `no_unwrap` warning for the same-branch guard suppression case
- ordering / prerequisites:
  - run after implementation
- cleanup / postcondition:
  - no temp files remain

- scenario or requirement:
  - `Scenario: Disjunctive guard still warns`
- artifact path:
  - `openspec/changes/design-obligation-covered-pass/evidence/no-unwrap-disjunction.txt`
- command / helper:
  - `cargo test --test ui no_unwrap_guarded -- --ignored --nocapture`
- exact expected output / acceptance signal:
  - fixture output includes a `no_unwrap` warning for disjunctive guards
- ordering / prerequisites:
  - run after implementation
- cleanup / postcondition:
  - no temp files remain

## Risks / Trade-offs

- Over-suppression -> negative fixture keeps disjunctive guard warning visible.
