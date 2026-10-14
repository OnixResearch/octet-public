## Context

The indexable-policy dogfood archive shows the current high-volume lints are valuable as inventory but too noisy for default enforcement in Octet's own lint code. The design goal is not to weaken the rules globally; it is to encode the known high-signal exceptions so the remaining findings are actionable.

## Decisions

### 1. Treat dogfood counts as first-class signal quality data

**Choice:** Add deterministic dogfood summaries that report raw findings, accepted debt, fixed debt, and residual findings per lint.

**Rationale:** Promotion decisions need evidence, not intuition. A lint with hundreds of accepted findings should stay inventory-only until its residual findings are explainable.

### 2. Add scoped exceptions, not broad disables

**Choice:** Implement exception classes for rustc-private implementation code, canonical local aliases (`rustc_hir as hir`), lint-identity-preserving names, and module/file layouts that intentionally mirror lint names.

**Rationale:** These are known product shapes, not arbitrary consumer preferences. Encoding them keeps the lints useful outside the narrow scope.

### 3. Keep the classifier pure

**Choice:** Extract pure classification helpers where each lint receives normalized facts such as import kind, owner path, module role, and configured exception classes.

**Rationale:** The exception matrix needs positive and negative unit tests without standing up rustc for every case.

## Risks / Trade-offs

- Overfitting to Octet internals could hide real consumer issues. Mitigation: exceptions must be explicitly scoped and covered by negative fixtures that prove ordinary consumer code still warns.
- New config keys can become a broad escape hatch. Mitigation: templates document cleanup notes and dogfood summaries count disabled/excepted findings separately.

## Validation

Run config tests, UI fixture rail, dogfood helper, and Cairn validation. Acceptance requires a saved dogfood summary showing lower residual noise or an explicit per-lint reason for remaining inventory-only status.
