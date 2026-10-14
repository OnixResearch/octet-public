## Phase 1: Implementation

- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback] Inventory existing proof list, proof explain, proof-card, proof-ref, and artifact-verifier diagnostics that describe proof obligations.
- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback] Add a pure goal-state model that accepts typed claims, assumptions, receipts, cards, provenance, bindings, and verifier outcomes.
- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback] Extend selected CLI or machine-readable outputs to render satisfied evidence, missing obligations, blockers, remediation, and non-claims in stable order.
- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback.non_claim] Document that the surface is Lean-inspired obligation feedback, not Lean execution, solver completeness, or whole-program proof.

## Phase 2: Verification

- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback.satisfied] Verify a fully backed proof goal reports the claim, assumptions, receipt, proof card, provenance, checked scope, and zero blockers deterministically.
- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback.missing] Verify missing receipts, stale bindings, missing proof cards, unsupported evidence tiers, and unmatched proof refs report explicit blockers and emit no `formal_proof` provenance.
- [x] [serial] r[octet.verus_proof_rails.goal_state_feedback.non_claim] Verify forbidden wording checks reject Lean-required, solver-complete, whole-program-correctness, certification, and verified-runtime overclaims.
- [x] [serial] Run focused proof/proof-ref/artifact tests plus Cairn validate and proposal/design/tasks gates for this change package. [depends:cairn-gates]
