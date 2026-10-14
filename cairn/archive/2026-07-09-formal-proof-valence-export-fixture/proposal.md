# Formal proof Valence export fixture

Octet already emits `formal_proof` provenance only for validated scoped proof receipts. The stack still needs a frozen export fixture that Valence and Cairn can consume without rerunning Verus, while preserving Octet's source-of-truth role for proof receipts.

## Scope

- Add a fixture artifact directory or manifest that represents a passed scoped Verus proof rail after `cargo octet prove` and `cargo octet artifact verify`.
- Export the fixture as Valence adapter input only after artifact verification succeeds.
- Add negative fixtures for missing proof receipt, stale proof-card identity, skipped artifact verification, imported Trellis promotion, missing non-claims, and widened release/correctness claims.
- Document that the export is an adapter input for Valence, not a new proof rail or release-readiness decision.

## Impact

- **Octet** owns the validated receipt, proof card, formal-proof provenance, and export precondition.
- **Valence** consumes the export for identity/linkage validation.
- **Cairn** may consume the downstream Valence graph/readiness summary but does not treat the Octet export alone as lifecycle approval.

## Non-claims

This change proves fixture export shape, artifact-verification preconditions, and proof-card/provenance linkage for the declared fixture only. It does not prove whole-program correctness, verifier soundness, semantic equivalence, dependency trust, release eligibility, or correctness of consumers that import the export.
