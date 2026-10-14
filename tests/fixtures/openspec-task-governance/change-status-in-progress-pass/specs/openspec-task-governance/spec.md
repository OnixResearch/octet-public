## ADDED Requirements

### Requirement: Change Status Mirrors Task State

Active `tasks.md` files MUST keep an in-progress status section when unchecked
governed tasks remain.

#### Scenario: Unchecked tasks keep change in progress

- GIVEN `I2` and `V1` remain unchecked
- WHEN a reviewer reads `## Change Status`
- THEN the file says `- Status: in-progress`
- AND `- Open tasks: I2, V1, H1` matches unchecked governed task IDs in file order

#### Scenario: Human checkpoint remains replayable while work stays open

- GIVEN `H1` remains unchecked
- WHEN a reviewer reads the checkpoint and coverage row
- THEN the replay path stays explicit
