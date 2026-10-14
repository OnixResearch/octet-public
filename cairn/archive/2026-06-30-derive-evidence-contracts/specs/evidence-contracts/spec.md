# Evidence Contracts Specification

## Purpose

Defines the `evidence-contracts` capability.

## Requirements

### Requirement: Rust-derived evidence contracts

r[octet.evidence_contracts.generated] Octet emitted evidence DTOs with compatibility-sensitive wire shapes SHOULD have deterministic Nickel contracts generated from Rust-owned schema sources rather than hand-authored parallel schemas.

#### Scenario: Finding and provenance contracts are generated

r[octet.evidence_contracts.generated.finding_provenance]

- GIVEN Octet owns the `Finding` v2 and `ProvenanceRecord` emitted DTOs in Rust
- WHEN evidence contracts are refreshed
- THEN generated Nickel contracts SHOULD describe their required fields, optional fields, schema versions, evidence-tier values, and BLAKE3 hash fields.
- AND the generated contracts SHOULD be stable across repeated generation from the same Rust-owned schema source.

### Requirement: Evidence contract registry freshness

r[octet.evidence_contracts.registry] Generated evidence contracts MUST be registered as `rust-derived` config or receipt families and covered by deterministic freshness validation.

#### Scenario: Registry covers generated contracts

r[octet.evidence_contracts.registry.covered]

- GIVEN generated evidence contracts are committed
- WHEN policy freshness validation runs
- THEN the registry MUST identify their Rust owner source, generated artifact path, generation command, validation command, and positive and negative fixtures.

#### Scenario: Stale generated contract fails

r[octet.evidence_contracts.registry.stale]

- GIVEN a Rust-owned evidence schema source changes without refreshing the generated Nickel contract
- WHEN policy freshness validation runs
- THEN validation MUST fail with a diagnostic naming the stale generated contract artifact.

### Requirement: Evidence contracts fail closed

r[octet.evidence_contracts.fail_closed] Evidence contracts MUST reject malformed emitted evidence records that omit required fields, use invalid evidence-tier discriminants, use unsupported schema versions, or provide wrong field types.

#### Scenario: Valid records pass

r[octet.evidence_contracts.fail_closed.valid_records]

- GIVEN a valid `Finding` v2 record and a valid `ProvenanceRecord`
- WHEN contract validation runs
- THEN both records MUST pass.

#### Scenario: Malformed records fail

r[octet.evidence_contracts.fail_closed.malformed_records]

- GIVEN an evidence record omits a required field, uses an invalid evidence tier, names an unsupported schema version, or supplies a value with the wrong type
- WHEN contract validation runs
- THEN validation MUST fail with a diagnostic naming the rejected field or contract.
