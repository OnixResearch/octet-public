## ADDED Requirements

### Requirement: Shared proof and artifact identity core

r[octet.proof_artifact_library.shared_identity] Octet SHOULD centralize deterministic proof and artifact identity helpers, including BLAKE3 content identity and canonical closure graph normalization, in shared pure code.

#### Scenario: Same content has same identity

r[octet.proof_artifact_library.shared_identity.same_content]

- GIVEN runner and standards paths compute identity for the same artifact text or bytes
- WHEN they call the shared identity core
- THEN both paths SHOULD produce the same BLAKE3 identity string.

#### Scenario: Malformed closure graph fails deterministically

r[octet.proof_artifact_library.shared_identity.bad_graph]

- GIVEN a closure graph input is malformed or missing required canonical fields
- WHEN canonicalization runs
- THEN the shared core SHOULD return a deterministic diagnostic instead of accepting inconsistent identity.

### Requirement: Shared proof receipt binding decisions

r[octet.proof_artifact_library.proof_binding] Proof rail relevance, proof receipt source binding, proof artifact binding, and verifier identity checks SHOULD be implemented once in deterministic shared code and reused by command shells.

#### Scenario: Relevant proof rail is selected consistently

r[octet.proof_artifact_library.proof_binding.relevance]

- GIVEN the same policy registry and source files are evaluated by standards and runner code
- WHEN proof rail relevance is computed
- THEN both paths SHOULD select the same relevant proof rails.

#### Scenario: Receipt binding is stable

r[octet.proof_artifact_library.proof_binding.stable]

- GIVEN a proof receipt binds source, proof artifact, verifier, and closure identities
- WHEN binding validation runs repeatedly
- THEN the shared core SHOULD produce stable pass or fail diagnostics for equivalent inputs.

### Requirement: Fail-closed equivalence across command surfaces

r[octet.proof_artifact_library.fail_closed_equivalence] Refactors that deduplicate proof and artifact logic MUST preserve fail-closed behavior for stale hashes, missing bindings, unsupported schemas, malformed closure material, and missing provenance.

#### Scenario: Valid evidence still passes

r[octet.proof_artifact_library.fail_closed_equivalence.valid]

- GIVEN proof and artifact evidence satisfied the previous contract
- WHEN the shared core is used by runner and standards paths
- THEN the evidence MUST still pass without changing the accepted wire shape.

#### Scenario: Invalid evidence still fails

r[octet.proof_artifact_library.fail_closed_equivalence.invalid]

- GIVEN proof or artifact evidence has stale hashes, missing bindings, unsupported schemas, malformed closure material, or missing provenance
- WHEN validation runs through either command surface
- THEN validation MUST fail with deterministic diagnostics naming the rejected field or artifact.
