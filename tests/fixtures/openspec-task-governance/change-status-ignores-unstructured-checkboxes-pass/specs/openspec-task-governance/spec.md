## ADDED Requirements

### Requirement: Change Status Mirrors Task State

Ordinary markdown checkboxes that do not start with `I<n>`, `V<n>`, or `H<n>`
MUST NOT change the open-task list.

#### Scenario: Unstructured checkboxes do not change status

- GIVEN unchecked ordinary markdown checkboxes appear in `tasks.md`
- WHEN a reviewer reads `## Change Status`
- THEN `Open tasks` still names only unchecked governed task IDs
