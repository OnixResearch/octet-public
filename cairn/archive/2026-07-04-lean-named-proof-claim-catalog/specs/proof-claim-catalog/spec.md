# Proof Claim Catalog Specification Delta

## ADDED Requirements

### Requirement: Proof claim IDs are stable and unique

r[octet.proof_claim_catalog.stable_ids] Octet SHOULD maintain a checked catalog for stable proof, evidence, and review-claim IDs that cross documentation, proof-card, proof-ref, suppression, release-evidence, or policy boundaries.

#### Scenario: Unique claim IDs export deterministically

r[octet.proof_claim_catalog.stable_ids.unique]

- GIVEN maintainers declare proof or evidence claims in the checked catalog
- WHEN the catalog export or freshness check runs
- THEN each claim ID MUST be unique, canonical, deterministically ordered, and tied to an owner surface.
- AND duplicate, malformed, or empty claim IDs MUST fail closed with diagnostics naming the rejected ID.

### Requirement: Cataloged claims link to evidence surfaces

r[octet.proof_claim_catalog.linkage] Cataloged claims SHOULD name their requirement or policy source, checked scope, expected evidence tier, accepted proof-card or receipt sources, lifecycle status, and explicit non-claims.

#### Scenario: Linked claim validates

r[octet.proof_claim_catalog.linkage.valid]

- GIVEN a cataloged claim names a requirement or policy ID, checked source scope, expected evidence tier, proof-card ID, receipt identity, provenance identity, or documentation anchor
- WHEN claim-reference validation runs
- THEN validation SHOULD confirm the referenced evidence matches the cataloged scope and tier.
- AND the report SHOULD identify the claim ID, owner surface, evidence match, lifecycle status, and non-claims.

#### Scenario: Stale or overpromoted claim fails

r[octet.proof_claim_catalog.linkage.stale]

- GIVEN a cataloged claim references a missing proof card, stale receipt, unknown requirement or policy ID, missing documentation anchor, mismatched source scope, or manual-review/imported evidence requested as formal proof
- WHEN claim-reference validation runs
- THEN validation MUST fail closed with diagnostics naming the stale or overpromoted field.

### Requirement: Claim renames use explicit aliases

r[octet.proof_claim_catalog.aliases] Claim renames SHOULD be represented as explicit aliases or deprecation records rather than silent string replacement.

#### Scenario: Explicit alias resolves

r[octet.proof_claim_catalog.aliases.explicit]

- GIVEN an old claim ID remains referenced by documentation or evidence
- AND the catalog declares an alias to a compatible replacement claim
- WHEN validation resolves the reference
- THEN the report SHOULD name both the alias and canonical replacement.
- AND incompatible replacements, cycles, or aliases to unknown claims MUST fail closed.

### Requirement: Claim catalog boundaries are explicit

r[octet.proof_claim_catalog.boundaries] The proof-claim catalog MUST NOT turn named claims into broader proof than their matched evidence tier, scope, receipt, and artifact identities support.

#### Scenario: Catalog rejects overclaims

r[octet.proof_claim_catalog.boundaries.no_overclaim]

- GIVEN catalog entries, generated reports, documentation, or proof-card references mention proof, verification, correctness, safety, certification, or semantic equivalence
- WHEN claim-boundary validation runs
- THEN the wording MUST scope each claim to its evidence tier, checked scope, receipt or artifact identity, and non-claims.
- AND the catalog MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate reviewed evidence.
