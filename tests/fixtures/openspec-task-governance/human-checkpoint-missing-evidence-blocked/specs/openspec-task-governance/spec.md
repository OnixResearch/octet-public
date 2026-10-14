## ADDED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

Every `H<n>` task MUST include `command=`, `input=`, `asserts=`, and
`evidence=`.

#### Scenario: Human checkpoint missing evidence path is blocked

- GIVEN `tasks.md` contains `H1 Human/oracle checkpoint: command=...; input=...; asserts=...`
- WHEN the tasks-stage gate runs
- THEN it reports that `H1` is missing the required `evidence=` field
