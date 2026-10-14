<!-- Template: openspec/templates/tasks.md -->

## Phase 1: Implementation

- [x] I1 Add repo-local task authoring guidance or a copyable template that requires `I<n>`, `V<n>`, and `H<n>` task IDs plus a trailing `## Verification Coverage` section
- [x] I2 Update the repo's tasks-stage gate guidance or logic so `I<n>` tasks cannot also claim verification or saved evidence in the same checkbox
- [x] I3 Update the repo's tasks-stage gate guidance or logic so it reads delta-spec scenarios and requires one verification-coverage row per added or modified scenario
- [x] I4 Update the repo's tasks-stage gate guidance or logic so every `H<n>` task must supply `command=`, `input=`, `asserts=`, and `evidence=`

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Implementation and verification are split" with a negative fixture that contains `I<n>` plus saved-evidence text in one checkbox; assert the gate output contains `Task I<n> mixes implementation and verification/evidence claims`; save the blocking gate output at `openspec/changes/harden-openspec-task-gates/evidence/bundled-implementation-task.txt`
- [x] V2 Verify scenario "One scenario does not hide inside a bundled checkbox" with a fixture that maps two scenarios to one `V<n>` task and shows the offending task text; save the gate output at `openspec/changes/harden-openspec-task-gates/evidence/bundled-scenario-task.txt`
- [x] V3 Verify scenario "Reviewer can trace a scenario to proof" with a negative fixture that omits one coverage row for a delta-spec scenario; assert the gate output contains `Scenario "<heading>" has no verification coverage row`; save the blocking gate output at `openspec/changes/harden-openspec-task-gates/evidence/missing-coverage-row.txt`
- [x] V4 Verify scenario "Shared transcript still keeps separate scenario rows" with a passing fixture that maps two scenarios to separate rows pointing at one shared artifact; save the passing gate output at `openspec/changes/harden-openspec-task-gates/evidence/shared-evidence-pass.txt`
- [x] V5 Verify scenario "Aspen replay task is self-describing" with a negative fixture that omits one required `H<n>` field and a passing fixture that fills all four fields; assert the blocking output contains `Human/oracle task H<n> is missing command=, input=, asserts=, or evidence=`; save the gate outputs at `openspec/changes/harden-openspec-task-gates/evidence/human-checkpoint-missing-field.txt` and `openspec/changes/harden-openspec-task-gates/evidence/human-checkpoint-pass.txt`
- [x] V6 Verify scenario "Bundled implementation task is blocked" with a fixture-specific gate run that emits the exact blocking finding promised by this change; save the output at `openspec/changes/harden-openspec-task-gates/evidence/blocking-bundled-implementation.txt`
- [x] V7 Verify scenario "Missing scenario mapping is blocked" with a fixture-specific gate run that emits the exact blocking finding promised by this change; save the output at `openspec/changes/harden-openspec-task-gates/evidence/blocking-missing-scenario.txt`
- [x] V8 Verify scenario "Human checkpoint missing evidence path is blocked" with a fixture-specific gate run that emits the exact blocking finding promised by this change; save the output at `openspec/changes/harden-openspec-task-gates/evidence/blocking-missing-human-evidence.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`openspec_gate stage=proposal change=harden-openspec-task-gates`; input=`active change proposal after spec/design/tasks drafting`; asserts=`proposal stage stays aligned with the task-governance capability and does not widen into proposal/design hardening`; evidence=`openspec/changes/harden-openspec-task-gates/evidence/proposal-gate.txt`
- [x] H2 Human/oracle checkpoint: command=`openspec_gate stage=design change=harden-openspec-task-gates`; input=`active change design after decisions and proof plan are written`; asserts=`design stage owns the exact task-ID, coverage-row, human-field, and blocking-finding contracts`; evidence=`openspec/changes/harden-openspec-task-gates/evidence/design-gate.txt`
- [x] H3 Human/oracle checkpoint: command=`openspec_gate stage=tasks change=harden-openspec-task-gates`; input=`active change tasks plus verification-coverage map`; asserts=`tasks stage shows every added or modified scenario mapped to planned proof and no checkbox bundles implementation with proof capture`; evidence=`openspec/changes/harden-openspec-task-gates/evidence/tasks-gate.txt`
- [x] H4 Human/oracle checkpoint: command=`review_metrics action=summary source=openspec-gate stage=tasks last=40`; input=`post-implementation comparison window after hardened task gate lands`; asserts=`task-stage omission and incoherent findings are materially lower than the pre-change baseline cited in proposal/design`; evidence=`openspec/changes/harden-openspec-task-gates/evidence/post-change-review-metrics.txt`

Each saved `H<n>` evidence transcript must be self-describing by including the
exact command preamble or helper invocation that produced it.

## Verification Coverage

- `Scenario: Implementation and verification are split` -> `V1` -> `openspec/changes/harden-openspec-task-gates/evidence/bundled-implementation-task.txt`
- `Scenario: One scenario does not hide inside a bundled checkbox` -> `V2` -> `openspec/changes/harden-openspec-task-gates/evidence/bundled-scenario-task.txt`
- `Scenario: Reviewer can trace a scenario to proof` -> `V3` -> `openspec/changes/harden-openspec-task-gates/evidence/missing-coverage-row.txt`
- `Scenario: Shared transcript still keeps separate scenario rows` -> `V4` -> `openspec/changes/harden-openspec-task-gates/evidence/shared-evidence-pass.txt`
- `Scenario: Aspen replay task is self-describing` -> `V5` -> `openspec/changes/harden-openspec-task-gates/evidence/human-checkpoint-pass.txt`
- `Scenario: Bundled implementation task is blocked` -> `V6` -> `openspec/changes/harden-openspec-task-gates/evidence/blocking-bundled-implementation.txt`
- `Scenario: Missing scenario mapping is blocked` -> `V7` -> `openspec/changes/harden-openspec-task-gates/evidence/blocking-missing-scenario.txt`
- `Scenario: Human checkpoint missing evidence path is blocked` -> `V8` -> `openspec/changes/harden-openspec-task-gates/evidence/blocking-missing-human-evidence.txt`
