# Verus Proof Rails Specification

## Purpose

Adds an iddqd-backed internal index for the proof claim catalog while preserving existing proof-card, proof-reference, and catalog JSON artifact shapes.

## Requirements

### Requirement: Proof claim identity index
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.index] Octet MUST validate proof claim catalog entries through identity indexes whose keys are derived from claim and alias values rather than duplicated temporary map keys.

#### Scenario: Canonical and alias lookup passes
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.positive]
- GIVEN a proof claim catalog has unique canonical claim ids, unique alias ids, valid alias targets, required artifacts, documentation anchors, and required non-claims
- WHEN proof claim reference validation resolves canonical and alias ids
- THEN validation MUST preserve the target claim identity and accept matching references.

#### Scenario: Duplicate or broken catalog identity fails
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.negative]
- GIVEN a proof claim catalog has duplicate claim ids, duplicate alias ids, alias/claim id conflicts, self-targeting aliases, missing targets, invalid evidence tiers, invalid formal-proof promotion, missing required artifacts, missing anchors, or weakened non-claims
- WHEN proof claim catalog validation builds identity indexes
- THEN validation MUST fail closed with deterministic diagnostics and MUST NOT resolve by scan order or overwrite behavior.

### Requirement: Catalog wire compatibility
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.validation] Octet MUST keep the proof claim catalog JSON vector shape compatible while using iddqd indexes only inside pure validation and resolution code.

#### Scenario: Artifact shapes remain stable
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.docs]
- GIVEN proof-card, proof-reference, and provenance export artifacts are produced or validated
- WHEN the proof claim catalog index is used internally
- THEN public artifact schemas MUST remain stable and aliases MUST NOT be treated as one-to-one proof claims.

### Requirement: Final validation evidence
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.final_validation] The change MUST include positive and negative proof claim catalog coverage plus focused Octet validation evidence before archive.

#### Scenario: Proof claim fixtures cover duplicate handling
r[octet.verus_proof_rails.iddqd_proof_claim_catalog.final_validation.fixtures]
- GIVEN valid and invalid proof claim catalog fixtures
- WHEN focused proof claim tests and Cairn gates run
- THEN unique claim and alias identities MUST pass and duplicate, missing, or overclaiming identities MUST fail closed.
