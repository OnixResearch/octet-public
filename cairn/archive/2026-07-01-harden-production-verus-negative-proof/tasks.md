## Contract

- [x] [serial] Specify production Verus negative proof behavior, including direct verifier rejection and no `formal_proof` provenance after failed production execution. r[octet.verus_proof_rails.production_negative]

## Implementation

- [x] [serial] Add an isolated invalid Verus proof target for failure-path evidence, with comments that it is not a correctness proof. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Add a production-mode negative proof fixture or opt-in rail that invokes `octet-production-verus` with explicit argv and cannot be selected by ordinary positive proof runs by accident. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Ensure `cargo octet prove` reports a deterministic verifier-failure diagnostic and writes only bounded failure artifacts for the negative production run. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Update docs or inline fixture notes so readers understand the negative proof tests fail-closed semantics only. r[octet.verus_proof_rails.production_negative]

## Verification

- [x] [serial] Add a direct negative wrapper test proving `octet-production-verus` rejects the invalid proof with non-zero status. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Add a `cargo octet prove` negative test asserting non-zero exit, deterministic diagnostic class, bounded stdout/stderr artifacts, and no `formal_proof` provenance. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Rerun existing positive fixture and production proof-rail checks to prove the negative fixture does not break normal proof runs. r[octet.verus_proof_rails.production_negative]
- [x] [serial] Run Cairn validation and gates for this change package before implementation and before archive. r[octet.verus_proof_rails.production_negative]
