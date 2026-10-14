## Contract

- [x] [serial] Specify evidence gates for promoting allow-by-default lints to profile membership or global `warn`. r[octet.advisory_lint_promotion.evidence_gate]
- [x] [serial] Specify migration-note and rollback requirements for promoted lints. r[octet.advisory_lint_promotion.migration]

## Implementation

- [x] [serial] Add a promotion candidate table for `raw_arithmetic_overflow`, `ambiguous_params`, `multi_lock_ordering`, `defaulting_trait_method`, and `numeric_units`. r[octet.advisory_lint_promotion.evidence_gate]
- [x] [serial] For each accepted candidate, update default levels or rollout profiles only after evidence gates pass. r[octet.advisory_lint_promotion.evidence_gate]
- [x] [parallel] Update lint metadata, docs, templates, and downstream rollout migration notes for any promotion. r[octet.advisory_lint_promotion.migration]

## Verification

- [x] [parallel] Save dogfood/downstream summaries with raw, accepted, fixed, and residual counts for each candidate. r[octet.advisory_lint_promotion.evidence_gate]
- [x] [serial] Run default-level snapshot tests, catalog freshness tests, UI rail, and config override tests. r[octet.advisory_lint_promotion.migration]
- [x] [serial] Run Cairn validation/gates. r[octet.advisory_lint_promotion.evidence_gate]
