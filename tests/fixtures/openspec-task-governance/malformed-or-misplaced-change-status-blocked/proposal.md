## Why

This fixture exists to prove the tasks-stage gate blocks a malformed or
misplaced `## Change Status` section.

## What Changes

- keep one governed verification task and one matching coverage row
- place `## Change Status` away from `## Verification Coverage`

## Non-Goals

- no missing status section
- no missing verification-coverage rows
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for malformed or
  misplaced change-status sections

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh malformed-or-misplaced-change-status-blocked`
