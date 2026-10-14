## Context

This fixture should fail only because `## Change Status` is not placed
immediately before `## Verification Coverage`.

## Goals / Non-Goals

**Goals:**
- keep one governed verification task and one matching coverage row
- place a syntactically valid status section before another phase heading

**Non-Goals:**
- missing status section
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Misplace an otherwise valid status section

**Choice:** Keep valid status bullets but place the whole section before Phase 2.

**Rationale:** The gate should emit the exact malformed-or-misplaced finding.

**Implementation:** The fixture keeps one verification row so section placement
is the only intended failure.
