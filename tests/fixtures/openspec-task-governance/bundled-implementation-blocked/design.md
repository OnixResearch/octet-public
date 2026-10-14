## Context

This fixture should fail only because one `I<n>` task mixes implementation with
saved-evidence capture.

## Goals / Non-Goals

**Goals:**
- keep one bad `I<n>` checkbox with a saved-artifact claim
- keep verification coverage otherwise complete

**Non-Goals:**
- scenario-row omissions
- malformed human/oracle checkpoints

## Decisions

### 1. Keep the failure structural

**Choice:** Use one `I1` checkbox whose text includes `save` plus a repo-relative
`evidence/` path.

**Rationale:** The repo task-governance contract treats those markers as a mixed
implementation/evidence claim.

**Implementation:** The fixture tasks file keeps a valid `V1` task and one
matching `## Verification Coverage` row so the mixed `I1` checkbox is the only
intended blocker.
