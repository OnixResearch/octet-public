## Context

This fixture should pass because each scenario keeps its own `V<n>` task and
its own coverage row even though both rows point at one shared artifact.

## Goals / Non-Goals

**Goals:**
- keep two delta-spec scenarios
- keep one `V<n>` task per scenario
- keep two separate coverage rows that share one artifact path

**Non-Goals:**
- mixed `I<n>` task markers
- missing coverage rows
- malformed `H<n>` checkpoint fields

## Decisions

### 1. Share the artifact, not the row

**Choice:** Point both coverage rows at the same artifact path while keeping
separate `V1` and `V2` tasks.

**Rationale:** The task-governance contract allows shared evidence artifacts but
still requires one row per scenario.

**Implementation:** The two rows share one repo-relative path under
`openspec/changes/example/evidence/shared-evidence-pass.txt`.
