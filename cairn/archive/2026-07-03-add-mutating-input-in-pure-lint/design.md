## Context

Functional core code can mutate local temporaries, but mutating caller-owned state through a signature weakens purity. The lint should inspect function signatures in declared pure contexts rather than trying to infer every internal mutation.

## Decisions

### 1. Start with signature hazards

**Choice:** Flag `&mut` parameters, raw pointer parameters, and parameters typed as locks, atomics, cells, or unsafe cells in configured pure contexts.

**Rationale:** Signature shapes are high signal and easy to explain.

### 2. Define pure context consistently

**Choice:** Initially use `core_source_scopes`; later integrate with `#[tigerstyle::pure]` if the pure-attribute contract lands.

**Rationale:** This lets the lint ship independently while preserving a path to stricter opt-in contracts.

### 3. Allow shell APIs and owned transitions

**Choice:** Do not emit for shell scopes, functions that take owned state and return new state, or local mutation that does not escape.

**Rationale:** The goal is explicit state modeling, not a ban on local variables.

## Risks / Trade-offs

- Some algorithms use `&mut` buffers for performance. Core-scope suppressions should explain why mutation is still deterministic and bounded.
- Trait method signatures may be externally imposed. Diagnostics should be advisory unless rollout evidence supports stricter defaults.

## Validation

Add signature-focused fixtures with both positive and negative coverage, plus pure classifier unit tests.
