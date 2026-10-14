## ADDED Requirements

### Requirement: Provenance record

r[octet.evidence_provenance.record] Octet evidence summaries that claim requirement or policy coverage SHOULD include a provenance record with source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.

#### Scenario: Provenance links source to artifact

r[octet.evidence_provenance.record.links]

- GIVEN an evidence artifact claims support for a requirement, policy, lint, or transcript assertion
- WHEN the artifact summary is emitted
- THEN the summary SHOULD include a source ID, checker ID, evidence tier, artifact locator, BLAKE3 content hash, and checked scope.
- AND a reviewer MUST be able to identify the originating requirement or policy from the record.

### Requirement: Provenance validation fails closed

r[octet.evidence_provenance.fail_closed] Provenance validation MUST fail closed when required source identity, checker identity, or content identity is missing or inconsistent.

#### Scenario: Missing source ID fails

r[octet.evidence_provenance.fail_closed.missing_source]

- GIVEN a provenance record omits the source ID
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the missing source ID field.

#### Scenario: Unknown checker ID fails

r[octet.evidence_provenance.fail_closed.unknown_checker]

- GIVEN a provenance record names a checker ID that is not known to the registry
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the unknown checker ID.

#### Scenario: Hash mismatch fails

r[octet.evidence_provenance.fail_closed.hash_mismatch]

- GIVEN a provenance record contains a BLAKE3 hash that does not match the referenced deterministic artifact
- WHEN provenance validation runs
- THEN validation MUST fail with a diagnostic naming the artifact locator.

### Requirement: Provenance registry ownership

r[octet.evidence_provenance.registry] The provenance record family MUST be registered in `standards/policies/registry.ncl` under the correct ownership classification.

#### Scenario: Registry covers provenance records

r[octet.evidence_provenance.registry.covered]

- GIVEN Octet emits provenance records after execution
- WHEN artifact manifest freshness validation runs
- THEN the provenance family MUST be covered as Rust-derived unless an explicit non-candidate rationale applies.
