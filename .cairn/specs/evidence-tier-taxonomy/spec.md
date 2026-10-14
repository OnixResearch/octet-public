# Evidence Tier Taxonomy Specification

## Purpose

Defines the `evidence-tier-taxonomy` capability.

## Requirements

### Requirement: Evidence-tier enum

r[octet.evidence_tiers.enum] Octet emitted evidence records SHOULD carry a machine-readable evidence tier selected from `static_lint`, `policy_check`, `ui_fixture`, `autofix_verified`, `transcript_assertion`, `formal_proof`, or `manual_review_required`.

#### Scenario: Known tier validates

r[octet.evidence_tiers.enum.known]

- GIVEN an evidence record uses one of the defined tier discriminants
- WHEN the record is serialized and validated
- THEN validation MUST accept the tier.
- AND the record MUST preserve the tier string deterministically in emitted JSON or sidecar output.

#### Scenario: Unknown tier fails closed

r[octet.evidence_tiers.enum.unknown]

- GIVEN an evidence record uses an unrecognized tier discriminant
- WHEN the record is validated
- THEN validation MUST fail closed with a diagnostic naming the unknown tier.

### Requirement: Tier-specific validation

r[octet.evidence_tiers.tier_validation] Octet MUST enforce tier-specific validation rules that prevent validation or review artifacts from being relabeled as formal proofs.

#### Scenario: Formal proof needs proof identity

r[octet.evidence_tiers.tier_validation.formal_proof_identity]

- GIVEN an evidence record uses the `formal_proof` tier
- WHEN the record is validated
- THEN the record MUST include verifier identity and proof artifact identity fields.
- AND validation MUST reject a `formal_proof` record that lacks those fields.

#### Scenario: Manual review remains explicit

r[octet.evidence_tiers.tier_validation.manual_review]

- GIVEN an evidence record uses the `manual_review_required` tier
- WHEN the record is rendered
- THEN it MUST include a human-action message.
- AND it MUST NOT be rendered as a successful automated proof.

### Requirement: Evidence-tier registry ownership

r[octet.evidence_tiers.registry] The evidence-tier receipt or sidecar family MUST be registered in `standards/policies/registry.ncl` with the correct `rust-derived` or `nickel-authored` ownership classification.

#### Scenario: Registry covers emitted artifact family

r[octet.evidence_tiers.registry.covered]

- GIVEN Octet emits an evidence-tier sidecar or receipt family
- WHEN policy freshness validation runs
- THEN the family MUST appear in the registry.
- AND generated contract artifacts MUST be fresh for Rust-derived receipts.
