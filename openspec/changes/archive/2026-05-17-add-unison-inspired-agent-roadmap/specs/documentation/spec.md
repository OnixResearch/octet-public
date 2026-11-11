## ADDED Requirements

### Requirement: Structured Session Todo State [r[structured-session-todo-state]]

The project MUST document and expose structured session todo states for pending edits, failing checks, unresolved review findings, stale specs, and completed work. The state model MUST make unresolved work distinguishable from completed or intentionally deferred work.

#### Scenario: Session todo state is structured [r[structured-session-todo-state.scenario.todo-state-structured]]

- GIVEN an agent session has pending edits, failing checks, unresolved review findings, and stale OpenSpec tasks
- WHEN session todo state is recorded or displayed
- THEN each item has a stable kind, status, source reference, and next action
- AND the output is machine-readable enough for follow-up agents to resume without parsing prose summaries

#### Scenario: Resolved and unresolved work remain distinguishable [r[structured-session-todo-state.scenario.resolved-unresolved-distinguishable]]

- GIVEN a task was completed, another was deferred, and another remains blocked
- WHEN the session state is inspected later
- THEN completed, deferred, and blocked items are represented as separate states
- AND blocked items preserve the blocker reason and required next evidence
