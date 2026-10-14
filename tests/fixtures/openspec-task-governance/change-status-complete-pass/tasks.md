## Phase 1: Implementation

- [x] I1 finish complete-status fixture

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Complete status is only valid when all tasks are checked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-complete-pass`; save output at `openspec/changes/change-status-complete-pass/evidence/change-status-complete-pass.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`cat tasks.md`; input=`fixture change-status-complete-pass`; asserts=`complete-status checkpoint stays replayable`; evidence=`openspec/changes/change-status-complete-pass/evidence/change-status-complete-human-pass.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Complete status is only valid when all tasks are checked` -> `V1` -> `openspec/changes/change-status-complete-pass/evidence/change-status-complete-pass.txt`
