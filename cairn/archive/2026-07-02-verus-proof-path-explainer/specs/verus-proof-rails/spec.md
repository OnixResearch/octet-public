## ADDED Requirements

### Requirement: Verus proof-path explainer

r[octet.verus_proof_rails.proof_path_explainer] Octet SHOULD document the Verus formal-proof path as a scoped evidence lane from requirement or policy ID to artifact verification, while preserving proof-vs-validation and imported-vs-formal-proof tier boundaries.

#### Scenario: Proof path names every direct stage

r[octet.verus_proof_rails.proof_path_explainer.stages]

- GIVEN Octet documentation describes Verus proof rails for operators or reviewers
- WHEN a reader follows the direct Octet Verus proof path
- THEN the documentation SHOULD name the requirement or policy ID, Verus proof rail, checked Rust source scope, proof target, verifier/solver/toolchain identity, structured proof receipt, `formal_proof` provenance, and artifact-verifier stage.
- AND the documentation SHOULD link the stages to existing proof commands or artifact commands where those commands are the review surface.

#### Scenario: Imported Trellis branch stays content-addressed

r[octet.verus_proof_rails.proof_path_explainer.trellis_branch]

- GIVEN Octet documentation describes using Trellis proof artifacts as references
- WHEN a reader follows the imported proof branch
- THEN the documentation SHOULD name the Trellis repository revision, source artifact hash, proof artifact hash, bounded log hash, receipt hash when present, hash algorithm, imported bundle status, and optional Octet rerun stage.
- AND the documentation MUST state that BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness.
- AND the imported branch MUST remain external or `manual_review_required` unless Octet reruns the proof with pinned verifier identity and validates an Octet-owned receipt.

#### Scenario: Proof path preserves evidence tiers

r[octet.verus_proof_rails.proof_path_explainer.tiers]

- GIVEN the Verus proof-path explainer mentions proof, validation, lint, policy, transcript, imported Trellis bundle, or manual review evidence
- WHEN the wording assigns an evidence tier
- THEN validated Octet-owned Verus proof receipts MAY be described as supporting `formal_proof` provenance.
- AND lint findings MUST remain `static_lint`, policy checks MUST remain `policy_check`, transcript checks MUST remain `transcript_assertion`, imported Trellis bundles MUST remain external or `manual_review_required` until rerun, unresolved human obligations MUST remain `manual_review_required`, and validation evidence MUST NOT be relabeled as formal proof.

#### Scenario: Proof path stays non-claiming

r[octet.verus_proof_rails.proof_path_explainer.non_claiming]

- GIVEN documentation presents the Verus proof path
- WHEN documentation validation scans proof and correctness language
- THEN the proof claim MUST be scoped to the declared property, source binding, proof binding, verifier identity, receipt, and artifact identity.
- AND the documentation MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate evidence.
