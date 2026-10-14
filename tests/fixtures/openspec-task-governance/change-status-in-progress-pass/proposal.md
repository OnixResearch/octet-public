## Why

This fixture exists to prove the tasks-stage gate accepts an in-progress
`## Change Status` section whose open-task list matches unchecked governed task
IDs in file order.

## What Changes

- keep one unchecked `I2`, one unchecked `V1`, and one unchecked `H1`
- keep a compliant `## Change Status` section immediately before
  `## Verification Coverage`

## Non-Goals

- no malformed change-status fields
- no missing verification-coverage rows
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: passing fixture for in-progress
  change-status sections

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-in-progress-pass`
