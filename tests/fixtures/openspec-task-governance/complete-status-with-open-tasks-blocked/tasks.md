## Phase 1: Implementation

- [x] I1 add contradictory complete-status example
- [ ] I2 leave one implementation task open

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Complete status with open tasks is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh complete-status-with-open-tasks-blocked`; save output at `openspec/changes/complete-status-with-open-tasks-blocked/evidence/complete-status-blocked.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Complete status with open tasks is blocked` -> `V1` -> `openspec/changes/complete-status-with-open-tasks-blocked/evidence/complete-status-blocked.txt`
