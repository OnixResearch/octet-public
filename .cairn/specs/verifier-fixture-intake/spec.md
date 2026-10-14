# Verifier Fixture Intake Specification

## Purpose

Define an Octet-owned intake rail for external verifier-inspired positive and negative fixtures.

## Requirements

### Requirement: Verifier fixture intake profile
r[octet.verifier_fixture_intake.profile] Octet MUST define a verifier-fixture intake profile that records fixture identity, source artifact ref, syntax family, expected verdict, expected diagnostics, assumptions, unsupported-feature labels, verifier-family label, fixture role, and explicit non-claims.

#### Scenario: Positive verifier-style fixture validates
r[octet.verifier_fixture_intake.positive]
- GIVEN an Octet-owned fixture models a bounded verifier-style example with a declared syntax family, expected verdict, assumptions, diagnostics, and non-claims
- WHEN verifier-fixture intake validation runs
- THEN Octet MUST accept the fixture as a normalized test/evidence input.

### Requirement: Fixture validation fails closed on malformed or overclaiming inputs
r[octet.verifier_fixture_intake.validation] Octet MUST reject verifier fixtures that are malformed, stale, unsupported, contradictory without expected failure, or overclaiming.

#### Scenario: Negative fixture validates expected failure
r[octet.verifier_fixture_intake.negative]
- GIVEN a fixture has a contradictory postcondition, unsupported feature, malformed contract, stale expected verdict, missing diagnostic, role promotion, or verifier-soundness overclaim
- WHEN verifier-fixture intake validation runs
- THEN validation MUST fail or classify the fixture as an expected negative case with deterministic diagnostics and MUST NOT promote it to proof evidence.

### Requirement: Verifier fixtures are Octet-owned minimal examples
r[octet.verifier_fixture_intake.integration] Octet MUST store minimal owned fixtures and MUST NOT require external verifier runtimes in starter checks.

#### Scenario: Fixture corpus runs without external verifier
- GIVEN the fixture corpus includes examples inspired by Kani-compatible harnesses, Creusot-style contracts, and Verus-style proof macros
- WHEN Octet runs fixture normalization or focused regression tests
- THEN the tests MUST operate on checked-in minimal fixtures without invoking Zani, Sunder, Certus, Kani, Verus, or Z4 unless a separate opt-in rail is selected.

### Requirement: Fixture intake preserves non-claims
r[octet.verifier_fixture_intake.final_validation] Octet MUST state that accepted verifier fixtures are regression and UX inputs only.

#### Scenario: Accepted fixture is not source-verifier proof
- GIVEN a verifier fixture passes Octet intake validation
- WHEN proof cards, standards checks, or UI tests consume it
- THEN the result MUST NOT claim source-verifier execution, verifier parity, proof soundness, solver soundness, source-code correctness, whole-program safety, or release eligibility.
