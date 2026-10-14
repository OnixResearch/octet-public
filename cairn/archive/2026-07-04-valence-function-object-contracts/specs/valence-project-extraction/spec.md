# Valence Project Extraction Specification Delta

## ADDED Requirements

### Requirement: Function Object normalization is core-owned

r[octet.valence_project_extraction.phase4.function_object_normalization] The system MUST ensure `valence-core` owns or explicitly validates the Function Object v1 normalization contract so callers cannot silently pair raw source with normalized source from a different algorithm.

#### Scenario: Core-owned normalization is stable

r[octet.valence_project_extraction.phase4.function_object_normalization.core_owned]

- GIVEN the Function Object core receives the same selected function source and scanner facts twice
- WHEN `valence-core` constructs Function Object v1 records using the declared normalization algorithm
- THEN the normalized body hash, semantic identity, object hash, and no-correctness boundary MUST be stable across both constructions.

#### Scenario: Normalization mismatch fails closed

r[octet.valence_project_extraction.phase4.function_object_normalization.rejects_mismatch]

- GIVEN a caller provides normalized source tagged with an unsupported algorithm or inconsistent with the Function Object v1 normalization contract
- WHEN the Function Object core attempts construction
- THEN it MUST return a typed construction error and MUST NOT emit an object hash.

### Requirement: Function Object hash material is explicit

r[octet.valence_project_extraction.phase4.function_object_hash_payload] The system MUST define reviewable typed hash material for Function Object v1 object hashing and MUST preserve the existing canonical hash payload unless a future reviewed migration updates schema and fixtures.

#### Scenario: Hash payload is reviewable and compatible

r[octet.valence_project_extraction.phase4.function_object_hash_payload.reviewable]

- GIVEN a Function Object v1 record is constructed
- WHEN its object hash is computed
- THEN tests MUST assert which typed payload fields are included and which display/source-local fields are excluded.
- AND existing object corpus compatibility MUST remain unchanged unless the change includes explicit migration fixtures and release notes.

### Requirement: Function Object core API uses typed contracts

r[octet.valence_project_extraction.phase4.function_object_typed_api] The system SHOULD expose typed Rust contracts for Function Object construction errors, dependency identity statuses, effect caveat classifications, macro caveat classifications, and pure-cache blockers while preserving existing serialized Function Object v1 strings.

#### Scenario: Typed contracts serialize compatibly

r[octet.valence_project_extraction.phase4.function_object_typed_api.serializes_compatibly]

- GIVEN typed Function Object status and caveat values are used inside `valence-core`
- WHEN a Function Object v1 DTO is serialized
- THEN the serialized schema, status strings, caveat strings, pure-cache blockers, and no-correctness fields SHOULD match the existing v1 compatibility contract.
- AND malformed construction inputs SHOULD fail with typed errors rather than stringly ad hoc messages.

### Requirement: Cargo Octet remains the Function Object shell during hardening

r[octet.valence_project_extraction.phase4.function_object_shell] The system MUST keep filesystem access, source discovery, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and process exits in `cargo-octet` while delegating Function Object normalization validation, caveat construction, pure-cache eligibility, and hash construction to `valence-core`.

#### Scenario: Shell delegates hardened core construction

r[octet.valence_project_extraction.phase4.function_object_shell.delegates]

- GIVEN an operator runs existing `cargo-octet object ...` commands
- WHEN Function Object v1 records are needed
- THEN `cargo-octet` MUST supply scanner facts to the hardened `valence-core` API and preserve existing command behavior, receipt schemas, and bounded non-claims.

### Requirement: Function Object hardening preserves evidence gates

r[octet.valence_project_extraction.phase4.function_object_gates] The system MUST preserve focused `valence-core` Function Object tests, focused `cargo-octet` object tests, object corpus gates, aggregate Valence object evidence, and Cairn validation after hardening the Function Object seam.

#### Scenario: Compatibility gates stay green

r[octet.valence_project_extraction.phase4.function_object_gates.compatibility]

- GIVEN the Function Object core API has been hardened
- WHEN focused Function Object tests, object corpus compatibility, object evidence gates, and Cairn validation run
- THEN they MUST pass without broadening existing bounded evidence contracts or claiming behavioral correctness, semantic equivalence, pure-cache eligibility, cross-machine determinism, cross-toolchain determinism, or general HIR/THIR/MIR extraction.
