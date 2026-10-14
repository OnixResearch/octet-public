## Why

This fixture exists to prove the tasks-stage gate accepts `Status: complete`
with `Open tasks: none` when every governed task checkbox is checked.

## What Changes

- keep one checked `I1`, one checked `V1`, and one checked `H1`
- keep a compliant complete-status section immediately before
  `## Verification Coverage`

## Non-Goals

- no malformed change-status fields
- no missing verification-coverage rows
- no mixed implementation/proof task text

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: passing fixture for complete
  change-status sections

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `tests/fixtures/openspec-task-governance/run-fixture-gate.sh change-status-complete-pass`
