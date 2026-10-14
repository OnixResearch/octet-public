# Evidence Provenance Specification

## Purpose

Adds optional Preserves envelope export for Octet receipts and provenance artifacts.

## Requirements

### Requirement: Preserves receipt export contract
r[octet.evidence_provenance.preserves_receipt_export.contract] Octet MUST support an optional Preserves receipt envelope export contract that links canonical Preserves envelopes to verified Octet source receipts, rails, roles, consumer profiles, and non-claims.

#### Scenario: Verified Octet receipt envelope passes
r[octet.evidence_provenance.preserves_receipt_export.fixtures.positive]
- GIVEN an Octet artifact directory has passed artifact verification and an export row records source receipt digest, rail, schema label, canonical Preserves digest, evidence role, consumer profile, and required non-claims
- WHEN export validation runs
- THEN validation MUST pass and preserve source receipt linkage.

#### Scenario: Invalid Octet receipt envelope fails
r[octet.evidence_provenance.preserves_receipt_export.fixtures.negative]
- GIVEN an export row has missing source receipt, stale digest, skipped verification, role promotion, unsupported schema, or missing non-claims
- WHEN export validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Export validation boundary
r[octet.evidence_provenance.preserves_receipt_export.validation] Octet MUST treat Preserves export as an adapter over already verified Octet artifacts, not as a new proof rail or source-of-truth receipt.

#### Scenario: Adapter boundary is visible
r[octet.evidence_provenance.preserves_receipt_export.docs]
- GIVEN a Preserves export validates
- WHEN the supported claim is rendered
- THEN it MUST state that Octet source receipts remain authoritative and the Preserves envelope only carries identity/linkage for consumers.

### Requirement: Final validation
r[octet.evidence_provenance.preserves_receipt_export.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers export health
r[octet.evidence_provenance.preserves_receipt_export.final_validation.fixtures]
- GIVEN valid and invalid Preserves export fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
