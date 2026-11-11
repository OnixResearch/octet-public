## Why

Tigerstyle's worktree is clean and the current test rails are green, but repo-
local review history shows the next bottleneck is OpenSpec task churn rather
than missing lint behavior. `review_metrics promotions` currently shows repeated
tasks-stage findings in the same three buckets:

- omission / tasks / prompt = 32
- omission / tasks / human = 21
- incoherent / tasks / human = 12

Recent changes such as `per-lint-configuration` and `autofix-suggestions`
needed repeated retries because task checkboxes bundled multiple auditable
claims, spec scenarios were not mapped cleanly to verification tasks, and human
validation steps named intent without the exact command, pinned input, or saved
evidence path.

Without a repo-local task-writing contract, the next OpenSpec change will pay
that same review tax again.

## What Changes

- add repo-local OpenSpec task-governance requirements for one auditable claim
  per checkbox class
- require explicit mapping from every added or modified spec scenario to at
  least one verification task and saved evidence path
- require human/oracle validation tasks to name the exact command or helper,
  pinned input or revision, asserted claim, and saved evidence path
- define how the repo's task template and tasks-stage gate enforce those rules
  with deterministic failure cases drawn from recent review history

## Non-Goals

- no lint-engine, README, or downstream rollout behavior changes
- no proposal-stage or design-stage gate hardening beyond what tasks-stage
  enforcement needs
- no retroactive rewrite of archived changes outside fixture or evidence use

## Capabilities

### New Capabilities
- `openspec-task-governance`: tigerstyle OpenSpec changes carry auditable task
  granularity, scenario-to-proof mapping, and explicit evidence-path contracts
  that the tasks-stage gate can check before implementation is considered done

## Impact

- **Files**: new OpenSpec delta spec plus follow-on changes to repo-local task
  templates/examples, task-stage gate guidance or logic, and fixture changes
  used to prove failure and pass cases
- **APIs**: repo-local `tasks.md` contract; possible follow-on changes to
  `openspec_gate` task-stage behavior
- **Dependencies**: none
- **Testing**: run proposal/design/tasks gates plus saved fixture changes that
  prove bundled claims, missing scenario coverage, and under-specified
  human/oracle validation are rejected while compliant tasks pass
