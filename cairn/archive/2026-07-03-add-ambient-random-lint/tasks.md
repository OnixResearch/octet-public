## Contract

- [x] [serial] Specify the `ambient_random` direct-call table, default level, and deterministic-remediation language. r[octet.purity_effect_lints.ambient_random]
- [x] [serial] Specify non-emitting shapes for injected RNG traits, seeds, nonce bytes, and generated IDs. r[octet.purity_effect_lints.ambient_random]

## Implementation

- [x] [serial] Implement a pure random-source classifier and late-lint shell for direct randomness calls. r[octet.purity_effect_lints.ambient_random]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.ambient_random.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for thread-local RNGs, OS randomness, `getrandom`, and UUID v4 generation. r[octet.purity_effect_lints.ambient_random.flags_random]
- [x] [parallel] Add negative UI fixtures for injected RNG/capability parameters, explicit seeds, and test contexts. r[octet.purity_effect_lints.ambient_random.allows_injected_randomness]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.ambient_random.catalog]
