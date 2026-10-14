## Context

This fixture should fail only because one `V<n>` task claims two scenarios at
once.

## Goals / Non-Goals

**Goals:**
- keep two delta-spec scenarios
- keep one bundled `V1` task that names both scenarios
- keep coverage rows present so the V-task bundling is isolated

**Non-Goals:**
- mixed `I<n>` task markers
- missing coverage rows
- malformed `H<n>` checkpoint fields

## Decisions

### 1. Bundle the verification task, not the coverage rows

**Choice:** Keep two `## Verification Coverage` rows but map both rows to the
same bundled `V1` task.

**Rationale:** The task-governance contract says one `V<n>` task verifies exactly
one scenario.

**Implementation:** The offending task text names both scenarios directly.
