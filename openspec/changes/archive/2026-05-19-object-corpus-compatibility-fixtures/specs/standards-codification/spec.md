## ADDED Requirements

### Requirement: Object corpus compatibility fixtures

Octet MUST maintain checked-in compatibility fixtures for object corpus receipt schema v1 that document accepted replay behavior and fail-closed rejection behavior.

#### Scenario: Positive v1 fixture replays cleanly

- GIVEN a checked-in v1 object corpus receipt compatibility fixture and its checked-in Rust source corpus
- WHEN `cargo octet object corpus check` replays that fixture against the source corpus
- THEN the check report MUST return `status=match` with matching compared fields
- AND the fixture MUST keep `claims_correctness=false` and `claims_semantic_equivalence=false`

#### Scenario: Negative fixtures reject unsupported compatibility shapes

- GIVEN checked-in negative object corpus receipt compatibility fixtures for a missing required field and a future schema version
- WHEN `cargo octet object corpus check` evaluates those fixtures against the source corpus
- THEN each check report MUST return `status=invalid`
- AND each report MUST include the expected typed error kind
- AND comparisons MUST remain empty for invalid fixtures
