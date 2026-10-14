## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

`Status: in-progress` MUST be rejected when no governed task remains open.

#### Scenario: In-progress status with no open tasks is blocked

- GIVEN active `tasks.md` says `- Status: in-progress`
- AND every governed task checkbox is checked
- WHEN the tasks-stage gate runs
- THEN it reports `Change status says in-progress but no open tasks remain`
