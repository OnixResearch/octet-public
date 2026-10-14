# Proof goal-state feedback

Octet goal-state feedback is a Lean-inspired obligation view for proof and
evidence review. It is emitted as deterministic data with schema
`octet-proof-goal-state/v1` and rendered in selected proof summaries so an
operator can see the current claim, assumptions, satisfied evidence, missing obligations,
blockers, remediation commands, and explicit non-claims in one place.

The goal state is computed from Octet facts: policy or requirement IDs, checked
scope, selected proof rail, proof receipt identity, proof card identity,
provenance identity, proof-ref diagnostics, artifact verification diagnostics,
and verifier outcome. Missing receipts, stale source or proof bindings, missing
proof cards, unsupported evidence tiers, unmatched proof refs, or overbroad claim
wording become blockers. A blocked goal emits no `formal_proof` provenance.

This is operator feedback, not a theorem-prover runtime. Its non-claims are:
not Lean execution; not solver completeness; not whole-program correctness; not
certification; not verified runtime proof. Lean is not required on the starter
`cargo octet check` path; proof rails remain opt-in and scoped to their validated
receipts and artifacts.
