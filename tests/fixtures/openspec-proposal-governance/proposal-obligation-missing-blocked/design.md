## Context

The proposal intentionally omits exact repo-relative paths, concrete fixture/scope membership, downstream validation boundaries, and exact evidence acceptance output.

## Goals / Non-Goals

**Goals:** show that proposal-gate review catches missing proposal obligations.
**Non-Goals:** runtime behavior changes.

## Decisions

### 1. Missing proposal details are intentionally not repaired here

**Choice:** Keep the design minimal so the proposal omission remains visible.
**Covers:** Proposal author records exact scope and downstream validation boundaries.
**Obligation cross-check:** The proposal does not name exact affected paths, concrete scope membership, downstream boundaries, or exact evidence output.
**Rationale:** This fixture should be blocked at proposal stage.
**Boundary cases:** none.
**Proof plan:** proposal gate should report a proposal omission.
