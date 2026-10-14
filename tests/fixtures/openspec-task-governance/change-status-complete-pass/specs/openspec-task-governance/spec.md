## ADDED Requirements

### Requirement: Change Status Mirrors Task State

Active `tasks.md` files MUST use the complete-status shape only when every
governed task checkbox is checked.

#### Scenario: Complete status is only valid when all tasks are checked

- GIVEN every governed task checkbox is checked
- WHEN a reviewer reads `## Change Status`
- THEN the file says `- Status: complete`
- AND it says `- Open tasks: none`
