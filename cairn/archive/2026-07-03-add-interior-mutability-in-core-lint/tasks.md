## Contract

- [x] [serial] Specify the `interior_mutability_in_core` type table, default level, and core-scope behavior. r[octet.purity_effect_lints.interior_mutability_in_core]
- [x] [serial] Specify remediation language for explicit state transitions and shell-owned synchronization. r[octet.purity_effect_lints.interior_mutability_in_core]

## Implementation

- [x] [serial] Implement a pure interior-mutability classifier and lint shell for fields/construction in configured core scopes. r[octet.purity_effect_lints.interior_mutability_in_core]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.interior_mutability_in_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for core-scope `Cell`, `RefCell`, `UnsafeCell`, locks, and atomics. r[octet.purity_effect_lints.interior_mutability_in_core.flags_interior_mutability]
- [x] [parallel] Add negative UI fixtures for immutable fields, explicit state-transition structs, shell-owned synchronization, and test contexts. r[octet.purity_effect_lints.interior_mutability_in_core.allows_explicit_state]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.interior_mutability_in_core.catalog]
