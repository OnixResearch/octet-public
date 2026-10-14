## Why

This fixture exists to prove the tasks-stage gate blocks an in-progress
open-task list that does not match unchecked governed task IDs.

## What Changes

- leave unchecked governed `I2` and `V1`
- keep `Status: in-progress` but omit `V1` from `Open tasks`

## Non-Goals

- no missing status section
- no malformed replay fields
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for open-task-list
  mismatch

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh open-task-list-mismatch-blocked`
