## Contract

- [x] [serial] Specify the `spawn_in_core` direct-call table, default level, and `core_source_scopes` behavior. r[octet.purity_effect_lints.spawn_in_core]
- [x] [serial] Specify remediation language for returned commands, work descriptions, or injected executor ports. r[octet.purity_effect_lints.spawn_in_core]

## Implementation

- [x] [serial] Implement a pure spawn-call classifier and late-lint shell for configured core scopes. r[octet.purity_effect_lints.spawn_in_core]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.spawn_in_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for `std::thread::spawn`, Tokio spawn APIs, and async-std spawn APIs in core scopes. r[octet.purity_effect_lints.spawn_in_core.flags_spawn]
- [x] [parallel] Add negative UI fixtures for shell-boundary spawns, returned command values, and injected executor ports. r[octet.purity_effect_lints.spawn_in_core.allows_shell_executor]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.spawn_in_core.catalog]
