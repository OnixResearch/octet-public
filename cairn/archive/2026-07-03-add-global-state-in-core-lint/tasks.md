## Contract

- [x] [serial] Specify the `global_state_in_core` declaration shapes, default level, and `core_source_scopes` behavior. r[octet.purity_effect_lints.global_state_in_core]
- [x] [serial] Specify non-emitting shapes for constants, immutable statics, shell-owned singletons, and explicit state parameters. r[octet.purity_effect_lints.global_state_in_core]

## Implementation

- [x] [serial] Implement a pure global-state declaration classifier and early/late lint shell for configured core scopes. r[octet.purity_effect_lints.global_state_in_core]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.global_state_in_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for `static mut`, `thread_local!`, `OnceLock`, `LazyLock`, global locks, and global atomics in core scopes. r[octet.purity_effect_lints.global_state_in_core.flags_global]
- [x] [parallel] Add negative UI fixtures for constants, immutable statics, explicit state threading, and shell-boundary singletons. r[octet.purity_effect_lints.global_state_in_core.allows_explicit_state]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.global_state_in_core.catalog]
