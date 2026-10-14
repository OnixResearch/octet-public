## ADDED Requirements

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
