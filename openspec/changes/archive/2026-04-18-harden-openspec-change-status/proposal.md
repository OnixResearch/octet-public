# Track OpenSpec Change Status From Task State

## Why

Repo-local review history now shows the same tasks-stage omission pattern often
enough to promote: a diff lands one slice of work, but the active OpenSpec
change still has many unchecked tasks and reviewers have to infer whether the
change is partially done or actually ready to close. The current
`openspec-task-governance` rules split implementation from proof and require
coverage rows, but they do not force `tasks.md` itself to say whether open
checkboxes remain.

Without an explicit status contract, partial progress keeps getting summarized
like full change completion.

## What Changes

- This active change hardens repo-local OpenSpec task governance. The other
  active changes still keep their own feature scope, and this same change will
  update these sibling `tasks.md` files to add `## Change Status` in the same
  rollout as the tightened gate:
  `openspec/changes/add-performance-review-kit/tasks.md`,
  `openspec/changes/add-assertion-review-kit/tasks.md`,
  `openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`,
  `openspec/changes/add-concurrency-state-review-kit/tasks.md`,
  `openspec/changes/clarify-safety-critical-claims/tasks.md`,
  `openspec/changes/add-system-comprehension-guidance/tasks.md`, and
  `openspec/changes/add-resilience-review-kit/tasks.md`.
- Extend the existing canonical main spec at
  `openspec/specs/openspec-task-governance/spec.md`; this change does not add a
  new main spec path.
- Add an exact `## Change Status` contract for active `tasks.md` files using
  the existing `openspec-task-governance` scope, reusing the canonical rule
  that governed task IDs stay unique within a file, and drive that status from
  the governed `I<n>`, `V<n>`, and `H<n>` task checkboxes.
- Require in-progress files to list every open task ID explicitly, and allow
  `Status: complete` only when every governed `I<n>`, `V<n>`, and `H<n>` task
  checkbox is checked.
- Harden the repo task template and tasks-stage gate with blocking findings for
  missing or inconsistent change-status sections.
- Prove pass cases with deterministic snippet artifacts and fail cases with
  dedicated `tests/fixtures/openspec-task-governance/` fixtures plus saved
  gate transcripts under
  `openspec/changes/harden-openspec-change-status/evidence/`.

## Non-Goals

- No lint-engine, rollout, or review-kit behavior changes.
- No retroactive rewrite of archived changes.
- No attempt to infer completion from commit messages or external summaries
  outside `tasks.md`.

## Capabilities

### Modified Capabilities

- `openspec-task-governance`: active changes now expose explicit change status
  and open-task lists so partial work cannot be mistaken for a complete change.

## Impact

- **Files**: change-local delta spec plus follow-on updates to
  `openspec/templates/tasks.md`, repo-local task-stage gate guidance or logic,
  `tests/fixtures/openspec-task-governance/`,
  `openspec/changes/add-performance-review-kit/tasks.md`,
  `openspec/changes/add-assertion-review-kit/tasks.md`,
  `openspec/changes/add-bounds-and-pacing-review-kit/tasks.md`,
  `openspec/changes/add-concurrency-state-review-kit/tasks.md`,
  `openspec/changes/clarify-safety-critical-claims/tasks.md`,
  `openspec/changes/add-system-comprehension-guidance/tasks.md`, and
  `openspec/changes/add-resilience-review-kit/tasks.md`.
- **APIs**: repo-local `tasks.md` contract and tasks-stage gate behavior.
- **Dependencies**: None.
- **Testing**: Save deterministic artifacts for the exact template snippet, a
  passing in-progress status fixture, a passing complete status fixture, a
  passing fixture that ignores non-`I/V/H` checkboxes, a rollout artifact for
  the seven real active sibling change files, and blocking gate transcripts for missing
  status, malformed or misplaced status, complete-with-open-tasks,
  in-progress-without-open-tasks, and open-task-list mismatch cases under
  `openspec/changes/harden-openspec-change-status/evidence/`. Each saved gate
  transcript begins with the exact replay command line
  `tests/fixtures/openspec-task-governance/run-fixture-gate.sh <fixture-name>`.
