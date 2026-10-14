## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

Active `tasks.md` files MUST keep `## Change Status` immediately before
`## Verification Coverage`.

#### Scenario: Missing change status is blocked

- GIVEN active `tasks.md` omits `## Change Status`
- WHEN the tasks-stage gate runs
- THEN it reports `tasks.md is missing ## Change Status`
