# Verus Proof Rails Specification Delta

## ADDED Requirements

### Requirement: Proof goal-state feedback

r[octet.verus_proof_rails.goal_state_feedback] Octet SHOULD provide Lean-inspired proof goal-state feedback for selected proof and evidence review surfaces without requiring Lean or broadening formal-proof claims.

#### Scenario: Satisfied proof goal is explainable

r[octet.verus_proof_rails.goal_state_feedback.satisfied]

- GIVEN a selected proof or proof-reference claim has matching source scope, proof binding, verifier identity, structured receipt, proof card, provenance record, and artifact identity
- WHEN the operator requests proof goal-state feedback
- THEN the output SHOULD name the checked claim, assumptions, required evidence, satisfied evidence, verifier mode, checked scope, receipt path, proof-card identity, provenance identity, and explicit non-claims in deterministic order.
- AND the output SHOULD make clear that the formal-proof claim is scoped to the matched receipt and artifact identities.

#### Scenario: Missing proof obligation is a blocker

r[octet.verus_proof_rails.goal_state_feedback.missing]

- GIVEN a proof or proof-reference claim has a missing receipt, stale source or proof binding, absent proof card, unsupported evidence tier, unmatched source anchor, or verifier failure
- WHEN proof goal-state feedback is computed
- THEN the output MUST report an explicit blocker naming the missing or stale obligation and the remediation surface when one exists.
- AND Octet MUST NOT emit `formal_proof` provenance for the blocked claim.

#### Scenario: Goal-state feedback remains non-claiming

r[octet.verus_proof_rails.goal_state_feedback.non_claim]

- GIVEN proof goal-state feedback mentions Lean inspiration, goals, hypotheses, holes, proof, verification, or correctness
- WHEN documentation and output claim-boundary checks run
- THEN the wording MUST NOT imply that Lean is required, that Octet performs unbounded proof search, that every obligation has a solver-complete answer, or that a scoped receipt proves whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime.
