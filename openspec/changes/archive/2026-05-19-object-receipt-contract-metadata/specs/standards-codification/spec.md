## ADDED Requirements

### Requirement: Function Object Corpus Contract Metadata [r[function-object-corpus-contract-metadata]]

Octet MUST expose Rust-derived contract metadata for `octet.function-object-corpus-receipt.v1` and its corpus check report so review and CI can verify schema names, schema versions, required fields, compared fields, and boundary non-claims without treating JSON files as the source of truth.

#### Scenario: Receipt contract metadata

- GIVEN the object corpus receipt DTO and schema/version constants in Rust
- WHEN an operator runs the object corpus contract metadata command
- THEN the JSON output names `octet.function-object-corpus-receipt.v1`, schema version `1`, required receipt fields, compared replay fields, and the no-correctness/no-semantic-equivalence boundary fields

#### Scenario: Check-report contract metadata

- GIVEN the object corpus check report DTO in Rust
- WHEN an operator runs the object corpus contract metadata command
- THEN the JSON output names the check report schema/version, required report fields, valid status values, and required error/comparison fields

#### Scenario: Checked-in fixture compatibility

- GIVEN the checked-in internal object corpus receipt fixture
- WHEN the fixture is validated by contract metadata tests
- THEN required fields are present, schema/version match the Rust constants, and `claims_correctness` plus `claims_semantic_equivalence` remain false

#### Scenario: Missing required field fails closed

- GIVEN a corpus receipt with a field required by contract metadata removed
- WHEN the corpus check path validates it
- THEN the result is a deterministic invalid-input report naming the missing field
