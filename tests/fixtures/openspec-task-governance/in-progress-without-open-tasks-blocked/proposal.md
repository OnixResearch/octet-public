## Why

This fixture exists to prove the tasks-stage gate blocks `Status: in-progress`
when no governed task remains open.

## What Changes

- keep every governed task checked
- claim `Status: in-progress` with `Open tasks: none`

## Non-Goals

- no missing status section
- no malformed replay fields
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for in-progress status
  with no open tasks

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh in-progress-without-open-tasks-blocked`
