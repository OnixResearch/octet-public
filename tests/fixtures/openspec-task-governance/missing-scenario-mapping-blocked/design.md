## Context

This fixture should fail only because one delta-spec scenario does not appear in
`## Verification Coverage`.

## Goals / Non-Goals

**Goals:**
- keep two scenario headings in the delta spec
- keep one matching coverage row and omit the target row only

**Non-Goals:**
- mixed `I<n>` task markers
- malformed `H<n>` checkpoint fields

## Decisions

### 1. Omit exactly one scenario row

**Choice:** Keep a valid `V1`/`V2` task pair but include only the second
scenario in `## Verification Coverage`.

**Rationale:** The gate should compare scenario headings against the coverage
matrix and report the missing target row by name.

**Implementation:** The target scenario is "Missing scenario mapping is
blocked". The second scenario keeps one valid row so the missing-row failure is
isolated.
