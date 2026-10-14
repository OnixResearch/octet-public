## Phase 1: Implementation

- [x] I1 [serial] Patch task template guidance with exact `V<n>` obligation prompts
- [x] I2 [serial] [depends:I1] Patch tasks-stage config guidance with exact acceptance-signal obligations
- [x] I3 [serial] [depends:I2] Add blocked and covered task-obligation fixtures plus runner checks

## Phase 2: Deterministic verification

- [x] V1 [depends:I3] Verify scenario "Task obligations map scenarios to exact proof" with command=`tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-covered-pass`; input=`tests/fixtures/openspec-task-governance/task-obligation-covered-pass/tasks.md`; expect=`PASS expected passing fixture: task-obligation-covered-pass`; evidence=`openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`
- [x] V2 [depends:I3] Verify scenario "Task obligation fixtures prove blocked and covered tasks" with command=`tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-covered-pass`; input=`tests/fixtures/openspec-task-governance/task-obligation-covered-pass`; expect=`PASS expected passing fixture: task-obligation-covered-pass`; evidence=`openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`

## Human / Oracle Checkpoints

- [x] H1 [depends:V1,V2] command=`tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-covered-pass`; input=`tests/fixtures/openspec-task-governance/task-obligation-covered-pass/tasks.md`; asserts=`covered fixture has exact scenario tasks and evidence mappings`; evidence=`openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Task obligations map scenarios to exact proof` -> `V1` -> `openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`
- `Scenario: Task obligation fixtures prove blocked and covered tasks` -> `V2` -> `openspec/changes/task-obligation-covered-pass/evidence/task-gate.txt`
