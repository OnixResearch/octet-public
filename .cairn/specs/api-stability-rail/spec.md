# Api Stability Rail Specification

## Purpose

Defines the `api-stability-rail` capability.

## Requirements

### Requirement: SemVer API-stability rail

r[octet.api_stability.semver_rail] Octet SHOULD provide an API-stability rail backed by `cargo-semver-checks` for release candidates that expose Rust library APIs.

#### Scenario: Compatible API passes

r[octet.api_stability.semver_rail.compatible]

- GIVEN a checked package has a valid baseline and no SemVer-breaking findings for the selected feature and target scope
- WHEN the API-stability rail runs
- THEN Octet SHOULD report success.
- AND the report SHOULD identify the checked package, baseline source, feature set, target triple, and `cargo-semver-checks` version.

#### Scenario: Breaking API fails closed

r[octet.api_stability.semver_rail.breaking]

- GIVEN `cargo-semver-checks` reports a denied SemVer finding for the selected release intent
- WHEN Octet evaluates the API-stability rail
- THEN Octet MUST fail the rail with diagnostics preserving the SemVer finding name, affected item, and required update level when provided.

### Requirement: API baseline declaration

r[octet.api_stability.baseline] API-stability execution MUST require an explicit or workspace-declared baseline before it can be treated as release evidence.

#### Scenario: Baseline source is explicit

r[octet.api_stability.baseline.explicit]

- GIVEN an operator configures an API-stability run
- WHEN Octet resolves the baseline
- THEN the baseline MUST identify one of a registry version, VCS revision, local source root, or saved rustdoc JSON artifact.
- AND the receipt MUST record the resolved baseline identity.

#### Scenario: Missing baseline cannot pass

r[octet.api_stability.baseline.missing]

- GIVEN no baseline is configured or passed on the command line
- WHEN the API-stability rail is requested for release evidence
- THEN Octet MUST fail closed with a diagnostic naming the missing baseline field.

### Requirement: API-stability receipts

r[octet.api_stability.receipt] API-stability runs SHOULD emit deterministic `policy_check` evidence that names the scope and known non-claims.

#### Scenario: Receipt records checked scope

r[octet.api_stability.receipt.scope]

- GIVEN the API-stability rail completes
- WHEN Octet writes its receipt
- THEN the receipt SHOULD include schema version, checker ID, command argv, package list, baseline identity, current identity, feature flags, target triple, findings, result status, and bounded log identity.

#### Scenario: Receipt preserves API-review boundary

r[octet.api_stability.receipt.boundary]

- GIVEN an API-stability receipt reports success
- WHEN the receipt is rendered or verified
- THEN it MUST NOT claim behavioral compatibility, business-domain compatibility, or exhaustive SemVer coverage beyond the selected checker and scope.

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
