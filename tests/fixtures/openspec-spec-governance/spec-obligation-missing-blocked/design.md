## Context

This fixture intentionally omits the proposal's archive non-goal and evidence promise from the delta spec.

## Goals / Non-Goals

**Goals:**
- Exercise spec omission detection.

**Non-Goals:**
- Do not normalize historical archived examples.

## Decisions

### 1. Prompt-only change

**Choice:** Add a prompt.

**Covers:** `Scenario: Maintainer sees prompt`

**Obligation cross-check:** Historical archive non-goal and saved evidence promise must be in the delta spec, but this fixture omits them.

**Rationale:** Negative control.

**Alternative considered:** None.

**Implementation:** Template text only.

**Boundary cases:** Archived examples remain untouched.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Maintainer sees prompt`
- artifact path:
  - `evidence/archive-untouched.txt`
- command / helper:
  - `check-fixture`
- exact expected output / acceptance signal:
  - `PASS archive untouched`
- ordering / prerequisites:
  - none
- cleanup / postcondition:
  - fixture removed

## Risks / Trade-offs

- none
