## ADDED Requirements

### Requirement: Human/Oracle Checkpoints Name Exact Inputs

Every `H<n>` task MUST include `command=`, `input=`, `asserts=`, and
`evidence=`.

#### Scenario: Aspen replay task is self-describing

- GIVEN a maintainer reads the `H1` task
- WHEN they inspect the replay fields
- THEN they can find the replay command or helper
- AND they can find the pinned input
- AND they can find the saved evidence path
