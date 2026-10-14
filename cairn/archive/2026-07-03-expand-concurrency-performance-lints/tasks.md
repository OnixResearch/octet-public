## Contract

- [x] [serial] Specify the candidate ranking matrix for concurrency and performance lint expansion. r[octet.concurrency_performance_lints.candidate_matrix]
- [x] [serial] Specify contracts for the first accepted concurrency/performance lint candidates and their default levels. r[octet.concurrency_performance_lints.first_wave]

## Implementation

- [x] [serial] Add the candidate matrix to docs or change evidence with signal, false-positive risk, type-info needs, fixture complexity, and remediation clarity. r[octet.concurrency_performance_lints.candidate_matrix]
- [x] [serial] Implement the first high-signal concurrency lint selected by the matrix, including registration and metadata. r[octet.concurrency_performance_lints.first_wave]
- [x] [serial] Implement the first high-signal performance lint selected by the matrix, including registration and metadata. r[octet.concurrency_performance_lints.first_wave]
- [x] [parallel] Update README, lint reference, templates, and downstream rollout docs for new lint identities and defaults. r[octet.concurrency_performance_lints.first_wave]

## Verification

- [x] [parallel] Add positive and negative UI fixtures, including async/Tokio support where needed. r[octet.concurrency_performance_lints.first_wave]
- [x] [serial] Update fixture manifests and catalog freshness snapshots. r[octet.concurrency_performance_lints.first_wave]
- [x] [serial] Run `cargo test --lib`, UI rail, and Cairn validation/gates. r[octet.concurrency_performance_lints.candidate_matrix]
