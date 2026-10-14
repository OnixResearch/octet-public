## MODIFIED Requirements

### Requirement: Task Template Shows Change-Status Fields

Tigerstyle's repo-local task template at `openspec/templates/tasks.md` MUST
continue to include a `## Change Status` section immediately before
`## Verification Coverage` with the existing required example lines and note.

The template MUST also include an omission-prevention prompt that tells authors
to split implementation from verification, give every scenario its own coverage
row, and make `H<n>` oracle tasks self-verifiable with exact `command=`,
`input=`, `asserts=`, and `evidence=` fields.

#### Scenario: Maintainer sees repeated task omission checks before gate review

- GIVEN a maintainer copies `openspec/templates/tasks.md`
- WHEN they write implementation, verification, and human/oracle tasks
- THEN the template reminds them not to combine implementation with evidence
  capture in a single `I<n>` task
- AND it reminds them that every added or modified scenario needs its own
  verification coverage row
- AND it reminds them that each `H<n>` task needs exact replay fields

### Requirement: Human/Oracle Checkpoints Name Exact Inputs

Every `H<n>` checkbox MUST continue to use the exact field layout with
`command=`, `input=`, `asserts=`, and `evidence=`.

The repo-local tasks guidance MUST also describe the repeated omission risk:
human/oracle tasks are not self-verifiable when they cite a broad command,
implicit input, multi-claim assertion, or missing evidence path.

#### Scenario: Human checkpoint guidance blocks implicit oracle inputs

- GIVEN a maintainer writes an `H<n>` task
- WHEN the task names a human/oracle checkpoint
- THEN repo-local guidance tells them to provide the exact replay command or
  helper path
- AND it tells them to provide the exact pinned input or prerequisite artifact
- AND it tells them to provide one assertion and one evidence path

## ADDED Requirements

### Requirement: Tasks-Stage Config Guidance Names Omission Checks

`openspec/config.yaml` MUST include tasks-stage rules that mention the repeated
omission-prevention prompts for split implementation/proof tasks, one-row-per-
scenario coverage, and exact human/oracle replay fields.

#### Scenario: Config task guidance includes promoted task omission checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN tasks-stage prompts are prepared for a new change
- THEN the tasks rules mention split implementation, verification, and
  human/oracle tasks
- AND the tasks rules mention one verification coverage row per scenario
- AND the tasks rules mention exact `H<n>` `command=`, `input=`, `asserts=`,
  and `evidence=` fields
