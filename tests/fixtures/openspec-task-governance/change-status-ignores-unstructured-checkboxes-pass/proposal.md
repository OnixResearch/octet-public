## Why

This fixture exists to prove ordinary markdown checkboxes do not affect the
open-task list in `## Change Status`.

## What Changes

- keep one unchecked governed `I2` and one unchecked governed `V1`
- add ordinary markdown checkboxes that do not start with `I<n>`, `V<n>`, or
  `H<n>`

## Non-Goals

- no malformed change-status fields
- no missing verification-coverage rows
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: passing fixture for ignoring
  unstructured checkboxes

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-ignores-unstructured-checkboxes-pass`
