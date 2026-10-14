## Context

This fixture should pass because unchecked governed task IDs appear in the
status section exactly once and in file order.

## Goals / Non-Goals

**Goals:**
- keep unchecked `I2`, `V1`, and `H1`
- keep `## Change Status` immediately before `## Verification Coverage`
- keep one scenario row for `V1` and one scenario row for `H1`

**Non-Goals:**
- malformed status bullets
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Leave three governed tasks open in file order

**Choice:** Keep `I2`, `V1`, and `H1` unchecked and list them as
`I2, V1, H1` in `## Change Status`.

**Rationale:** The gate should accept an in-progress status section when it
matches unchecked governed task IDs exactly.

**Implementation:** The fixture uses one implementation task, one verification
row, and one human/oracle row so all three governed task classes appear.
