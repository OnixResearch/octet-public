# Lint Autofix Quality Specification

## Purpose

Defines the `lint-autofix-quality` capability.

## Requirements

### Requirement: Machine-applicable autofix proof

r[octet.lint_autofix_quality.machine_applicable] Every lint that emits a `MachineApplicable` suggestion MUST have checked autofix proof unless the lint is explicitly listed with an out-of-scope rationale.

#### Scenario: Machine fix has fixed artifact

r[octet.lint_autofix_quality.machine_applicable.fixed]

- GIVEN a lint emits a `MachineApplicable` suggestion in a UI fixture
- WHEN the autofix rail runs
- THEN the rail MUST apply the suggestion, compare the resulting source to a checked-in `.fixed` artifact, rerun the fix to prove idempotence, and rerun checking to prove no warning remains.

#### Scenario: Missing machine fix proof fails freshness

r[octet.lint_autofix_quality.machine_applicable.missing]

- GIVEN a lint emits a `MachineApplicable` suggestion but has no `.fixed` artifact and no out-of-scope rationale
- WHEN autofix freshness validation runs
- THEN validation MUST fail with a diagnostic naming the lint.

### Requirement: Non-machine suggestion tier evidence

r[octet.lint_autofix_quality.non_machine_tiers] `MaybeIncorrect` and `HasPlaceholders` suggestions MUST remain visibly classified in diagnostics and MUST NOT be treated as automatic rewrites by the autofix proof rail.

#### Scenario: MaybeIncorrect tier is visible

r[octet.lint_autofix_quality.non_machine_tiers.maybe]

- GIVEN a lint emits a `MaybeIncorrect` suggestion
- WHEN the UI stderr snapshot is checked
- THEN the snapshot MUST include `structured suggestion applicability: MaybeIncorrect`.

#### Scenario: Placeholder tier is not machine-applied

r[octet.lint_autofix_quality.non_machine_tiers.placeholder]

- GIVEN a lint emits a `HasPlaceholders` suggestion
- WHEN the machine-applicable autofix rail runs
- THEN the rail MUST NOT require a `.fixed` artifact for that placeholder suggestion unless the change explicitly opts in.
