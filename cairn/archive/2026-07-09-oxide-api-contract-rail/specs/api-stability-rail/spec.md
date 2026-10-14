# API Stability Rail Specification

## Purpose

Adds an Oxide-inspired API/schema contract rail for Octet machine surfaces, generated contracts, compatibility diffs, and generated-client fixtures.

## Requirements

### Requirement: Explicit contract boundary
r[octet.api_stability_rail.oxide_contracts.boundary] Octet MUST record the source of truth, trust boundary, and supported claim for each generated API or schema contract.

#### Scenario: Contract source is declared
r[octet.api_stability_rail.oxide_contracts.boundary.source]
- GIVEN Octet emits a generated schema, OpenAPI-like document, or receipt contract
- WHEN the contract rail records it
- THEN the record MUST name whether the source is a Rust DTO, Nickel-authored policy, external schema input, or compatibility fixture.

#### Scenario: Hidden authority is rejected
r[octet.api_stability_rail.oxide_contracts.boundary.reject_hidden_authority]
- GIVEN a generated client or schema document is available
- WHEN Octet evaluates evidence claims
- THEN Octet MUST NOT treat generator success as proof of service correctness, evidence truth, or release eligibility.

### Requirement: Schema generation flow
r[octet.api_stability_rail.oxide_contracts.schema_generation] Octet SHOULD generate machine contracts from typed Rust DTOs or checked Nickel policy inputs instead of hand-maintained JSON documents.

#### Scenario: Rust receipt contract is derived outward
r[octet.api_stability_rail.oxide_contracts.schema_generation.rust_outward]
- GIVEN a runtime receipt DTO is owned by Rust
- WHEN a schema contract is needed for downstream validation
- THEN the contract SHOULD be generated from the Rust type or a Rust-derived contract rather than hand-authored separately.

#### Scenario: Human config remains Nickel-owned
r[octet.api_stability_rail.oxide_contracts.schema_generation.nickel_owned]
- GIVEN humans compose or override a policy input before execution
- WHEN Octet records its contract
- THEN the human-authored source MUST remain Nickel or a Nickel-exported deterministic runtime input.

### Requirement: Compatibility diff classification
r[octet.api_stability_rail.oxide_contracts.compatibility_diff] Octet MUST classify contract changes through deterministic compatibility policies before promoting generated artifacts.

#### Scenario: Additive change passes by policy
r[octet.api_stability_rail.oxide_contracts.compatibility_diff.additive]
- GIVEN a new optional field is added under a profile that allows additive changes
- WHEN the old and new contracts are compared
- THEN the compatibility report SHOULD classify the change as additive and eligible for promotion.

#### Scenario: Breaking change fails closed
r[octet.api_stability_rail.oxide_contracts.compatibility_diff.breaking]
- GIVEN a required field is removed or an enum meaning changes incompatibly
- WHEN the old and new contracts are compared
- THEN the compatibility report MUST fail closed unless an explicit change record authorizes the break.

### Requirement: Generated-client fixtures
r[octet.api_stability_rail.oxide_contracts.generated_clients] Octet MAY use generated clients as downstream fixture consumers for contract smoke tests.

#### Scenario: Generated client proves consumption only
r[octet.api_stability_rail.oxide_contracts.generated_clients.nonclaim]
- GIVEN a generated client compiles and exercises fixture responses
- WHEN Octet records the result
- THEN the result MUST claim only contract consumption for the selected generator profile and MUST NOT claim API behavior correctness.

#### Scenario: Generator failure is diagnostic
r[octet.api_stability_rail.oxide_contracts.generated_clients.failure]
- GIVEN a contract cannot be consumed by the selected generator
- WHEN the fixture rail runs
- THEN Octet MUST report deterministic generator diagnostics and MUST NOT promote the contract as generated-client eligible.

### Requirement: Positive and negative contract corpus
r[octet.api_stability_rail.oxide_contracts.negative_corpus] Octet MUST keep positive and negative corpora for contract evolution and malformed inputs.

#### Scenario: Positive corpus covers accepted evolution
r[octet.api_stability_rail.oxide_contracts.negative_corpus.positive]
- GIVEN accepted additive changes, deprecations, and generated-client smoke fixtures are present
- WHEN the corpus runs
- THEN valid cases MUST produce stable compatibility reports.

#### Scenario: Negative corpus covers rejection paths
r[octet.api_stability_rail.oxide_contracts.negative_corpus.negative]
- GIVEN malformed schemas, removed required fields, unsupported generator inputs, or overclaim text are present
- WHEN the corpus runs
- THEN invalid cases MUST fail with deterministic diagnostics and no promoted artifact.

### Requirement: Contract rail validation
r[octet.api_stability_rail.oxide_contracts.validation] Octet MUST validate the API/schema rail with focused tests and Cairn gates before implementation is marked complete.

#### Scenario: Validation evidence is bounded
r[octet.api_stability_rail.oxide_contracts.validation.bounded]
- GIVEN the contract rail passes focused tests and fixtures
- WHEN Octet records validation evidence
- THEN the evidence MUST state the tested contract profiles and MUST NOT broaden the claim to whole-service correctness.
