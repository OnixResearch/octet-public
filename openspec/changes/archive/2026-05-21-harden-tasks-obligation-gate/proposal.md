## Why

Review metrics still show repeated `omission|tasks|auto-fix` findings after proposal, delta-spec, and design hardening. These failures occur when `tasks.md` leaves required behavior or artifacts implicit: broad verification tasks hide scenario-specific acceptance, required artifacts are missing, or a task can be satisfied by a weaker check than the delta spec requires.

## What Changes

- Harden `openspec/templates/tasks.md` with explicit task-obligation prompts for scenario-specific verification tasks, exact acceptance signals, required artifacts, and negative/continued-warning behavior.
- Update `openspec/config.yaml` tasks-stage guidance so task gate review blocks missing required scenario/task/evidence obligations.
- Update `openspec-task-governance` and add deterministic task-governance fixtures proving missing task obligations are blocked and explicit task/evidence mappings pass.

## Non-Goals

- Do not change runtime lint behavior, cargo-octet behavior, or object corpus fixtures.
- Do not mutate historical archived evidence outside this change's eventual archive.
- Do not solve human-oracle-specific task omissions; those remain a separate `omission|tasks|human` cluster.

## Capabilities

### Modified Capabilities

- `openspec-task-governance`: tasks guidance requires exact per-scenario verification tasks, artifacts, and acceptance signals for every delta-spec obligation.

## Impact

- **Files / artifacts**: `openspec/templates/tasks.md`, `openspec/config.yaml`, `openspec/specs/openspec-task-governance/spec.md`, `tests/fixtures/openspec-task-governance/task-obligation-*`, `tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh`, and this change archive/evidence.
- **Concrete scope**: only tasks-stage auto-fix omission prevention for `I<n>`/`V<n>`/coverage mappings; existing change-status and human checkpoint fixtures remain in place.
- **Downstream validation boundaries**: no Aspen, Valence object corpus, or UI snapshot regeneration; fixture proof stays under `tests/fixtures/openspec-task-governance/`.
- **Ordering / dependencies**: build on existing task-governance fixture infrastructure and the landed proposal/spec/design obligation hardening.
- **Verification / evidence**: static helper checks must print `PASS tasks template includes task obligation fields`, `PASS config includes tasks-stage obligation guidance`, and `PASS task-obligation fixtures encode missing and covered obligations`; fixture runner must print `PASS expected blocked fixture: task-obligation-missing-blocked` and `PASS expected passing fixture: task-obligation-covered-pass`; proposal/design/tasks gates, final `openspec validate --all --strict --json`, and `git diff --check` must pass.
