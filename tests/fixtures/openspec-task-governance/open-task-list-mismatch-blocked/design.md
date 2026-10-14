## Context

This fixture should fail only because `Open tasks` does not match unchecked
governed task IDs.

## Goals / Non-Goals

**Goals:**
- leave unchecked governed `I2` and `V1`
- keep an in-progress status section with an incomplete open-task list

**Non-Goals:**
- missing status section
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Omit one unchecked ID from the list

**Choice:** Keep `I2` and `V1` unchecked but list only `I2` in `Open tasks`.

**Rationale:** The gate should emit the exact open-task-list-mismatch finding.

**Implementation:** The fixture keeps one verification row so the mismatched
list is the only intended failure.
