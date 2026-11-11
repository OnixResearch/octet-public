## ADDED Requirements

### Requirement: Task Template Shows Change-Status Fields

Tigerstyle's repo-local task template at `openspec/templates/tasks.md` MUST
include a `## Change Status` section immediately before
`## Verification Coverage`.

That template section MUST keep these exact example lines in order:

- `## Change Status`
- `- Status: in-progress`
- `- Open tasks: I1, I2, V1, V2, H1`

The example `Open tasks` line MUST match the unchecked placeholder task IDs
that appear earlier in the template, in file order.

The template MUST also include this exact note after the example lines:

`Use \`Open tasks: none\` only when every governed \`I<n>\`, \`V<n>\`, and \`H<n>\` task checkbox is checked.`

#### Scenario: Template shows change-status fields

- GIVEN a maintainer copies `openspec/templates/tasks.md`
- WHEN they reach the status section before `## Verification Coverage`
- THEN they see the exact `in-progress` example lines above
- AND the example `Open tasks` line matches the placeholder task IDs in file
  order
- AND they see the exact `Open tasks: none` note above

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

## MODIFIED Requirements

### Requirement: Tasks-Stage Gate Flags Missing Governance

The system MUST make tigerstyle's tasks-stage gate report the existing
canonical `openspec-task-governance` blocking findings plus these additional
change-status blocking findings:

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
