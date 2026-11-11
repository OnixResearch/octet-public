# Design: Track OpenSpec Change Status From Task State

## Context

Tigerstyle already hardened `tasks.md` around typed `I<n>`, `V<n>`, and `H<n>`
units, scenario coverage rows, and replayable human checkpoints. That removed a
large class of bundled-claim churn, but repo-local review metrics still show a
repeated tasks omission pattern: a change lands one useful slice while the
active `tasks.md` file still has unchecked governed task checkboxes, and
reviewers have to guess whether the change is merely partial or truly complete.

The gap is not scenario mapping anymore. The gap is status visibility.
`tasks.md` needs one deterministic place where open work is named directly so
partial implementation cannot read like archive-ready completion.

Scope stays limited to repo-local OpenSpec task governance. The active
review-kit changes keep their own feature scope; only their task-governance
shape is affected here.

## Goals / Non-Goals

**Goals:**
- make active `tasks.md` files say whether the change is still in progress or
  actually complete
- make remaining open task IDs visible without scanning the whole file
- let the tasks-stage gate block status sections that contradict checkbox state
- keep the new status contract copyable through `openspec/templates/tasks.md`

**Non-Goals:**
- retroactive archive cleanup
- commit-message or chat-summary policing outside `tasks.md`
- feature-specific implementation work in the review-kit changes

## Decisions

### 1. Every active `tasks.md` gets an exact `## Change Status` section

**Choice:** Active `tasks.md` files keep a `## Change Status` section
immediately before `## Verification Coverage`. Only governed `I<n>`, `V<n>`,
and `H<n>` task checkboxes drive this status contract. A governed checkbox is a
checkbox line whose task text begins with one of those IDs. Files with zero
governed task checkboxes remain invalid under the existing Structured Task
Units baseline rather than gaining a second special case here.

That section uses one of these exact shapes:

- `## Change Status`
- `- Status: in-progress`
- `- Open tasks: <comma-separated unchecked task IDs in file order>`

or

- `## Change Status`
- `- Status: complete`
- `- Open tasks: none`

If every governed task checkbox in the file is checked, the section must use
the `complete` / `none` shape above. The `Open tasks` line otherwise lists
unchecked `I<n>`, `V<n>`, and `H<n>` IDs exactly once, in the same order the
governed task checkboxes appear earlier in the file, with `, ` as the
separator.

**Rationale:** Reviewers should not have to infer progress from a long checklist
or from external prose. One exact status section gives a deterministic answer.

**Alternative considered:** Keep status implicit and rely on reviewers to read
all checkboxes. Rejected because the promoted finding is specifically about
partial work being mistaken for full completion.

### 2. The template ships one exact in-progress example

**Choice:** `openspec/templates/tasks.md` adds this exact example section before
`## Verification Coverage`:

- `## Change Status`
- `- Status: in-progress`
- `- Open tasks: I1, I2, V1, V2, H1`

That example matches the template's unchecked placeholder tasks in file order.
The template also keeps this exact note after the example lines:
`Use \`Open tasks: none\` only when every governed \`I<n>\`, \`V<n>\`, and
\`H<n>\` task checkbox is checked.`

**Rationale:** If the template leaves status prose vague, authors will drift
back into ad hoc summaries.

**Alternative considered:** Add only a prose note about unfinished tasks.
Rejected because the repo already has enough prose guidance; this gap needs an
exact copyable shape.

### 3. The tasks-stage gate blocks contradictory or missing status

Repo-local task-stage gate guidance or supporting checks enforce the detailed
parsing and blocking behavior below.

**Choice:** The hardened tasks-stage gate keeps blocking findings for the new
status failures. At minimum it reports these exact blocking strings:

- `tasks.md is missing ## Change Status`
- `Change status section is malformed or misplaced`
- `Change status says complete but unchecked tasks remain`
- `Change status says in-progress but no open tasks remain`
- `Change status open-task list does not match unchecked tasks`

The gate derives unchecked task IDs from checkbox state, not from the status
section itself, and it also checks that `## Change Status` appears immediately
before `## Verification Coverage`.

**Rationale:** The new governance only helps if contradictory status becomes a
stage-blocking error instead of another review comment.

**Alternative considered:** Emit warnings only. Rejected because the repeated
failure mode is false completion, which should block tasks-stage advancement.

### 4. Proof uses small fixtures and exact saved outputs

**Choice:** Deterministic proof uses fixture-local `tasks.md` files under
`tests/fixtures/openspec-task-governance/` plus saved artifacts under
`openspec/changes/harden-openspec-change-status/evidence/`.

Passing proof uses exact snippet artifacts. Blocking proof uses self-
describing tasks-stage gate transcripts that include the replay command preamble
plus the blocking line.

Saved proof uses these exact outputs:

- `change-status-template-snippet.txt` contains these exact lines in order:
  - `## Change Status`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, V1, V2, H1`
  - `Use \`Open tasks: none\` only when every governed \`I<n>\`, \`V<n>\`, and \`H<n>\` task checkbox is checked.`
  - `## Verification Coverage`
- `change-status-in-progress-pass.txt` contains these exact lines in order:
  - `## Change Status`
  - `- Status: in-progress`
  - `- Open tasks: I2, V1, H1`
  - `## Verification Coverage`
- `change-status-complete-pass.txt` contains these exact lines in order:
  - `## Change Status`
  - `- Status: complete`
  - `- Open tasks: none`
  - `## Verification Coverage`
- `unstructured-checkboxes-ignored-pass.txt` contains these exact lines in
  order:
  - `## Change Status`
  - `- Status: in-progress`
  - `- Open tasks: I2, V1`
  - `## Verification Coverage`
- `active-change-rollout-status.txt` contains these exact lines in order:
  - `file:openspec/changes/add-performance-review-kit/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6`
  - `## Verification Coverage`
  - `file:openspec/changes/add-assertion-review-kit/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6`
  - `## Verification Coverage`
  - `file:openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6`
  - `## Verification Coverage`
  - `file:openspec/changes/add-concurrency-state-review-kit/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V3, V4, V5, V6`
  - `## Verification Coverage`
  - `file:openspec/changes/clarify-safety-critical-claims/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, H1, V1, V2, V3, V4`
  - `## Verification Coverage`
  - `file:openspec/changes/add-system-comprehension-guidance/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V2, V3, V4, V5`
  - `## Verification Coverage`
  - `file:openspec/changes/add-resilience-review-kit/tasks.md`
  - `- Status: in-progress`
  - `- Open tasks: I1, I2, I3, H1, V1, V2, V3, V4, V5`
  - `## Verification Coverage`
- `missing-change-status.txt` is a tasks-stage gate transcript for fixture
  `missing-change-status-blocked` and contains the command preamble
  `tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked`
  plus the blocking line `tasks.md is missing ## Change Status`
- `malformed-or-misplaced-change-status.txt` is a tasks-stage gate transcript
  for fixture `malformed-or-misplaced-change-status-blocked` and contains the
  command preamble `tests/fixtures/openspec-task-governance/run-fixture-gate.sh malformed-or-misplaced-change-status-blocked`
  plus the blocking line `Change status section is malformed or misplaced`
- `complete-status-blocked.txt` is a tasks-stage gate transcript for fixture
  `complete-status-with-open-tasks-blocked` and contains the command preamble
  `tests/fixtures/openspec-task-governance/run-fixture-gate.sh complete-status-with-open-tasks-blocked`
  plus the blocking line `Change status says complete but unchecked tasks remain`
- `in-progress-without-open-tasks-blocked.txt` is a tasks-stage gate transcript
  for fixture `in-progress-without-open-tasks-blocked` and contains the command
  preamble `tests/fixtures/openspec-task-governance/run-fixture-gate.sh in-progress-without-open-tasks-blocked`
  plus the blocking line `Change status says in-progress but no open tasks remain`
- `open-task-list-mismatch.txt` is a tasks-stage gate transcript for fixture
  `open-task-list-mismatch-blocked` and contains the command preamble
  `tests/fixtures/openspec-task-governance/run-fixture-gate.sh open-task-list-mismatch-blocked`
  plus the blocking line `Change status open-task list does not match unchecked tasks`
- `helper-cleanup.txt` contains these exact lines in order:
  - `tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked`
  - `removed:openspec/changes/missing-change-status-blocked`

Fixture-to-artifact mapping stays explicit:

- `change-status-in-progress-pass` -> `change-status-in-progress-pass.txt`
- `change-status-complete-pass` -> `change-status-complete-pass.txt`
- `change-status-ignores-unstructured-checkboxes-pass` -> `unstructured-checkboxes-ignored-pass.txt`
- current active sibling task files -> `active-change-rollout-status.txt`
- `missing-change-status-blocked` -> `missing-change-status.txt`
- `malformed-or-misplaced-change-status-blocked` -> `malformed-or-misplaced-change-status.txt`
- `complete-status-with-open-tasks-blocked` -> `complete-status-blocked.txt`
- `in-progress-without-open-tasks-blocked` -> `in-progress-without-open-tasks-blocked.txt`
- `open-task-list-mismatch-blocked` -> `open-task-list-mismatch.txt`
- helper cleanup check -> `helper-cleanup.txt`

**Rationale:** The rule is simple enough that review should see exact passing
and failing examples, not broad prose.

### 5. Roll out the new status section to current active changes in the same change

**Choice:** The implementation updates the currently active sibling change
files below so they carry compliant `## Change Status` sections in the same
rollout as the hardened tasks-stage gate:

- `openspec/changes/add-performance-review-kit/tasks.md`
- `openspec/changes/add-assertion-review-kit/tasks.md`
- `openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`
- `openspec/changes/add-concurrency-state-review-kit/tasks.md`
- `openspec/changes/clarify-safety-critical-claims/tasks.md`
- `openspec/changes/add-system-comprehension-guidance/tasks.md`
- `openspec/changes/add-resilience-review-kit/tasks.md`

**Rationale:** Tightening the gate in one change while updating the already-
active task files in that same rollout avoids a mixed governance state and does
not strand those changes in a self-inflicted failure mode.

**Alternative considered:** Grandfather all currently active changes. Rejected
because it would keep the repo in a mixed governance state right when the point
of this change is to make status explicit across active work.

## Risks / Trade-offs

**[More task-file ceremony]** -> Authors now maintain one extra section.
Mitigation: keep the section to two bullet lines and ship an exact template
example.

**[Open-task drift]** -> Authors may update checkboxes and forget the status
line. Mitigation: the tasks-stage gate derives unchecked IDs and blocks drift.

**[Archive mismatch fear]** -> Older archived files do not have this section.
Mitigation: this change relies on the existing canonical `openspec-task-governance`
active-change scope, which already excludes `openspec/changes/archive/**`, and
on the earlier archived proof in
`openspec/changes/archive/2026-04-16-harden-openspec-task-gates/` rather than
adding a second archive-only fixture here.
