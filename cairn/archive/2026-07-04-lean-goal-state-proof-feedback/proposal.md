## Why

Lean's most transferable proof UX is not its syntax; it is the way a user can see the current goal, hypotheses, missing obligations, and exact blockers. Octet already has proof cards, proof-ref validation, and `cargo octet prove --explain`, but reviewers still need a single obligation-state view that answers: what claim is being checked, what assumptions are in scope, which evidence artifacts satisfy it, and what remains missing.

The goal is a Lean-inspired goal-state model for Octet proof and evidence surfaces. It should improve operator feedback without making Lean mandatory, broadening formal-proof claims, or hiding evidence behind unbounded automation.

## What Changes

- Define a deterministic proof/evidence goal-state model for selected Verus proof rails, proof-card decks, proof-ref validation, and artifact verification diagnostics.
- Show claims, assumptions, source and proof bindings, required artifacts, satisfied evidence, missing obligations, blockers, remediation commands, and non-claims in stable order.
- Keep the model in pure core code over typed in-memory inputs; CLI shells render human, JSON, or SARIF-facing output.
- Add positive and negative tests for satisfied goals, missing receipts, stale bindings, unsupported evidence tiers, and overbroad claims.

## Impact

- **Files**: `cargo-octet/src/` proof/proof-ref/artifact output cores, proof-card validation helpers, docs for proof operator feedback, and fixtures under proof/evidence tests.
- **Testing**: pure goal-state unit tests, CLI rendering snapshots or structured-output tests, negative stale/missing/overclaim cases, existing proof rails, artifact verification tests, and Cairn validate/gates.
