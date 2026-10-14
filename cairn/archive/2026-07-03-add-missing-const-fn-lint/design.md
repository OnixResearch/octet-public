## Context

Octet cannot prove arbitrary Rust purity, but `const fn` can encode a useful subset. This lint should be conservative: suggest `const` only when the source shape is clearly compatible.

## Decisions

### 1. Make the lint advisory first

**Choice:** Default `missing_const_fn` to `allow` or another advisory level until dogfood evidence shows low noise.

**Rationale:** Rust const rules evolve and heuristic suggestions can be wrong.

### 2. Restrict to simple bodies

**Choice:** Start with functions using literals, parameters, arithmetic, comparisons, boolean logic, `if`, `match`, and tuple/struct construction over primitive or copy-like values.

**Rationale:** This provides a safe high-signal subset.

### 3. Avoid public API churn by default

**Choice:** Prefer private functions or explicitly configured scopes first; public API suggestions need rollout evidence.

**Rationale:** Adding `const` can be a public promise and should not be made casually.

## Risks / Trade-offs

- Compiler const-compatibility may differ from heuristic matching. Use `MaybeIncorrect` suggestions unless proven by compile-fail/fix rails.
- Some pure helpers intentionally avoid `const` to keep APIs flexible.

## Validation

Use positive/negative UI fixtures and, if autofix is offered, prove fixed code compiles and remains idempotent.
