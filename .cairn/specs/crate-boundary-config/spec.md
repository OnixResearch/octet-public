# Crate Boundary Config Specification

## Purpose

Defines the `crate-boundary-config` capability.

## Requirements

### Requirement: Policy-driven crate boundary mechanism

r[octet.crate_boundary_config.mechanism] The architecture checker MUST read crate-boundary obligations only from an activated project architecture policy.

#### Scenario: Boundary scopes are declared in active policy

r[octet.crate_boundary_config.mechanism.declared]

- GIVEN a project selects an architecture policy with core, shell, adapter, and composition-root scopes
- WHEN the architecture checker resolves crate-boundary obligations
- THEN every check MUST use the declared project scopes and rules
- AND no check MUST substitute hardcoded Octet crate paths

#### Scenario: Malformed active boundary config fails closed

r[octet.crate_boundary_config.mechanism.fail_closed]

- GIVEN an active project policy requires crate-boundary evaluation but omits required scopes or role declarations
- WHEN policy validation runs
- THEN validation MUST fail with a deterministic diagnostic before source evaluation

### Requirement: Octet-layout default profile equivalence

r[octet.crate_boundary_config.default_profile] Octet MUST preserve its intended first-party boundary diagnostics through an explicit Octet project policy rather than consumer fallback literals.

#### Scenario: Explicit Octet self-policy matches baseline

r[octet.crate_boundary_config.default_profile.baseline]

- GIVEN the Octet repository selects its checked-in first-party architecture policy
- WHEN the architecture checker runs against the same baseline sources
- THEN the selected obligations and expected diagnostics MUST match the approved pre-migration baseline

### Requirement: Non-silencing fallback

r[octet.crate_boundary_config.fallback] A missing project architecture-policy pointer MUST disable project architecture evaluation instead of selecting Octet-layout fallback obligations.

#### Scenario: Missing project policy is inactive

r[octet.crate_boundary_config.fallback.defaults]

- GIVEN a consumer declares no project architecture-policy pointer
- WHEN `cargo octet check` resolves architecture activation
- THEN project crate-boundary checks MUST remain inactive
- AND output MUST NOT report an FCIS or Hexagonal architecture pass

#### Scenario: Active policy cannot omit required mechanism

- GIVEN a project activates `gate` or `admission` architecture mode but omits required crate-boundary data
- WHEN policy validation runs
- THEN validation MUST fail closed
- AND the checker MUST NOT use fallback paths or an empty obligation set

### Requirement: Generalized architecture checks

r[octet.crate_boundary_config.checks] Activated crate-boundary checks MUST resolve target paths, roles, optional `no_std`, forbidden dependencies, and external-domain rules from project policy.

#### Scenario: Custom-layout consumer is checkable

r[octet.crate_boundary_config.checks.custom_layout]

- GIVEN an active project policy declares a custom domain, application, adapter, and composition-root layout
- WHEN architecture evaluation runs
- THEN checks MUST use the custom declarations without Octet path assumptions
- AND valid custom layouts MUST not receive fallback-layout findings

#### Scenario: Boundary config is receipt-covered

r[octet.crate_boundary_config.artifact]

- GIVEN an activated project architecture policy is exported
- WHEN config-receipt freshness validation runs
- THEN the exported artifact MUST carry a BLAKE3 digest
- AND source or contract drift MUST fail freshness validation
