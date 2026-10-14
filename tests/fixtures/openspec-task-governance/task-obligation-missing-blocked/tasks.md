## Phase 1: Implementation

- [x] I1 [serial] Add config fields

## Phase 2: Deterministic verification

- [x] V1 [depends:I1] Unit tests for config parsing with new fields; save output at `openspec/changes/task-obligation-missing-blocked/evidence/config-tests.txt`

## Human / Oracle Checkpoints

- [x] H1 [depends:V1] command=`tests/fixtures/openspec-task-governance/run-obligation-fixture-gate.sh task-obligation-missing-blocked`; input=`tasks.md`; asserts=`fixture should be blocked`; evidence=`openspec/changes/task-obligation-missing-blocked/evidence/task-gate.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Task obligations map scenarios to exact proof` -> `V1` -> `openspec/changes/task-obligation-missing-blocked/evidence/config-tests.txt`
- `Scenario: Task obligation fixtures prove blocked and covered tasks` -> `H1` -> `openspec/changes/task-obligation-missing-blocked/evidence/task-gate.txt`
