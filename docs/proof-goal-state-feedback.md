# Proof goal-state feedback

Octet goal-state feedback is a Lean-inspired obligation view for proof and
evidence review. It is emitted as deterministic data with schema
`octet-proof-goal-state/v1` and rendered in selected proof summaries so an
operator can see the current claim, assumptions, satisfied evidence, missing obligations,
blockers, remediation commands, and explicit non-claims in one place.

The goal state is computed from Octet facts. For each rail in a
`cargo octet prove` summary those are the policy ID, checked scope, selected
proof rail, verifier outcome and receipt validation diagnostics, the BLAKE3
identity of the receipt bytes the run writes, the receipt digest and locator its
`formal_proof` provenance record attests, and the diagnostics of the proof card
generated for the rail. Missing receipts or provenance, a provenance digest that
disagrees with the receipt bytes, stale source or proof bindings, verifier
failures, proof-card diagnostics (for example a card demoted to
`manual_review_required`), unsupported evidence tiers, or overbroad claim
wording become blockers. Proof-ref and artifact-verification diagnostics are not
inputs to the prove summary; run `cargo octet proof-ref validate` and
`cargo octet artifact verify` for those. A blocked goal emits no `formal_proof`
provenance: the rail's receipt and any Aeneas stage artifacts are still bound by
BLAKE3 in `provenance.jsonl`, but as `manual_review_required` records whose
`human_action` names the blockers. The rail status and its proof card's
evidence tier are `manual_review_required`, the blockers become run
diagnostics so the run status is `failed`, and `cargo octet artifact verify`
still accepts the bundle.
Its remediation commands include a runnable `cargo octet prove --explain` rerun
that carries the run's `--registry` or `--policy-artifacts-dir`, `--source`, and
`--family` arguments, selects the goal with `--rail <policy-id>#rail-<N>`, and
writes to a sibling `<artifact-dir>-explain` directory so the reviewed bundle is
not overwritten.

This is operator feedback, not a theorem-prover runtime. Its non-claims are:
not Lean execution; not solver completeness; not whole-program correctness; not
certification; not verified runtime proof. Lean is not required on the starter
`cargo octet check` path; proof rails remain opt-in and scoped to their validated
receipts and artifacts.
