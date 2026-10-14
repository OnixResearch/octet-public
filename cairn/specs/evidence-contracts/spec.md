# Evidence Contracts Specification

## Purpose

Defines generated contracts for Octet evidence DTOs so emitted findings and provenance records have a single Rust-owned wire-shape source of truth plus deterministic Nickel validation artifacts.

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

### Requirement: ChaosControl proof-style input contract
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.contract] Octet MUST support a contract for ChaosControl proof-style input rails that records selected profile, focused crates, policy registry identity, input revisions, commands, and non-claims.

#### Scenario: Current input contract passes
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.positive]
- GIVEN a contract row names ChaosControl, an Octet profile, focused crates, registry digest, input revision, command, and required non-claims
- WHEN validation runs
- THEN the row MUST pass and preserve the consumer/profile identity.

#### Scenario: Stale input contract fails
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.fixtures.negative]
- GIVEN a contract row has stale input revision, missing profile, missing focused crate, or overbroad proof-style claim
- WHEN validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Contract validation
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.validation] Octet MUST validate proof-style input contracts without claiming ChaosControl runtime determinism or VMM correctness.

#### Scenario: Boundary is visible
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.docs]
- GIVEN a ChaosControl proof-style input contract passes
- WHEN the supported claim is rendered
- THEN it MUST state that Octet validates profile/input identity only and does not prove ChaosControl behavioral correctness.

### Requirement: Final validation
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers contract health
r[octet.evidence_contracts.chaoscontrol_proof_style_inputs.final_validation.fixtures]
- GIVEN valid and invalid input contract fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

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
