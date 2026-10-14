## Phase 1: Implementation

- [x] I0 [serial] Update delta specs under `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/specs/` to capture promoted omission-prevention behavior for stage-authoring and task-governance. ✅ 1m (completed: 2026-05-18T17:44:00Z)
- [x] I1 [serial] Update `openspec/templates/proposal.md`, `openspec/templates/spec.md`, and `openspec/templates/design.md` with omission-prevention prompts for the promoted proposal/spec/design omission classes. ✅ 4m (completed: 2026-05-18T17:32:05Z)
- [x] I2 [serial] Update `openspec/templates/tasks.md` with omission-prevention prompts for split implementation/proof tasks, one-row-per-scenario coverage, and exact `H<n>` replay fields. ✅ 3m (completed: 2026-05-18T17:36:03Z)
- [x] I3 [serial] Update `openspec/config.yaml` stage rules so proposal/spec/design/tasks prompts name the same omission-prevention obligations. ✅ 2m (completed: 2026-05-18T17:37:18Z)
- [x] I4 [serial] Add deterministic evidence helper `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`. ✅ 3m (completed: 2026-05-18T17:38:56Z)

## Phase 2: Deterministic verification

- [x] V1 [depends:I1] Verify scenario "Maintainer sees repeated proposal omission checks before drafting" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/proposal-template-omission-prompts.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V2 [depends:I1] Verify scenario "Maintainer sees repeated delta-spec omission checks before review" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/spec-template-omission-prompts.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V3 [depends:I1] Verify scenario "Maintainer sees repeated design omission checks before implementation" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/design-template-omission-prompts.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V4 [depends:I3] Verify scenario "Config guidance includes promoted omission-prevention checks" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V5 [depends:I2] Verify scenario "Maintainer sees repeated task omission checks before gate review" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V6 [depends:I2] Verify scenario "Human checkpoint guidance blocks implicit oracle inputs" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`. ✅ 1m (completed: 2026-05-18T17:40:05Z)
- [x] V7 [depends:I0,I1,I2,I3,I4,V1,V2,V3,V4,V5,V6,V8,V9] Verify active change validity with `openspec validate roi-01-openspec-omission-gate-rails --strict --json`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/openspec-validate.txt`. ✅ 1m (completed: 2026-05-18T17:56:03Z)
- [x] V8 [depends:I3] Verify scenario "Config task guidance includes promoted task omission checks" with `python3 openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/update-omission-artifacts.py`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`. ✅ 1m (completed: 2026-05-18T17:44:00Z)
- [x] V9 [depends:I0] Verify repeated omission source metrics with `review_metrics_promotions --cwd /path/to/git/octet --min 2 --limit 5`; save output at `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/review-metrics-omissions.txt`. ✅ 1m (completed: 2026-05-18T17:44:00Z)

## Phase 3: Human/oracle checkpoints

- [x] H1 [depends:V7,V8,V9] Human/oracle checkpoint: command=`openspec_gate stage=proposal change=roi-01-openspec-omission-gate-rails`; input=`proposal.md plus openspec-stage-authoring and openspec-task-governance delta specs`; asserts=`proposal-stage review accepts the omission-prevention scope and evidence plan`; evidence=`openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/proposal-gate.txt` ✅ 1m (completed: 2026-05-18T17:53:21Z)
- [x] H2 [depends:V7,V8,V9,H1] Human/oracle checkpoint: command=`openspec_gate stage=design change=roi-01-openspec-omission-gate-rails`; input=`final design.md with scenario coverage and proof plan`; asserts=`design-stage review accepts the scenario coverage, boundary cases, and proof acceptance signals`; evidence=`openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/design-gate.txt` ✅ 1m (completed: 2026-05-18T17:53:56Z)
- [x] H3 [depends:V7,V8,V9,H1,H2] Human/oracle checkpoint: command=`openspec_gate stage=tasks change=roi-01-openspec-omission-gate-rails`; input=`tasks.md and delta specs`; asserts=`tasks-stage review accepts one task per scenario and exact H-task replay fields`; evidence=`openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-gate.txt` ✅ 1m (completed: 2026-05-18T17:56:03Z)

## Change Status
- Status: complete
- Open tasks: none
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Maintainer sees repeated proposal omission checks before drafting` -> `V1` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/proposal-template-omission-prompts.txt`
- `Scenario: Maintainer sees repeated delta-spec omission checks before review` -> `V2` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/spec-template-omission-prompts.txt`
- `Scenario: Maintainer sees repeated design omission checks before implementation` -> `V3` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/design-template-omission-prompts.txt`
- `Scenario: Config guidance includes promoted omission-prevention checks` -> `V4` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`
- `Scenario: Maintainer sees repeated task omission checks before gate review` -> `V5` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`
- `Scenario: Human checkpoint guidance blocks implicit oracle inputs` -> `V6` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/tasks-template-omission-prompts.txt`
- `Scenario: Config task guidance includes promoted task omission checks` -> `V8` -> `openspec/changes/archive/2026-05-18-roi-01-openspec-omission-gate-rails/evidence/config-omission-guidance.txt`
