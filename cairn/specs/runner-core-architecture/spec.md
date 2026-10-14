# Runner Core Architecture Specification

## Purpose

Captures the module-boundary expectations that keep Octet runner behavior split into deterministic pure cores, thin side-effect shells, and reusable integration-test support.

## Requirements

### Requirement: Runner functional-core boundaries

r[octet.runner_core_architecture.boundaries] Octet runner modules SHOULD separate deterministic domain decisions from filesystem, process, environment, and output side effects.

#### Scenario: Pure validator is directly testable

r[octet.runner_core_architecture.boundaries.pure_validator]

- GIVEN runner behavior validates an object, proof receipt, supplement receipt, CLI mode, or artifact identity
- WHEN the validation logic is exercised in tests
- THEN it SHOULD accept in-memory inputs and return deterministic outputs or diagnostics without reading files, spawning processes, or printing output.

#### Scenario: Shell owns side effects

r[octet.runner_core_architecture.boundaries.shell_effects]

- GIVEN a command reads files, writes artifacts, spawns tools, or prints output
- WHEN the command is implemented
- THEN those side effects SHOULD be isolated in a thin shell that delegates deterministic decisions to pure core functions.

### Requirement: Hotspot extraction

r[octet.runner_core_architecture.hotspot_extraction] Large runner and standards hotspots SHOULD be split by behavior seams before adding more unrelated responsibilities.

#### Scenario: Behavior seam gets its own module

r[octet.runner_core_architecture.hotspot_extraction.seam]

- GIVEN a large module contains independent behavior seams such as CLI parsing, object construction, supplement comparison, proof planning, or artifact writing
- WHEN the module is refactored
- THEN each seam SHOULD move to a focused module with a small public API and direct tests.

#### Scenario: CLI output remains stable

r[octet.runner_core_architecture.hotspot_extraction.stable_output]

- GIVEN a behavior seam is extracted from a CLI command
- WHEN existing golden or transcript tests run
- THEN user-visible output, exit codes, and artifact formats SHOULD remain stable unless the change explicitly updates the contract.

### Requirement: Integration test support modules

r[octet.runner_core_architecture.test_support] Large integration tests SHOULD use reusable test-support builders for repeated fixture setup while keeping scenario assertions visible in the test body.

#### Scenario: Repeated fixture setup is shared

r[octet.runner_core_architecture.test_support.shared]

- GIVEN multiple integration tests construct equivalent workspace, artifact, proof, or receipt fixtures
- WHEN the tests are refactored
- THEN repeated setup SHOULD move to named test-support helpers.

#### Scenario: Scenario intent stays visible

r[octet.runner_core_architecture.test_support.visible]

- GIVEN a test uses shared fixture helpers
- WHEN a reviewer reads the test body
- THEN the behavior under test, expected success case, and expected failure case SHOULD remain explicit in the test body.

### Requirement: Effects enter cores through explicit shell boundaries

r[octet.runner_core_architecture.explicit_effects] Octet runner and standards cores SHOULD receive explicit in-memory inputs, typed command results, artifact bytes, capability tokens, or execution plans rather than reading or performing platform effects directly.

#### Scenario: Shell captures ambient inputs

r[octet.runner_core_architecture.explicit_effects.shell_captures]

- GIVEN a command needs filesystem traversal, environment variables, process execution, wall-clock time, randomness, logging, metrics, network access, async-runtime handles, or output writing
- WHEN the command is implemented
- THEN the shell SHOULD acquire or execute that effect and pass typed values or results into the pure core.
- AND the pure core SHOULD return decisions, diagnostics, structured events, or plans rather than performing the effect itself.

#### Scenario: Core rejects ambient effect access

r[octet.runner_core_architecture.explicit_effects.core_rejects_ambient]

- GIVEN a migrated pure core is exercised in tests
- WHEN the test runs without filesystem setup, process spawning, environment mutation, clocks, randomness, logging sinks, metrics sinks, network access, or async runtime setup
- THEN the core SHOULD still be directly testable from in-memory inputs.

#### Scenario: Effect boundary preserves claim limits

r[octet.runner_core_architecture.explicit_effects.non_claim]

- GIVEN documentation or reports describe explicit effect seams
- WHEN reviewers inspect the wording
- THEN the wording MUST NOT claim Octet provides a full Rust effect system, whole-program purity proof, or complete side-effect absence guarantee.
- AND it SHOULD state that Octet validates declared seams and observed source shapes within configured scope.

### Requirement: Capability traits remain narrow

r[octet.runner_core_architecture.capability_traits] When Octet uses Rust traits or tokens to represent shell-injected capabilities, they SHOULD be small, explicit, and limited to one effect family or domain operation.

#### Scenario: Broad service object is avoided

r[octet.runner_core_architecture.capability_traits.narrow]

- GIVEN a pure core needs deferred access to an operation that cannot be represented as a concrete input value
- WHEN a capability is introduced
- THEN the capability SHOULD expose only the operations needed for that effect family.
- AND it SHOULD NOT become a broad service object combining filesystem, environment, process, logging, metrics, clock, random, and network access.

### Requirement: Effect seams have positive and negative tests

r[octet.runner_core_architecture.effect_tests] Effect seam tests SHOULD prove both successful shell-to-core data flow and fail-closed handling for missing, malformed, timed-out, denied, or failed effect results.

#### Scenario: Shell tests cover success and failure

r[octet.runner_core_architecture.effect_tests.positive_negative]

- GIVEN a shell captures an effect and passes typed results into a pure core
- WHEN tests exercise the seam
- THEN positive tests SHOULD verify accepted effect results produce the expected decisions or artifacts.
- AND negative tests SHOULD verify missing data, malformed data, denied execution, timeout, non-zero status, stale identity, or unsupported capability errors produce deterministic diagnostics.
