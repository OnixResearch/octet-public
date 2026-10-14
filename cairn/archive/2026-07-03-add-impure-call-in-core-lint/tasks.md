## Contract

- [x] [serial] Specify the `impure_call_in_core` effect-family list, default `warn` level, and `core_source_scopes` behavior. r[octet.purity_effect_lints.impure_call_in_core]
- [x] [serial] Specify remediation language for replacing direct platform calls with injected ports or shell-boundary helpers. r[octet.purity_effect_lints.impure_call_in_core]

## Implementation

- [x] [serial] Implement a pure effect-path classifier and a thin late-lint shell for direct calls in core scopes. r[octet.purity_effect_lints.impure_call_in_core]
- [x] [parallel] Register the lint, metadata, domain group membership, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.impure_call_in_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for direct `fs`, `io`, `net`, `process`, `env`, `random`, and runtime calls in configured core scopes. r[octet.purity_effect_lints.impure_call_in_core.flags_effect]
- [x] [parallel] Add negative UI fixtures for shell-scope code and core code that consumes injected traits/capabilities without constructing effects. r[octet.purity_effect_lints.impure_call_in_core.allows_boundary]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.impure_call_in_core.catalog]
