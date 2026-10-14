## Context

This fixture should fail only because one `H<n>` task omits `evidence=`.

## Goals / Non-Goals

**Goals:**
- keep one malformed `H1` task
- keep one matching coverage row so the malformed field is isolated

**Non-Goals:**
- mixed `I<n>` task markers
- scenario-row omissions

## Decisions

### 1. Omit the evidence field only

**Choice:** Keep `command=`, `input=`, and `asserts=` but omit `evidence=`.

**Rationale:** The task-governance contract requires all four replay fields.

**Implementation:** The fixture uses one `H1` task plus one `## Verification Coverage` row for scenario "Human checkpoint missing evidence path is blocked".
