## ADDED Requirements

### Requirement: External schema Nickel contract library

r[octet.evidence_contracts.external_schema_library] Octet SHOULD provide a checked-in generated Nickel contract library for selected external evidence schemas that are accepted as Octet input.

#### Scenario: Contracts are generated from pinned schemas

r[octet.evidence_contracts.external_schema_library.pinned]

- GIVEN an external evidence rail accepts a third-party JSON report shape
- WHEN Octet refreshes external schema contracts
- THEN the generated Nickel contract SHOULD be derived from a checked-in pinned schema input.
- AND the generated metadata SHOULD record the schema source identity, schema dialect, generator identity, BLAKE3 source digest, generated artifact path, and generated artifact digest.

#### Scenario: Library index names supported contracts

r[octet.evidence_contracts.external_schema_library.index]

- GIVEN generated external schema contracts are committed
- WHEN a maintainer inspects the contract bundle entry point
- THEN the entry point SHOULD list each supported external report contract by stable contract ID, tool family, schema version or dialect, and generated artifact path.

### Requirement: External schema contract freshness

r[octet.evidence_contracts.external_schema_freshness] Generated external schema contracts MUST be covered by deterministic registry and manifest freshness validation.

#### Scenario: Stale generated contract fails validation

r[octet.evidence_contracts.external_schema_freshness.stale]

- GIVEN a pinned external schema input changes without regenerating the matching Nickel contract or manifest
- WHEN policy or contract freshness validation runs
- THEN validation MUST fail with a diagnostic naming the stale generated contract artifact.

#### Scenario: Network schema lookup is not required

r[octet.evidence_contracts.external_schema_freshness.offline]

- GIVEN external schema contract validation runs in an offline release or CI environment
- WHEN validation checks generated contract freshness
- THEN validation MUST use checked-in schema inputs and manifests only.
- AND validation MUST NOT fetch SchemaStore, GitHub, tool documentation, or other network resources.

### Requirement: External schema validation boundaries

r[octet.evidence_contracts.external_schema_boundaries] External schema contracts MUST validate accepted input shapes without becoming the normative compatibility source for Octet-emitted receipts or evidence claims.

#### Scenario: External report shape validation preserves non-claims

r[octet.evidence_contracts.external_schema_boundaries.non_claim]

- GIVEN an external report passes its generated Nickel contract
- WHEN Octet imports or summarizes that report
- THEN Octet MAY treat the report shape as accepted input.
- AND Octet MUST NOT claim the generated contract proves the external tool result is complete, correct, vulnerability-free, or formally verified.

#### Scenario: Unsupported schema capability is visible

r[octet.evidence_contracts.external_schema_boundaries.unsupported]

- GIVEN a pinned external JSON schema uses a dialect or keyword the generator cannot faithfully represent
- WHEN contract generation or validation runs
- THEN Octet MUST either fail closed with a diagnostic naming the unsupported capability or record an explicit predicate-fallback/caveat field that reviewers can inspect.
