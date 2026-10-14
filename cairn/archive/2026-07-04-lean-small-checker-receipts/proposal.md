## Why

Lean's trust story depends on a small kernel that checks proof objects rather than trusting every tactic that produced them. Octet should adapt that discipline for evidence receipts: producer tools may be rich and varied, but promotion-grade claims should be accepted only after a small deterministic checker validates the receipt, bindings, provenance, and non-claims.

Octet already has artifact verification and structured proof receipt validation. This change makes the producer/checker separation explicit across receipt families so reviewers can tell what generated an artifact and what independently accepted it.

## What Changes

- Define a small-checker receipt contract for promotion-grade evidence and formal-proof surfaces.
- Require receipts or provenance summaries to distinguish producer identity from checker identity when a checker validates the claim.
- Keep checker logic as pure deterministic cores over normalized in-memory inputs, with CLI shells only acquiring files and rendering reports.
- Add positive and negative tests proving producer-only assertions cannot promote evidence, stale bindings fail, and checker diagnostics are stable.

## Impact

- **Files**: artifact verifier cores, proof receipt validation, evidence receipt DTOs, provenance summaries, registry entries for checker IDs, generated contracts, and receipt fixtures.
- **Testing**: valid producer/checker receipt tests, negative self-asserted producer-only fixtures, stale/malformed binding fixtures, artifact verifier tests, generated-contract freshness, and Cairn validate/gates.
