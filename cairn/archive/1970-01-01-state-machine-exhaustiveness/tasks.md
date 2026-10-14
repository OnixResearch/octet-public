# Tasks

- [x] [serial] Define the marked state-machine transition-core contract and supported source-shape inventory. r[octet.state_machine_exhaustiveness.contract]
- [x] [serial] Implement the pure exhaustiveness decision core over parsed state/event variant and arm facts. r[octet.state_machine_exhaustiveness.matrix]
- [x] [serial] Add lint integration that reports wildcard fallback, unsupported marked shapes, mutation-coupled transitions, and missing explicit invalid-transition behavior. r[octet.state_machine_exhaustiveness.pure_core]
- [x] [serial] Add positive UI fixtures for explicit closed state/event matrices and negative UI fixtures for wildcard fallback, hidden invalid transitions, unsupported generated shapes, and mutation-driven state changes. r[octet.state_machine_exhaustiveness.fixtures]
- [x] [serial] Emit bounded source-shape receipts and Valence export rows with BLAKE3 identities, checker/source spans, status, diagnostics, and explicit non-claims. r[octet.state_machine_exhaustiveness.receipts]
- [x] [serial] Document consumer configuration, state-machine marker expectations, formal-proof boundaries, and the relationship to existing enum-match lints. r[octet.state_machine_exhaustiveness.docs]
- [x] [serial] Run focused lint tests, UI fixtures, policy validation, artifact verification checks, and Cairn validation/gates. r[octet.state_machine_exhaustiveness.final_validation]
