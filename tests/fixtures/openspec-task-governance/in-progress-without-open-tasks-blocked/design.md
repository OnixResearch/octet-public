## Context

This fixture should fail only because the status section says in-progress even
though every governed task is checked.

## Goals / Non-Goals

**Goals:**
- keep every governed task checked
- keep the wrong in-progress status shape with no open tasks

**Non-Goals:**
- missing status section
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Leave no open tasks but keep in-progress status

**Choice:** Check every governed task and still say `- Status: in-progress` and
`- Open tasks: none`.

**Rationale:** The gate should emit the exact in-progress-without-open-tasks
finding.

**Implementation:** The fixture keeps one verification row so contradictory
status is the only intended failure.
