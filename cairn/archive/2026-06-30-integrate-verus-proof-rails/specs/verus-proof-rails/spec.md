## ADDED Requirements

### Requirement: Verus proof-rail contract

r[octet.verus_proof_rails.contract] Octet SHOULD support Verus as an opt-in formal-proof rail that is scoped to declared source paths, proof paths, verifier identity, pass signal, and trust boundaries.

#### Scenario: Pinned Verus rail is declared

r[octet.verus_proof_rails.contract.declared]

- GIVEN a policy declares a Verus proof rail
- WHEN Octet exports or validates the policy registry
- THEN the rail MUST declare `backend = "verus"`, a deterministic command, source paths, proof paths, an exact pass signal, and trust-boundary notes.
- AND the rail MUST NOT rely on an ambient unpinned `verus` binary as verifier identity for `formal_proof` evidence.

#### Scenario: Proof claim remains scoped

r[octet.verus_proof_rails.contract.scoped]

- GIVEN a Verus proof rail passes
- WHEN Octet renders proof evidence or documentation for that rail
- THEN the evidence MUST name the checked source scope, proof artifact identity, verifier identity, and covered property or policy ID.
- AND the evidence MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety without separate evidence.

### Requirement: Persistent Verus proof receipts

r[octet.verus_proof_rails.receipts] Octet MUST persist proof receipts for executed Verus rails when the caller requests proof artifacts, and those receipts MUST be validated before they can support `formal_proof` provenance.

#### Scenario: Matching proof receipt passes

r[octet.verus_proof_rails.receipts.matching]

- GIVEN a Verus proof rail has a persisted receipt containing the configured pass signal
- AND the receipt contains matching BLAKE3 command and source bindings when the rail enables command/source binding
- AND the receipt contains configured verifier, provenance, and trust-boundary lines
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the proof artifact.

#### Scenario: Stale or incomplete proof receipt fails

r[octet.verus_proof_rails.receipts.fail_closed]

- GIVEN a Verus proof receipt is missing the configured pass signal, has a stale source BLAKE3 binding, has wrong verifier identity, omits a required trust-boundary line, or omits proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the offending receipt field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Proof artifact verifier is read-only

r[octet.verus_proof_rails.receipts.artifact_verify]

- GIVEN an operator has a saved proof artifact directory
- WHEN `cargo octet artifact verify` validates proof receipts and provenance
- THEN the verifier MUST recompute BLAKE3 identities and validate provenance coverage without mutating the artifact directory.
- AND malformed, missing, stale, or unmatched proof artifacts MUST fail closed.

### Requirement: Proof command surface

r[octet.verus_proof_rails.prove_command] Octet SHOULD expose a `cargo octet prove` command that runs selected proof rails and writes reviewable proof artifacts without changing ordinary lint-check behavior.

#### Scenario: Prove command writes scoped artifacts

r[octet.verus_proof_rails.prove_command.artifacts]

- GIVEN a workspace declares a selected Verus proof rail
- WHEN the operator runs `cargo octet prove` with an artifact directory
- THEN Octet SHOULD run the pinned verifier command, write a proof receipt, write provenance with `evidence_tier = formal_proof` only for validated proof receipts, and report the checked scope.
- AND the command SHOULD return a non-zero status when any selected proof rail fails validation.

#### Scenario: Starter lint path remains unaffected

r[octet.verus_proof_rails.prove_command.starter_unaffected]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile with no selected proof rails
- WHEN the check completes
- THEN Octet MUST NOT require Verus to be installed or executed.
- AND the absence of Verus artifacts MUST NOT be rendered as a formal-proof failure for that starter check.

### Requirement: Verus proof registry ownership

r[octet.verus_proof_rails.registry] Verus proof receipt and verifier identity families MUST be registered in `standards/policies/registry.ncl` with Rust-derived ownership or explicit non-candidate rationale.

#### Scenario: Registry covers proof receipt family

r[octet.verus_proof_rails.registry.covered]

- GIVEN Octet emits Verus proof receipts after execution
- WHEN config/receipt family validation runs
- THEN the Verus proof receipt family MUST be registered as Rust-derived.
- AND generated policy artifacts MUST include the proof-rail fields needed by downstream receipt validators.

#### Scenario: Raw verifier internals remain excluded

r[octet.verus_proof_rails.registry.non_candidate]

- GIVEN verifier internals, raw logs, raw environment state, or secrets are encountered during proof execution
- WHEN registry ownership is reviewed
- THEN those raw surfaces MUST be excluded from Nickel-owned config and receipts except as bounded paths, labels, digests, allowlists, or explicit non-candidate rationale.

### Requirement: Verified-module proof coverage

r[octet.verus_proof_rails.verified_module_coverage] Octet verification profiles MUST make unbacked `verified/` module or formal-proof claims explicit instead of silently treating purity lint success as proof.

#### Scenario: Covered verified module passes

r[octet.verus_proof_rails.verified_module_coverage.covered]

- GIVEN a source file under `verified/` is selected by a verification profile
- AND the policy registry declares a matching Verus proof rail for that source file
- WHEN verification-profile checks run
- THEN Octet MUST accept the verified-module coverage obligation only after the proof receipt validates.

#### Scenario: Unbacked verified claim is manual review or failure

r[octet.verus_proof_rails.verified_module_coverage.unbacked]

- GIVEN a source file, document, or artifact claims formal verification for a module but no matching proof rail or validated proof receipt exists
- WHEN verification-profile checks run
- THEN Octet MUST either fail the selected verification gate or emit `manual_review_required` provenance with a human-action message, according to the selected profile.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.
