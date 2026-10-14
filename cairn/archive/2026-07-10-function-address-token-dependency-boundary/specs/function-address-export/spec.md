# Function Address Export Specification

## Purpose

Clarify that Octet function-address dependency identity records source-token dependency sets, not resolved Rust semantic closure.

## Requirements

### Requirement: Dependency identity kind is explicit
r[octet.function_address_export.dependency_boundary.kind] Octet MUST label function-address dependency identity with a kind that distinguishes source-token dependency sets from resolved semantic dependency closure.

#### Scenario: Token-set identity is visible
r[octet.function_address_export.dependency_boundary.fixtures]
- GIVEN a function-address report records dependency identity for a Rust function
- WHEN an operator inspects the record
- THEN the record MUST identify the dependency evidence as source-token/source-shape identity only.

### Requirement: Existing dependency hashes remain compatible
r[octet.function_address_export.dependency_boundary.compat] Octet SHOULD continue accepting existing `dependency_set_hash` baselines while adding explicit dependency identity kind metadata.

#### Scenario: Existing baseline loads
r[octet.function_address_export.dependency_boundary.validation]
- GIVEN a baseline has the existing dependency hash fields and no forbidden overclaim text
- WHEN Octet loads the baseline
- THEN Octet SHOULD accept it and render the explicit token-set dependency identity in the next report.

### Requirement: Dependency closure overclaims fail closed
r[octet.function_address_export.dependency_boundary.overclaim] Octet MUST reject function-address baseline or projection text that claims source-token dependency identity proves resolved call graph closure, Rust semantic dependencies, whole-program safety, or release eligibility.

#### Scenario: Overclaim is rejected
r[octet.function_address_export.dependency_boundary.docs]
- GIVEN a baseline or projection describes the dependency hash as complete semantic closure
- WHEN function-address validation runs
- THEN Octet MUST fail with a deterministic overclaim diagnostic.
