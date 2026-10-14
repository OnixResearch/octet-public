## ADDED Requirements

### Requirement: Verus proof-path explainer

r[octet.verus_proof_rails.proof_path_explainer] Octet SHOULD document the Verus formal-proof path as a scoped evidence lane from requirement or policy ID to artifact verification, while preserving proof-vs-validation tier boundaries.

#### Scenario: Proof path names every stage

r[octet.verus_proof_rails.proof_path_explainer.stages]

- GIVEN Octet documentation describes Verus proof rails for operators or reviewers
- WHEN a reader follows the Verus proof path
- THEN the documentation SHOULD name the requirement or policy ID, Verus proof rail, checked Rust source scope, proof target, verifier/solver/toolchain identity, structured proof receipt, `formal_proof` provenance, and artifact-verifier stage.
- AND the documentation SHOULD link the stages to existing proof commands or artifact commands where those commands are the review surface.

#### Scenario: Proof path preserves evidence tiers

r[octet.verus_proof_rails.proof_path_explainer.tiers]

- GIVEN the Verus proof-path explainer mentions proof, validation, lint, policy, transcript, or manual review evidence
- WHEN the wording assigns an evidence tier
- THEN validated Verus proof receipts MAY be described as supporting `formal_proof` provenance.
- AND lint findings MUST remain `static_lint`, policy checks MUST remain `policy_check`, transcript checks MUST remain `transcript_assertion`, unresolved human obligations MUST remain `manual_review_required`, and validation evidence MUST NOT be relabeled as formal proof.

#### Scenario: Proof path stays non-claiming

r[octet.verus_proof_rails.proof_path_explainer.non_claiming]

- GIVEN documentation presents the Verus proof path
- WHEN documentation validation scans proof and correctness language
- THEN the proof claim MUST be scoped to the declared property, source binding, proof binding, verifier identity, receipt, and artifact identity.
- AND the documentation MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate evidence.
