## ADDED Requirements

### Requirement: Production Verus negative proof behavior

r[octet.verus_proof_rails.production_negative] Octet MUST prove that production-mode Verus proof failures fail closed before expanding production formal-proof scope.

#### Scenario: Invalid production proof is rejected

r[octet.verus_proof_rails.production_negative.direct_rejects]

- GIVEN an Octet-owned Verus proof fixture with an intentionally unprovable postcondition
- WHEN `octet-production-verus` runs that proof fixture through explicit argv
- THEN the verifier command MUST exit non-zero.
- AND the fixture MUST be labeled as failure-path evidence only, not as a valid formal proof.

#### Scenario: Failed production rail emits no formal proof

r[octet.verus_proof_rails.production_negative.no_formal_proof]

- GIVEN a selected production-mode Verus rail runs an invalid proof fixture
- WHEN `cargo octet prove` captures the failed verifier result
- THEN the proof run MUST report a deterministic verifier-failure diagnostic and return non-zero.
- AND the artifact directory MAY contain bounded failure stdout/stderr artifacts.
- AND Octet MUST NOT emit `formal_proof` provenance for the failed rail.

#### Scenario: Negative fixture cannot break ordinary proof runs

r[octet.verus_proof_rails.production_negative.isolated]

- GIVEN the repository contains an invalid Verus proof fixture for fail-closed testing
- WHEN an operator runs the normal positive production pilot proof scope
- THEN the invalid fixture MUST NOT be selected accidentally.
- AND the existing positive production proof rail MUST continue to validate with scoped `formal_proof` provenance.
