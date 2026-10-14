# openspec-task-governance Specification

## Purpose

Define tigerstyle's repo-local OpenSpec task-governance contract for active
changes: typed implementation/verification/human checkpoints, explicit
scenario-to-evidence coverage rows, and replayable human/oracle task fields.
## Requirements
### Requirement: Structured Task Units

Every active tigerstyle `tasks.md` file for a new or modified capability MUST
use structured checkbox task IDs and keep implementation, verification, and
human/oracle claims in separate checkbox units.

For this capability, "active" means a change directory directly under
`openspec/changes/` and explicitly excludes `openspec/changes/archive/**`.

The supported task-ID classes are:

- `I<n>` for implementation work
- `V<n>` for deterministic verification work
- `H<n>` for human/oracle checkpoints

Those IDs MUST be unique within the file.

`I<n>` tasks MUST describe one implementation claim and MUST NOT also claim
verification or evidence capture in the same checkbox.

`V<n>` tasks MUST verify exactly one named scenario or one requirement-level
invariant that has no scenario heading, and each `V<n>` task MUST name the
exact command/helper, required input or artifact, evidence path, and acceptance
signal needed to prove that scenario.

`H<n>` tasks MUST confirm exactly one human/oracle claim.

#### Scenario: Implementation and verification are split

- GIVEN a change needs code edits and a saved test transcript
- WHEN the maintainer writes `tasks.md`
- THEN the code edit lives in an `I<n>` checkbox
- AND the saved transcript lives in a separate `V<n>` or `H<n>` checkbox

#### Scenario: One scenario does not hide inside a bundled checkbox

- GIVEN a delta spec adds a scenario named `Explicit warn override still warns`
- WHEN the maintainer adds verification tasks
- THEN one `V<n>` task names that scenario directly
- AND the same checkbox does not also claim unrelated scenario coverage

#### Scenario: Task obligations map scenarios to exact proof

- GIVEN a delta spec scenario requires an omitted-field default, explicit
  override, required artifact, negative control, or continued-warning behavior
- WHEN the maintainer writes a `V<n>` verification task for that scenario
- THEN the task names the exact scenario, command or helper, input or fixture,
  saved evidence path, and expected acceptance signal
- AND the task cannot be satisfied by a broader check that omits that
  scenario-specific behavior

### Requirement: Verification Coverage Matrix

Every active tigerstyle `tasks.md` file for a change with delta specs MUST end
with a `## Verification Coverage` section.

That section MUST contain one mapping row for every added or modified scenario
heading, plus one row for any requirement-level invariant that has no scenario
heading.

A `Requirement:` row is only valid when that requirement has zero scenario
headings in the parsed delta spec. If a requirement has one or more scenarios,
those scenario rows are mandatory and a requirement row MUST NOT replace them.

Each mapping row MUST use this exact shape:

- `` `Scenario: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``
- `` `Requirement: <heading>` -> `V<n>|H<n>` -> `<repo-relative evidence path>` ``

A scenario MAY share the same evidence artifact as another scenario, but it MUST
still keep its own mapping row.

#### Scenario: Reviewer can trace a scenario to proof

- GIVEN a reviewer reads a delta spec scenario
- WHEN they open `tasks.md`
- THEN they can find one exact coverage row for that scenario
- AND that row names the task ID and saved evidence path without guesswork

#### Scenario: Shared transcript still keeps separate scenario rows

- GIVEN two scenarios are proven by the same saved UI transcript
- WHEN the maintainer writes the coverage section
- THEN both scenarios still appear as separate rows
- AND both rows may point at the same evidence path

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

### Requirement: Tasks-Stage Gate Flags Missing Governance

The system MUST make tigerstyle's tasks-stage gate report the existing
canonical `openspec-task-governance` blocking findings, repeated task-obligation
omissions where a required scenario, artifact, exact acceptance signal, negative
control, or continued-warning case is not represented by an explicit
`V<n>`/coverage mapping, plus these additional change-status blocking findings:

- `tasks.md is missing ## Change Status`
- `Change status section is malformed or misplaced`
- `Change status says complete but unchecked tasks remain`
- `Change status says in-progress but no open tasks remain`
- `Change status open-task list does not match unchecked tasks`

#### Scenario: Missing change status is blocked

- GIVEN active `tasks.md` omits `## Change Status`
- WHEN the tasks-stage gate runs
- THEN it reports `tasks.md is missing ## Change Status`

#### Scenario: Malformed or misplaced change status is blocked

- GIVEN active `tasks.md` includes `## Change Status`
- AND the section is malformed or not placed immediately before
  `## Verification Coverage`
- WHEN the tasks-stage gate runs
- THEN it reports `Change status section is malformed or misplaced`

#### Scenario: Complete status with open tasks is blocked

- GIVEN active `tasks.md` says `- Status: complete`
- AND at least one governed task checkbox remains unchecked
- WHEN the tasks-stage gate runs
- THEN it reports `Change status says complete but unchecked tasks remain`

#### Scenario: In-progress status with no open tasks is blocked

- GIVEN active `tasks.md` says `- Status: in-progress`
- AND every governed task checkbox is checked
- WHEN the tasks-stage gate runs
- THEN it reports `Change status says in-progress but no open tasks remain`

#### Scenario: Incorrect open-task list is blocked

- GIVEN active `tasks.md` says `- Status: in-progress`
- AND its `Open tasks` line does not match the unchecked task IDs in file order
- WHEN the tasks-stage gate runs
- THEN it reports `Change status open-task list does not match unchecked tasks`

#### Scenario: Task obligation fixtures prove blocked and covered tasks

- GIVEN the fixture `tests/fixtures/openspec-task-governance/task-obligation-missing-blocked`
- AND the fixture `tests/fixtures/openspec-task-governance/task-obligation-covered-pass`
- WHEN `tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh`
  runs each fixture through the tasks gate
- THEN the missing fixture is accepted only when the gate blocks the omitted
  task obligation
- AND the covered fixture is accepted only when the gate passes explicit
  task/evidence mappings
- AND saved evidence for this change is written only under this change's
  evidence directory and eventual archive copy, not by editing historical
  archives

### Requirement: Task Template Shows Change-Status Fields

Tigerstyle's repo-local task template at `openspec/templates/tasks.md` MUST
continue to include a `## Change Status` section immediately before
`## Verification Coverage` with the existing required example lines and note.

The template MUST also include omission-prevention prompts that tell authors to
split implementation from verification, give every scenario its own coverage
row, make `H<n>` oracle tasks self-verifiable with exact `command=`, `input=`,
`asserts=`, and `evidence=` fields, and make each `V<n>` task name the exact
scenario, command/helper, required artifact or fixture, evidence path, and
acceptance signal.

#### Scenario: Maintainer sees repeated task omission checks before gate review

- GIVEN a maintainer copies `openspec/templates/tasks.md`
- WHEN they write implementation, verification, and human/oracle tasks
- THEN the template reminds them not to combine implementation with evidence
  capture in a single `I<n>` task
- AND it reminds them that every added or modified scenario needs its own
  verification coverage row
- AND it reminds them that each `H<n>` task needs exact replay fields

#### Scenario: Task obligations map scenarios to exact proof

- GIVEN a delta spec scenario requires an omitted-field default, explicit
  override, required artifact, negative control, or continued-warning behavior
- WHEN the maintainer writes `V<n>` tasks from the template
- THEN the template tells them to name that exact scenario, command/helper,
  input or fixture, evidence path, and expected PASS output or diagnostic
- AND it warns that broad tasks such as `test config parsing` are insufficient
  when the scenario requires a specific absent-field default, explicit override,
  or continued-warning case

### Requirement: Change Status Mirrors Task State

Every active tigerstyle `tasks.md` file MUST include a `## Change Status`
section immediately before `## Verification Coverage`.

For this requirement, checkbox state means checkbox lines whose task text
begins with an `I<n>`, `V<n>`, or `H<n>` ID. This requirement reuses the
existing governed-task rule that those task IDs are unique within the file.
Files with zero governed task checkboxes remain invalid under the existing
Structured Task Units requirement; this change does not redefine that baseline.

That section MUST use one of these exact shapes. In active files, the
`in-progress` `Open tasks` line uses `, ` as the separator between task IDs,
with no trailing comma or extra brackets:

- `## Change Status`
- `- Status: in-progress`
- `- Open tasks: <comma-separated unchecked task IDs in file order>`

or

- `## Change Status`
- `- Status: complete`
- `- Open tasks: none`

If any governed task checkbox remains unchecked, `Status` MUST be
`in-progress` and `Open tasks` MUST list every unchecked `I<n>`, `V<n>`, and
`H<n>` task ID exactly once in file order.

If every governed task checkbox in the file is checked, the section MUST say
`- Status: complete` and `- Open tasks: none`.

`Status: complete` is only valid when every governed task checkbox in the
file is checked.

#### Scenario: Unchecked tasks keep change in progress

- GIVEN an active `tasks.md` file still has unchecked governed task
  checkboxes
- WHEN a reviewer reads `## Change Status`
- THEN the file says `- Status: in-progress`
- AND `- Open tasks: ...` names every unchecked task ID in file order

#### Scenario: Complete status is only valid when all tasks are checked

- GIVEN an active `tasks.md` file has every governed task checkbox checked
- WHEN a reviewer reads `## Change Status`
- THEN the file says `- Status: complete`
- AND it says `- Open tasks: none`

#### Scenario: Unstructured checkboxes do not change status

- GIVEN an active `tasks.md` file also contains ordinary markdown checkboxes
  that do not start with `I<n>`, `V<n>`, or `H<n>`
- WHEN a reviewer reads `## Change Status`
- THEN `Open tasks` still names only the unchecked governed task IDs

### Requirement: Current Active Changes Are Migrated Alongside Gate Hardening

The currently active sibling task files below MUST gain compliant
`## Change Status` sections in the same rollout as the hardened change-status
gate:

- `openspec/changes/add-performance-review-kit/tasks.md`
- `openspec/changes/add-assertion-review-kit/tasks.md`
- `openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`
- `openspec/changes/add-concurrency-state-review-kit/tasks.md`
- `openspec/changes/clarify-safety-critical-claims/tasks.md`
- `openspec/changes/add-system-comprehension-guidance/tasks.md`
- `openspec/changes/add-resilience-review-kit/tasks.md`

Each file MUST keep `## Change Status` immediately before
`## Verification Coverage`, and each `Open tasks` line MUST match that file's
unchecked governed task IDs.

#### Scenario: Existing active changes stay compliant during rollout

- GIVEN the hardened gate is ready to enforce repo-wide
- WHEN a maintainer opens the seven currently active sibling task files above
- THEN each file has a compliant `## Change Status` section immediately before
  `## Verification Coverage`
- AND each `Open tasks` line matches that file's unchecked governed task IDs

### Requirement: Replay Helper Cleans Up Temporary Active Changes

The system MUST ensure the replay helper
`tests/fixtures/openspec-task-governance/run-fixture-gate.sh` removes the
temporary `openspec/changes/<fixture-name>/` directory after each fixture gate
run finishes.

#### Scenario: Helper cleanup check

- GIVEN the helper replays fixture `missing-change-status-blocked`
- WHEN the helper exits
- THEN `openspec/changes/missing-change-status-blocked/` no longer exists

### Requirement: Tasks-Stage Config Guidance Names Omission Checks

`openspec/config.yaml` MUST include tasks-stage rules that mention the repeated
omission-prevention prompts for split implementation/proof tasks, one-row-per-
scenario coverage, exact per-scenario verification acceptance signals, required
artifacts/fixtures, and exact human/oracle replay fields.

#### Scenario: Config task guidance includes promoted task omission checks

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN tasks-stage prompts are prepared for a new change
- THEN the tasks rules mention split implementation, verification, and
  human/oracle tasks
- AND the tasks rules mention one verification coverage row per scenario
- AND the tasks rules mention exact `V<n>` scenario commands, required artifacts
  or fixtures, evidence paths, and acceptance signals
- AND the tasks rules mention exact `H<n>` `command=`, `input=`, `asserts=`,
  and `evidence=` fields
