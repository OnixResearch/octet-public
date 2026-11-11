# Compiler Architecture Targets Specification

## Purpose

Defines the `compiler-architecture-targets` capability.

## Requirements

### Requirement: Explicit Cargo target coverage
r[octet.architecture.targets.coverage]

The collector MUST derive selected compilation units from a bounded locked Cargo plan. It MUST support explicit all-target selection without silently omitting non-library Rust targets. Unsupported modes and ambiguous selected unit keys MUST fail closed.

#### Scenario: Complete target plan
- GIVEN explicit library, binary, test, example, benchmark, proc-macro, and build-script compilation units
- WHEN the target plan is accepted
- THEN each selected Rust unit has an expected compiler identity.

#### Scenario: Ambiguous or unsupported plan
- GIVEN duplicate observation keys, an unknown graph version, or an unsupported unit mode
- WHEN planning runs
- THEN no clean coverage result is emitted.

### Requirement: Compiler observations select exact units
r[octet.architecture.targets.binding]

Collector context v2 MUST bind the source root, test configuration, target triple, and resolved features. The collector MUST reject conflicting or unexpected observations for selected packages. Missing units MUST remain coverage failures.

#### Scenario: Same crate with distinct test configuration
- GIVEN normal and test compilations with the same crate name
- WHEN their compiler observations arrive
- THEN each matches only its exact expected unit.

#### Scenario: Wrong configuration or omitted unit
- GIVEN a wrong source root, target, feature set, test configuration, or absent expected unit
- WHEN collector or coverage validation runs
- THEN it rejects the mismatch without reusing another unit's evidence.

### Requirement: Verified publication preserves claim limits
r[octet.architecture.targets.evidence]

Publication MUST include compiling positive and negative controls, current tests, strict Clippy, and generated-contract freshness. Compilation failure MUST NOT count as architecture rejection. Library-only or source-only evidence MUST NOT establish all-target compiler acceptance.

#### Scenario: Actual compiler control
- GIVEN a compiled pure or effectful fixture and current collector artifacts
- WHEN the evidence is evaluated
- THEN it records exact unit coverage and observed effects separately from lint findings.

#### Scenario: Missing compiler evidence
- GIVEN only planner tests or source inspection
- WHEN publication is requested
- THEN all-target acceptance remains blocked.

### Requirement: Cargo-resolved all-feature equivalence
r[octet.architecture.features.equivalence]

The collector MUST accept `--all-features` only when a bounded locked Cargo plan equals the plan for the explicit policy row.
The comparison MUST preserve complete unit descriptions, resolved features, dependency edges, and selected roots.
Unit indices and collection order MUST NOT determine equivalence.

#### Scenario: Complete reviewed row
- GIVEN a policy row that resolves to every feature and target selected by the invocation
- WHEN the collector receives `--all-targets --all-features`
- THEN it accepts the selection and binds compiler units to the actual Cargo-resolved features.

#### Scenario: Omitted feature or dependency
- GIVEN a policy row that omits a selected feature, feature-gated target, optional dependency, or dependency edge
- WHEN the actual and declared plans differ
- THEN the collector rejects the invocation before publishing a collector context or clean architecture evidence.

### Requirement: Feature selection fails closed
r[octet.architecture.features.rejection]

All-feature support MUST retain exact target, default-feature, production-package, and compiler-shard checks.
It MUST reject mixed or repeated feature selectors and unsupported shorthand.
The comparison MUST reject malformed, incomplete, or ambiguous graph facts.

#### Scenario: Ambiguous or mismatched input
- GIVEN mixed feature selectors, repeated selectors, wrong defaults, wrong targets, or missing graph references
- WHEN collection starts
- THEN it returns an integration error without a clean architecture result.

#### Scenario: Missing actual compiler unit
- GIVEN equivalent accepted plans and a missing expected compiler shard
- WHEN the collector checks coverage
- THEN coverage remains incomplete and the invocation fails.

### Requirement: All-feature publication evidence
r[octet.architecture.features.evidence]

Publication MUST include actual compiling all-target controls for explicit and all-feature invocations.
It MUST include a full deny-all consumer pass and a rejected under-declared policy row.
It MUST retain focused tests, strict Clippy, formatting, and lifecycle evidence.
Planner-only, warning-only, or policy-disabled results MUST NOT establish all-feature acceptance.

#### Scenario: Executed positive and negative controls
- GIVEN current runner and collector artifacts with an active architecture policy
- WHEN the full deny-all hook checks complete and incomplete explicit policy rows with `--all-features`
- THEN the complete row passes and the incomplete row fails before compiler acceptance.

#### Scenario: Evidence lacks actual compilation
- GIVEN only source inspection or successful unit planning
- WHEN publication is requested
- THEN acceptance remains incomplete.
