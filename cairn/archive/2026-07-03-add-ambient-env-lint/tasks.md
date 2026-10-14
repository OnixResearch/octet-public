## Contract

- [x] [serial] Specify the `ambient_env` API list, default level, test-context behavior, and shell-boundary suppression guidance. r[octet.purity_effect_lints.ambient_env]
- [x] [serial] Specify diagnostic remediation that replaces env access with explicit config/path/argument parameters. r[octet.purity_effect_lints.ambient_env]

## Implementation

- [x] [serial] Implement a pure env-path classifier and late-lint shell for direct `std::env` calls. r[octet.purity_effect_lints.ambient_env]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.ambient_env.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for env var reads, process args, current-dir reads, and env mutation in production code. r[octet.purity_effect_lints.ambient_env.flags_env]
- [x] [parallel] Add negative UI fixtures for explicit config parameters, test contexts, and justified shell-boundary helpers. r[octet.purity_effect_lints.ambient_env.allows_explicit_input]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.ambient_env.catalog]
