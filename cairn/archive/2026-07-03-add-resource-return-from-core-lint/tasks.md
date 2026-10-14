## Contract

- [x] [serial] Specify the `resource_return_from_core` resource-type table, default level, and core-scope behavior. r[octet.purity_effect_lints.resource_return_from_core]
- [x] [serial] Specify remediation language for command descriptions, domain values, and shell-owned capabilities. r[octet.purity_effect_lints.resource_return_from_core]

## Implementation

- [x] [serial] Implement a pure return-type resource classifier and late-lint shell for configured core scopes. r[octet.purity_effect_lints.resource_return_from_core]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.resource_return_from_core.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for core functions returning files, sockets, commands, join handles, channels, and connection/client handles. r[octet.purity_effect_lints.resource_return_from_core.flags_resource_return]
- [x] [parallel] Add negative UI fixtures for shell APIs, command-description enums, serializable plans, domain values, and injected ports. r[octet.purity_effect_lints.resource_return_from_core.allows_descriptions]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.resource_return_from_core.catalog]
