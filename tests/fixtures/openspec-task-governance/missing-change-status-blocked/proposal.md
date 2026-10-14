## Why

This fixture exists to prove the tasks-stage gate blocks active `tasks.md`
files that omit `## Change Status`.

## What Changes

- keep one governed verification task and one matching coverage row
- omit `## Change Status` entirely

## Non-Goals

- no malformed replay fields
- no missing verification-coverage rows
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for missing
  change-status sections

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh missing-change-status-blocked`
