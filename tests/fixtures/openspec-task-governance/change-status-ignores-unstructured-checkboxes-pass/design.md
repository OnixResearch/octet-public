## Context

This fixture should pass because only governed `I<n>`, `V<n>`, and `H<n>` task
IDs contribute to `Open tasks`.

## Goals / Non-Goals

**Goals:**
- keep unchecked governed `I2` and `V1`
- include ordinary markdown checkboxes that stay out of the status list
- keep one matching verification-coverage row

**Non-Goals:**
- malformed status bullets
- missing verification coverage
- mixed implementation/proof task text

## Decisions

### 1. Mix governed and unstructured checkboxes

**Choice:** Keep one unchecked ordinary checkbox alongside governed tasks, but
list only `I2, V1` in `## Change Status`.

**Rationale:** The gate should ignore ordinary markdown checkboxes.

**Implementation:** The fixture keeps one scenario row for `V1` and leaves the
ordinary checkbox outside governed task parsing.
