## MODIFIED Requirements

### Requirement: Proof evidence references

r[octet.verus_proof_rails.proof_evidence_ref] Octet SHOULD provide a Rust-owned `ProofEvidenceRef` data contract that links local claims, documentation references, or future lint suppressions to existing proof-card evidence without introducing a consumer-implementable proof trait.

#### Scenario: Reference fields are explicit

r[octet.verus_proof_rails.proof_evidence_ref.fields]

- GIVEN a `ProofEvidenceRef` JSON record is authored
- WHEN Octet validates the record shape
- THEN the record SHOULD include schema version, reference ID, consumer context, proof or property ID, source requirement or policy ID, checked Rust source scope, expected evidence tier, required status, required artifacts, and explicit non-claims.
- AND the record MUST NOT be represented as a Rust trait or marker that consumer code can implement to assert proof status.

#### Scenario: Reference is linkage not proof

r[octet.verus_proof_rails.proof_evidence_ref.boundary]

- GIVEN a `ProofEvidenceRef` validates against a proof card
- WHEN Octet reports the result
- THEN the report MUST state or encode that the reference validates linkage to existing proof-card evidence.
- AND the reference MUST NOT by itself emit `formal_proof` provenance, rerun Verus, certify a system, or broaden the proof beyond the matched proof card.

### Requirement: Proof-ref validation CLI

r[octet.verus_proof_rails.proof_evidence_ref.cli] Octet SHOULD expose a read-only `cargo octet proof-ref validate --proof-cards PATH --ref PATH` command that validates a proof evidence reference against a saved `proof-cards.json` deck.

#### Scenario: Matching reference validates

r[octet.verus_proof_rails.proof_evidence_ref.cli.matching]

- GIVEN a proof-card deck contains a card with the referenced proof or property ID, policy ID, checked scope, evidence tier, status, artifacts, and non-claims
- WHEN the operator runs `cargo octet proof-ref validate --proof-cards proof-cards.json --ref proof-ref.json`
- THEN the command SHOULD exit successfully.
- AND the human or JSON report SHOULD name the reference ID, proof or property ID, status, and zero diagnostics.

#### Scenario: Validation is read-only

r[octet.verus_proof_rails.proof_evidence_ref.cli.read_only]

- GIVEN an operator runs proof-ref validation
- WHEN the command finishes
- THEN it MUST NOT mutate the proof-card deck, proof receipts, provenance, source files, or artifact directory.
- AND it MUST NOT execute verifier commands.

### Requirement: Proof-ref diagnostics

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics] Proof evidence reference validation MUST fail closed with deterministic diagnostics when the referenced proof card is missing or mismatched.

#### Scenario: Missing or mismatched reference fails

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics.fail_closed]

- GIVEN a `ProofEvidenceRef` names a proof or property ID absent from the proof-card deck, or mismatches source requirement, checked scope, expected evidence tier, required status, required artifacts, or non-claims
- WHEN proof-ref validation runs
- THEN validation MUST fail closed with diagnostics naming the rejected field.
- AND validation MUST NOT upgrade manual-review or external proof-card evidence to `formal_proof`.

#### Scenario: Overbroad reference wording fails

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics.overclaim]

- GIVEN a `ProofEvidenceRef` non-claim or consumer-context field claims whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or verified-runtime behavior
- WHEN validation runs
- THEN validation MUST fail with a diagnostic naming the unsupported claim.
