## Why

This fixture exists to prove the tasks-stage gate blocks `Status: complete`
when unchecked governed tasks remain.

## What Changes

- leave unchecked governed `I2` and `V1`
- claim `Status: complete` with `Open tasks: none`

## Non-Goals

- no missing status section
- no malformed replay fields
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for complete status with
  open tasks

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh complete-status-with-open-tasks-blocked`
