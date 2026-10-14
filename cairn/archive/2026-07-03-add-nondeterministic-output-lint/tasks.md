## Contract

- [x] [serial] Specify the `nondeterministic_output` local-flow shapes, advisory/default level, and sorting guardrails. r[octet.purity_effect_lints.nondeterministic_output]
- [x] [serial] Specify remediation language for stable sort keys or ordered collection types. r[octet.purity_effect_lints.nondeterministic_output]

## Implementation

- [x] [serial] Implement pure local-flow classification for unordered iteration to ordered outputs. r[octet.purity_effect_lints.nondeterministic_output]
- [x] [parallel] Register the lint, metadata, docs, templates, and downstream rollout notes. r[octet.purity_effect_lints.nondeterministic_output.catalog]

## Verification

- [x] [parallel] Add positive UI fixtures for unsorted `HashMap`/`HashSet` iteration collected into ordered output. r[octet.purity_effect_lints.nondeterministic_output.flags_unsorted_hash_output]
- [x] [parallel] Add negative UI fixtures for sorted output, `BTreeMap`/`BTreeSet`, explicit unordered returns, and test contexts. r[octet.purity_effect_lints.nondeterministic_output.allows_stable_order]
- [x] [serial] Update fixture manifests and run `cargo test --lib`, the ignored UI rail, and Cairn validation/gates. r[octet.purity_effect_lints.nondeterministic_output.catalog]
