## Context

The proposal explicitly states exact artifacts, scope, downstream boundaries, non-goals, and evidence outputs.

## Goals / Non-Goals

**Goals:** prove a fully covered proposal can pass proposal-gate review.
**Non-Goals:** changing runtime lint behavior or historical archives.

## Decisions

### 1. Proposal obligations are carried forward

**Choice:** Keep proposal-stage obligations explicit and mirror them in spec/design proof planning.
**Covers:** Proposal author records exact scope and downstream validation boundaries; Proposal governance fixtures prove blocked and covered obligations.
**Obligation cross-check:** The proposal names exact paths, concrete fixture scope, downstream validation boundaries, non-goals, and exact expected evidence output.
**Rationale:** Passing fixture must demonstrate the intended authoring shape.
**Boundary cases:** Excludes runtime lint/object corpus changes and historical archive rewrites.
**Proof plan:** proposal fixture runner prints `PASS expected passing fixture: proposal-obligation-covered-pass`.
