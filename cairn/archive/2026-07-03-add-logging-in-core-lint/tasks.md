## Contract

- [x] [serial] Specify the `logging_in_core` macro/call table, default level, expansion handling, and core-scope behavior. r[octet.purity_effect_lints.logging_in_core]
- [x] [serial] Specify remediation language for returned events, observations, or shell-owned telemetry ports. r[octet.purity_effect_lints.logging_in_core]

## Implementation

- [x] [serial] Implement pure observability-name classifiers and lint shells for macro/call sites in configured core scopes. r[octet.purity_effect_lints.logging_in_core]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.logging_in_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for `println!`, `eprintln!`, `dbg!`, logging, tracing, and metrics emission in core scopes. r[octet.purity_effect_lints.logging_in_core.flags_logging]
- [x] [parallel] Add negative UI fixtures for shell logging, returned events/observations, telemetry ports, and test-only output. r[octet.purity_effect_lints.logging_in_core.allows_events]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.logging_in_core.catalog]
