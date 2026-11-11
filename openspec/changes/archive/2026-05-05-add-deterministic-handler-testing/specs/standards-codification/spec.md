## ADDED Requirements

### Requirement: Deterministic Handler Test Guidance [r[deterministic-handler-test-guidance]]

The system MUST document deterministic handler testing for effectful boundaries, including fixed clocks, seeded randomness, recording stores, fake remotes, and configured failure handlers.

#### Scenario: Handler test is ambient-free [r[deterministic-handler-test-guidance-scenario-handler-test-is-ambient-free]]

- GIVEN a test exercises effect-boundary logic
- WHEN deterministic handler guidance is applied
- THEN the test uses injected handler state rather than live time, randomness, network, or filesystem

#### Scenario: Recording handler proves requested effects [r[deterministic-handler-test-guidance-scenario-recording-handler-proves-requested-effects]]

- GIVEN core code emits effect descriptions
- WHEN a recording handler test runs
- THEN the test can assert the exact ordered effects without performing real side effects

### Requirement: Invalid Path Handler Evidence [r[invalid-path-handler-evidence]]

The system MUST require handler tests to cover invalid paths and cleanup obligations where the effect contract includes failure behavior.

#### Scenario: Failure handler proves cleanup [r[invalid-path-handler-evidence-scenario-failure-handler-proves-cleanup]]

- GIVEN an effect handler can fail after partial progress
- WHEN invalid-path evidence is captured
- THEN the evidence shows the reported error and cleanup or no-mutation behavior
