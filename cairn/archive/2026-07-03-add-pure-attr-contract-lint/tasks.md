## Contract

- [x] [serial] Specify recognized pure-marker syntax, namespace compatibility, default level, and direct-effect limits. r[octet.purity_effect_lints.pure_attr_contract]
- [x] [serial] Specify the banned direct-effect checklist and diagnostic wording that avoids overclaiming transitive proof. r[octet.purity_effect_lints.pure_attr_contract]

## Implementation

- [x] [serial] Implement pure marker parsing and a lint shell that reuses shared effect/signature classifiers for marked functions. r[octet.purity_effect_lints.pure_attr_contract]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.pure_attr_contract.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for marked functions with platform IO, time, env, random, logging, spawning, global/interior mutation, mutating input, panic, and resource-handle violations. r[octet.purity_effect_lints.pure_attr_contract.flags_marked_impurity]
- [x] [parallel] Add negative UI fixtures for clean marked pure functions, unmarked shell functions, explicit inputs/outputs, and required `register_tool` setup. r[octet.purity_effect_lints.pure_attr_contract.allows_clean_marked_pure]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.pure_attr_contract.catalog]
