## Context

This fixture should pass because one `H<n>` task includes all four replay
fields.

## Goals / Non-Goals

**Goals:**
- keep one well-formed `H1` task
- keep one matching coverage row so the checkpoint shape is isolated

**Non-Goals:**
- mixed `I<n>` task markers
- scenario-row omissions

## Decisions

### 1. Fill every replay field

**Choice:** Keep `command=`, `input=`, `asserts=`, and `evidence=` in one `H1`
checkpoint.

**Rationale:** The task-governance contract requires all four replay fields.

**Implementation:** The fixture uses one `H1` task plus one `## Verification Coverage` row for scenario "Aspen replay task is self-describing".
