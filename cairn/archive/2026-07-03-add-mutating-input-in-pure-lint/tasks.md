## Contract

- [x] [serial] Specify the `mutating_input_in_pure` hazardous-parameter table, default level, and declared-pure context rules. r[octet.purity_effect_lints.mutating_input_in_pure]
- [x] [serial] Specify remediation for owned state transitions, return values, or shell placement. r[octet.purity_effect_lints.mutating_input_in_pure]

## Implementation

- [x] [serial] Implement a pure signature classifier and late-lint shell for functions in declared pure contexts. r[octet.purity_effect_lints.mutating_input_in_pure]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.mutating_input_in_pure.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for `&mut`, raw pointers, locks, atomics, cells, and unsafe-cell parameters in pure contexts. r[octet.purity_effect_lints.mutating_input_in_pure.flags_mutating_input]
- [x] [parallel] Add negative UI fixtures for owned state transitions, immutable borrows, local mutation, shell APIs, and test contexts. r[octet.purity_effect_lints.mutating_input_in_pure.allows_explicit_transition]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.mutating_input_in_pure.catalog]
