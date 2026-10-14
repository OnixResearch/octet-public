# Trust Boundary Index Specification

## Purpose

Defines the `trust-boundary-index` capability.

## Requirements

### Requirement: Trust-boundary index themes

r[octet.trust_boundary_index.themes] Octet documentation SHOULD provide an index that links the checked-path framing, evidence stack, evidence tiers, provenance chain, scope-boundary, and controlled-deployment themes to concrete Octet surfaces.

#### Scenario: Index covers all themes

r[octet.trust_boundary_index.themes.complete]

- GIVEN the trust-boundary or evidence-path index exists
- WHEN a reader scans the index
- THEN it SHOULD include checked-path framing, evidence stack, evidence tiers, provenance chain, scope-boundary, and controlled-deployment entries.
- AND each entry SHOULD link to a concrete Octet doc section, command, artifact guide, or planned change output.

### Requirement: Index avoids overclaims

r[octet.trust_boundary_index.no_overclaims] The trust-boundary index MUST NOT introduce unsupported verified-runtime, proof-stack, or whole-program correctness claims.

#### Scenario: Index claim scan is clean

r[octet.trust_boundary_index.no_overclaims.clean]

- GIVEN documentation validation scans the trust-boundary index
- WHEN it encounters verification, proof, or correctness language
- THEN the language MUST be scoped to the linked evidence surface.
- AND unqualified verified-runtime, proof-stack, or correctness-guarantee claims MUST fail validation.
