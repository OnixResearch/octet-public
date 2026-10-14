# Evidence Provenance Specification

## Purpose

Add a generic Octet producer handoff for stack-wide evidence chains.

## Requirements

### Requirement: Octet evidence-chain handoffs are typed
r[octet.evidence_provenance.evidence_chain_handoff.contract] Octet MUST provide a generic evidence-chain producer handoff envelope with typed evidence kind, producer identity, run identity, canonical roots, parent links, policy/profile hashes, projections, and non-claim boundaries.

#### Scenario: Function-address profile uses generic handoff
r[octet.evidence_provenance.evidence_chain_handoff.function_address]
- GIVEN Octet exports function-address evidence
- WHEN the export is marked for stack-chain consumption
- THEN Octet MUST wrap the function-address profile payload in the generic producer handoff envelope.

### Requirement: Handoff validation fails closed
r[octet.evidence_provenance.evidence_chain_handoff.core] Octet MUST validate evidence-chain handoff fields before treating an output as ready for Kamacite canonicalization.

#### Scenario: Invalid handoff is rejected
r[octet.evidence_provenance.evidence_chain_handoff.negative]
- GIVEN a producer handoff omits evidence kind, uses an untyped parent hash, contains malformed BLAKE3, lacks policy identity, duplicates a canonical root, treats JSON as authority, or weakens non-claims
- WHEN handoff validation runs
- THEN Octet MUST emit deterministic diagnostics and MUST NOT promote the handoff as stack-chain ready.

### Requirement: Compatibility projections are non-authoritative
r[octet.evidence_provenance.evidence_chain_handoff.projection] Octet MAY emit JSON projections for evidence-chain profiles, but MUST document that Kamacite canonical receipts own portable stack identity.

#### Scenario: Valid handoff feeds Kamacite
r[octet.evidence_provenance.evidence_chain_handoff.positive]
- GIVEN a valid Octet evidence-chain producer handoff
- WHEN Kamacite canonicalizes the handoff
- THEN the resulting receipt MUST preserve Octet's typed parent links and non-claim boundary.

### Requirement: Stack smoke validates the first profile
r[octet.evidence_provenance.evidence_chain_handoff.validation] Octet SHOULD validate the generic handoff through the function-address stack smoke before additional profiles depend on it.

#### Scenario: Smoke proves profile-neutral envelope
- GIVEN function-address is the first evidence-chain profile
- WHEN the stack smoke runs through Octet, Kamacite, Valence, Mantle, and Cairn
- THEN the smoke MUST prove the generic envelope can carry one profile without profile-specific wrapper code.
