## Phase 1: Implementation

- [x] I1 add open-task-list mismatch example
- [ ] I2 leave one implementation task open

## Phase 2: Deterministic verification

- [ ] V1 Verify scenario "Incorrect open-task list is blocked" with `bash tests/fixtures/openspec-task-governance/run-fixture-gate.sh open-task-list-mismatch-blocked`; save output at `openspec/changes/open-task-list-mismatch-blocked/evidence/open-task-list-mismatch.txt`

## Change Status
- Status: in-progress
- Open tasks: I2
## Verification Coverage

- `Scenario: Incorrect open-task list is blocked` -> `V1` -> `openspec/changes/open-task-list-mismatch-blocked/evidence/open-task-list-mismatch.txt`
