## Context

This fixture should pass because every governed checkbox is checked and the
status section says complete with no open tasks.

## Goals / Non-Goals

**Goals:**
- keep all governed tasks checked
- keep `Status: complete` plus `Open tasks: none`
- keep `## Change Status` immediately before `## Verification Coverage`

**Non-Goals:**
- malformed status bullets
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Check every governed task

**Choice:** Keep `I1`, `V1`, and `H1` checked and use the exact complete
status shape.

**Rationale:** The gate should accept `complete` only when no governed task
remains open.

**Implementation:** The fixture keeps one verification row and one human row so
all governed classes can be fully checked.
