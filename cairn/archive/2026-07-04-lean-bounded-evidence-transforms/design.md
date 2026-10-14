## Context

Lean tactics and `simp` can hide large amounts of search and rewriting. Octet needs the opposite default: explicit, bounded transformations whose output can be reviewed and validated. A transform is acceptable when it is deterministic, scoped to a named family, and records enough identity to reproduce or reject its result.

## Decisions

### 1. Transforms are pure cores with declared contracts

**Choice:** Each transform SHOULD be a pure deterministic operation over typed inputs with a named transform ID, accepted input schema, output schema, rule set, budget, diagnostics, and non-claims.

**Rationale:** Reviewers can reason about a transform only when its boundaries and failure modes are explicit.

### 2. Rewrite sets are explicit and versioned

**Choice:** Normalization should use named rule sets rather than hidden global simplification. Rule-set identity SHOULD be recorded where transformed artifacts support evidence claims.

**Rationale:** Lean-style simplification is useful only if Octet can explain exactly what was rewritten.

### 3. Transformed evidence carries parent provenance

**Choice:** When a transform emits a derived artifact, provenance SHOULD name the transform ID, rule-set identity, input artifacts, output artifact, evidence tier, and non-claims.

**Rationale:** Derived evidence must remain traceable back to the material it summarizes or normalizes.

### 4. Budgets fail closed

**Choice:** Transform budgets, recursion limits, input-size limits, and unsupported rule cases MUST fail closed with deterministic diagnostics rather than silently producing partial evidence.

**Rationale:** Bounded automation is safe only when exceeding the bound is visible.

## Risks / Trade-offs

- Too many transform IDs can fragment the model; start with high-value evidence normalization surfaces.
- Rule-set versioning can cause fixture churn; that churn is useful when claim-affecting rewrites change.
- Transforms must not imply that normalized evidence is complete, formally proved, or semantically equivalent unless separately checked.
