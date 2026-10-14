## Phase 1: Implementation

- [x] I1 add change-status fixture shape
- [ ] I2 leave one implementation item open

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Unchecked tasks keep change in progress" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-in-progress-pass`; save output at `openspec/changes/change-status-in-progress-pass/evidence/change-status-in-progress-pass.txt`

## Phase 3: Human/oracle checkpoints

- [ ] H1 Human/oracle checkpoint: command=`cat tasks.md`; input=`fixture change-status-in-progress-pass`; asserts=`remaining-work checkpoint stays replayable`; evidence=`openspec/changes/change-status-in-progress-pass/evidence/change-status-human-pass.txt`

## Change Status
- Status: in-progress
- Open tasks: I2, V1, H1
## Verification Coverage

- `Scenario: Unchecked tasks keep change in progress` -> `V1` -> `openspec/changes/change-status-in-progress-pass/evidence/change-status-in-progress-pass.txt`
- `Scenario: Human checkpoint remains replayable while work stays open` -> `H1` -> `openspec/changes/change-status-in-progress-pass/evidence/change-status-human-pass.txt`
