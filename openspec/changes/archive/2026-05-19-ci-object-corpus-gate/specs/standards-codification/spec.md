## ADDED Requirements

### Requirement: Function Object Corpus CI Gate [r[function-object-corpus-ci-gate]]

Octet MUST expose a named CI/check target for the internal function object corpus evidence gate so fixture replay, fixture freshness, Rust-derived contract metadata compatibility, and fresh-target receipt determinism can be run without rediscovering individual test names.

#### Scenario: Object corpus CI check covers fixture replay

- GIVEN the checked-in internal object corpus receipt fixture
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute the fixture replay and byte-for-byte fixture freshness assertions for the selected internal corpus

#### Scenario: Object corpus CI check covers contract metadata

- GIVEN the Rust-derived object corpus receipt/check-report contract metadata
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute the fixture compatibility assertions for required fields, schema/version identity, and boundary fields

#### Scenario: Object corpus CI check covers fresh-target determinism

- GIVEN the fresh-target object corpus determinism rail
- WHEN the named object corpus CI/check target runs
- THEN it MUST execute receipt generation across fresh target directories and assert stable replay-significant fields

#### Scenario: Object corpus CI check preserves bounded non-claims

- GIVEN a passing named object corpus CI/check target
- WHEN the result is documented or used as evidence
- THEN it MUST be described as bounded local evidence replay for the selected corpus and MUST NOT claim correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage
