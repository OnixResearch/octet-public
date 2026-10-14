## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

In-progress `Open tasks` lines MUST match unchecked governed task IDs exactly.

#### Scenario: Incorrect open-task list is blocked

- GIVEN active `tasks.md` says `- Status: in-progress`
- AND its `Open tasks` line does not match unchecked task IDs in file order
- WHEN the tasks-stage gate runs
- THEN it reports `Change status open-task list does not match unchecked tasks`
