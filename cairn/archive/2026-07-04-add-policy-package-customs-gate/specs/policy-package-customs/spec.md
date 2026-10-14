## ADDED Requirements

### Requirement: Local policy-package customs gate

r[octet.policy_package_customs.gate] Octet SHOULD provide a local customs gate that validates contributed policy, profile, template, proof-rail, and generated-contract packages before they are promoted as repository-owned surfaces.

#### Scenario: Complete package passes with deterministic report

r[octet.policy_package_customs.gate.complete]

- GIVEN a contributed package declares package ID, kind, owner, source paths, generated artifacts, registry family IDs, generation commands, validation commands, fixtures, provenance, license notes, and non-claims
- WHEN the customs gate validates the package
- THEN validation SHOULD pass.
- AND the gate SHOULD emit a deterministic report listing checked surfaces, accepted artifacts, evidence tiers, and remaining manual-review actions.

#### Scenario: Missing registry coverage fails

r[octet.policy_package_customs.gate.missing_registry]

- GIVEN a package introduces a config, receipt, generated contract, or policy family
- WHEN the customs gate validates the package
- THEN validation MUST fail unless the family is registered with ownership classification, owner source path, generation command, validation command, and status.

### Requirement: Package fixture and export coverage

r[octet.policy_package_customs.fixtures] Promotable policy packages MUST include positive and negative fixtures plus deterministic export or generated-artifact freshness coverage when they define contracts, schemas, profiles, templates, or policy rules.

#### Scenario: Missing negative fixture blocks promotion

r[octet.policy_package_customs.fixtures.negative_required]

- GIVEN a package defines a contract, schema, profile, template, or policy rule that can reject malformed input
- WHEN the package declares only positive fixtures
- THEN the customs gate MUST fail with a diagnostic naming the missing negative fixture coverage.

#### Scenario: Stale generated artifact blocks promotion

r[octet.policy_package_customs.fixtures.stale_generated]

- GIVEN a package includes generated artifacts derived from Nickel, Rust DTOs, or pinned external schemas
- WHEN source inputs change without refreshing the generated artifacts or manifest
- THEN the customs gate MUST fail with a diagnostic naming the stale generated artifact or manifest entry.

### Requirement: Package boundaries and non-claims

r[octet.policy_package_customs.boundaries] The customs gate MUST preserve local-review boundaries and MUST NOT treat a package as trusted, safe, vulnerability-free, or production-ready merely because its metadata and fixtures pass.

#### Scenario: Secret-bearing package is rejected or excluded

r[octet.policy_package_customs.boundaries.secrets]

- GIVEN a package contains raw secrets, credentials, private keys, or other secret material
- WHEN the customs gate evaluates the package
- THEN validation MUST fail or require a non-candidate exclusion rationale before promotion.

#### Scenario: Remote fetch is not part of validation

r[octet.policy_package_customs.boundaries.offline]

- GIVEN customs validation runs in CI or release-readiness mode
- WHEN it validates a package
- THEN validation MUST use checked-in package sources, manifests, fixtures, and generated artifacts only.
- AND it MUST NOT fetch remote package indexes, schemas, or trust metadata.
