## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

`Status: complete` MUST be rejected when unchecked governed task IDs remain.

#### Scenario: Complete status with open tasks is blocked

- GIVEN active `tasks.md` says `- Status: complete`
- AND at least one governed task checkbox remains unchecked
- WHEN the tasks-stage gate runs
- THEN it reports `Change status says complete but unchecked tasks remain`
