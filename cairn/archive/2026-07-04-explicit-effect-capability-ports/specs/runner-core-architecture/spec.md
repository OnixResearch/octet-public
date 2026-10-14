# Runner Core Architecture Specification Delta

## ADDED Requirements

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
