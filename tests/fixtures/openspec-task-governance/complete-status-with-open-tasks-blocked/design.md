## Context

This fixture should fail only because the status section claims complete while
unchecked governed task IDs remain.

## Goals / Non-Goals

**Goals:**
- leave unchecked governed `I2` and `V1`
- keep a syntactically valid status section with the wrong complete-state shape

**Non-Goals:**
- missing status section
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Claim complete too early

**Choice:** Keep `I2` and `V1` unchecked but say `- Status: complete` and
`- Open tasks: none`.

**Rationale:** The gate should emit the exact complete-with-open-tasks finding.

**Implementation:** The fixture keeps one verification row so contradictory
status is the only intended failure.
