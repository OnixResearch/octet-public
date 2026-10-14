## Context

This fixture carries the proposal's archive non-goal and evidence promise into the delta spec.

## Goals / Non-Goals

**Goals:**
- Exercise spec obligation coverage.

**Non-Goals:**
- Do not normalize historical archived examples.

## Decisions

### 1. Prompt plus boundary scenario

**Choice:** Add a prompt and a named archive-boundary scenario.

**Covers:** `Scenario: Archive non-goal and evidence promise are specified`

**Obligation cross-check:** The proposal's non-goal and evidence promise are covered by the named scenario.

**Rationale:** Positive control.

**Alternative considered:** Prompt-only wording. Rejected because it omits the proposal promise.

**Implementation:** Template and config text only.

**Boundary cases:** Archived examples remain untouched.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Archive non-goal and evidence promise are specified`
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
