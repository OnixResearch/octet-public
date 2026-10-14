# Lint Guardrail Fixtures Specification

## Purpose

Defines the `lint-guardrail-fixtures` capability.

## Requirements

### Requirement: Guardrail fixture manifest

r[octet.lint_guardrail_fixtures.manifest] Octet MUST maintain a checked manifest of non-emitting guardrail fixtures for registered lints.

#### Scenario: Guardrail manifest names valid fixtures

r[octet.lint_guardrail_fixtures.manifest.valid]

- GIVEN `tests/fixtures/lint_guardrail_fixtures.json` is checked in
- WHEN fixture freshness validation runs
- THEN every manifest entry MUST reference a known registered lint and a fixture listed in `ui_fixture_set.json`.
- AND every entry MUST carry a scenario category such as `safe_shape`, `configured_off`, `within_limit`, `test_context`, `macro_guardrail`, or `known_exception`.

#### Scenario: Unknown guardrail lint fails

r[octet.lint_guardrail_fixtures.manifest.unknown]

- GIVEN the guardrail manifest references a lint not in the live inventory
- WHEN fixture freshness validation runs
- THEN validation MUST fail with a diagnostic naming the unknown lint.

### Requirement: Positive and negative coverage completeness

r[octet.lint_guardrail_fixtures.complete] Every registered lint MUST have at least one emitting fixture and at least one non-emitting guardrail fixture before the lint catalog is considered fresh.

#### Scenario: Missing guardrail is rejected

r[octet.lint_guardrail_fixtures.complete.missing]

- GIVEN a registered lint has an emitting fixture but no guardrail fixture
- WHEN fixture freshness validation runs
- THEN validation MUST fail with a diagnostic naming the lint and the missing guardrail class.

#### Scenario: Guardrail fixture remains clean

r[octet.lint_guardrail_fixtures.complete.clean]

- GIVEN a fixture is listed as a guardrail for a lint
- WHEN the UI rail runs
- THEN the fixture MUST NOT emit that lint's canonical diagnostic.
- AND any expected diagnostics from other lints MUST be documented by the fixture category or separate emitting manifest.
