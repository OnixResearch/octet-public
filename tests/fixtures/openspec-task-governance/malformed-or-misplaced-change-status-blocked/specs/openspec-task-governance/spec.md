## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

Active `tasks.md` files MUST keep `## Change Status` immediately before
`## Verification Coverage`.

#### Scenario: Malformed or misplaced change status is blocked

- GIVEN active `tasks.md` includes `## Change Status`
- AND the section is malformed or not placed immediately before `## Verification Coverage`
- WHEN the tasks-stage gate runs
- THEN it reports `Change status section is malformed or misplaced`
