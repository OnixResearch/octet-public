## Contract

- [x] [serial] Specify the `missing_const_fn` conservative eligibility rules, advisory default level, and public-API guardrails. r[octet.purity_effect_lints.missing_const_fn]
- [x] [serial] Specify suggestion applicability and when no autofix is allowed. r[octet.purity_effect_lints.missing_const_fn]

## Implementation

- [x] [serial] Implement a pure const-eligibility classifier and lint shell for conservative function shapes. r[octet.purity_effect_lints.missing_const_fn]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.missing_const_fn.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for simple arithmetic, comparison, match, and value-construction helpers that can be `const fn`. r[octet.purity_effect_lints.missing_const_fn.flags_const_eligible]
- [x] [parallel] Add negative UI fixtures for async, allocation, non-const calls, trait methods, shell effects, public API guardrails, and already-const functions. r[octet.purity_effect_lints.missing_const_fn.allows_non_const]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, Cairn validation/gates, and autofix rail if a machine-applicable fix is provided. r[octet.purity_effect_lints.missing_const_fn.catalog]
