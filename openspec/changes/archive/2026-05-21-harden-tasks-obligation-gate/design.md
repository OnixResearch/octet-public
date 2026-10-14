## Context

Existing tasks governance already splits implementation/verification/human tasks, requires change status, and maps scenarios to evidence rows. The remaining `omission|tasks|auto-fix` findings show a narrower gap: tasks can still be too broad to force every required behavior, artifact, or exact acceptance signal.

## Goals / Non-Goals

**Goals:** make tasks-stage omissions visible before implementation; force exact `V<n>` acceptance signals for each scenario; add deterministic negative/positive fixture coverage; preserve archive closeout evidence.

**Non-Goals:** runtime behavior changes, historical archive rewrites, or human-oracle-specific cluster hardening.

## Decisions

### 1. Tasks template adds task-obligation cross-check guidance

**Choice:** Expand `openspec/templates/tasks.md` so `V<n>` guidance requires exact scenario names, exact commands/helpers, required artifacts, and exact acceptance signals, including negative/continued-warning cases.

**Covers:** `Task Template Shows Change-Status Fields`; `Task obligations map scenarios to exact proof`.

**Obligation cross-check:** Carries the proposal promise to block broad tasks such as “test config parsing” when the delta spec requires a specific absent-field default, explicit `warn` override, required fixture artifact, or continued-warning case.

**Rationale:** Verification coverage rows alone are insufficient if the pointed task is too broad to prove the scenario.

**Boundary cases:** Shared evidence artifacts remain allowed, but each scenario still needs its own task/row or a row pointing to a task that names that exact scenario and acceptance signal.

**Proof plan:** Static helper evidence must print `PASS tasks template includes task obligation fields`; fixture gates must prove missing and covered behavior.

### 2. Config tasks rule mirrors scenario-specific verification obligations

**Choice:** Update `openspec/config.yaml` tasks rules to require per-scenario `V<n>` tasks with exact acceptance signals and required artifacts.

**Covers:** `Tasks-Stage Config Guidance Names Omission Checks`; `Task obligations map scenarios to exact proof`.

**Obligation cross-check:** Makes the new task-obligation checks visible to stage-gate prompts, not only to authors copying the template.

**Rationale:** Existing active changes may not have been freshly copied from the template.

**Proof plan:** Static helper evidence must print `PASS config includes tasks-stage obligation guidance`; tasks gate for this change must pass.

### 3. Fixture pair proves missing and covered task obligations

**Choice:** Add `task-obligation-missing-blocked` and `task-obligation-covered-pass` under `tests/fixtures/openspec-task-governance/` plus a runner that calls the tasks gate and checks expected outcomes.

**Covers:** `Task obligation fixtures prove blocked and covered tasks`.

**Obligation cross-check:** The missing fixture uses a broad verification task that can skip the scenario-specific absent-field/default acceptance. The covered fixture names exact scenarios, commands/helpers, evidence paths, and PASS outputs.

**Rationale:** Deterministic fixtures prevent this hardening from being prose-only.

**Boundary cases:** The blocked fixture succeeds only when the gate reports an omission/tasks finding; the covered fixture must pass without `VERDICT: FAIL` or `blocked`.

**Proof plan:** Expected outputs are `PASS expected blocked fixture: task-obligation-missing-blocked` and `PASS expected passing fixture: task-obligation-covered-pass`.

## Risks / Trade-offs

- **Over-constraining shared evidence** → Allow shared artifacts but require distinct scenario rows and exact acceptance signals.
- **False-positive fixture success** → Require target omission/tasks/missing text for the blocked fixture.
- **Archive drift** → Rerun helper from archive and record active-path rewrite proof.

## Deterministic Proof Plan

- `python openspec/changes/archive/2026-05-21-harden-tasks-obligation-gate/evidence/check-tasks-obligation-fixtures.py template` → `PASS tasks template includes task obligation fields`.
- `python openspec/changes/archive/2026-05-21-harden-tasks-obligation-gate/evidence/check-tasks-obligation-fixtures.py config` → `PASS config includes tasks-stage obligation guidance`.
- `python openspec/changes/archive/2026-05-21-harden-tasks-obligation-gate/evidence/check-tasks-obligation-fixtures.py fixtures` → `PASS task-obligation fixtures encode missing and covered obligations`.
- `tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-missing-blocked` → `PASS expected blocked fixture: task-obligation-missing-blocked`.
- `tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-covered-pass` → `PASS expected passing fixture: task-obligation-covered-pass`.
- Proposal, design, and tasks gates pass; final `openspec validate --all --strict --json` and `git diff --check` pass after archive.
