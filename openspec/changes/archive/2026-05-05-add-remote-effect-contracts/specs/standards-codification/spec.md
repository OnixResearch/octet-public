## ADDED Requirements

### Requirement: Remote Execution Effect Contract [r[remote-execution-effect-contract]]

The system MUST document remote execution as a first-class effect family that requires explicit timeout, retry bound, batch bound, idempotency key, and receipt or replay semantics.

#### Scenario: Remote task carries bounds [r[remote-execution-effect-contract-scenario-remote-task-carries-bounds]]

- GIVEN a design schedules remote work
- WHEN the remote effect contract is applied
- THEN the task model names timeout, max retries, idempotency key, and batch bound

#### Scenario: Receipt semantics are explicit [r[remote-execution-effect-contract-scenario-receipt-semantics-are-explicit]]

- GIVEN remote work completes or fails
- WHEN review evidence is captured
- THEN the receipt records enough identity to replay, correlate, or audit the attempted work

### Requirement: Remote Lint Staging [r[remote-lint-staging]]

The system MUST keep remote-effect enforcement advisory or checklist-based until fixtures prove a low false-positive static signal.

#### Scenario: Weak static signal remains human reviewed [r[remote-lint-staging-scenario-weak-static-signal-remains-human-reviewed]]

- GIVEN remote execution APIs differ by consumer
- WHEN a checker is proposed
- THEN the policy records human-process or advisory classification before hard gating
