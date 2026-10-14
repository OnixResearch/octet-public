## Why

This fixture exists to prove the tasks-stage gate accepts two separate scenario
rows that point at one shared artifact.

## What Changes

- add two spec scenarios
- keep one `V<n>` task per scenario
- point both coverage rows at one shared artifact path

## Non-Goals

- no mixed `I<n>` task markers
- no missing coverage rows
- no malformed `H<n>` checkpoint fields

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: passing fixture for shared evidence with
  separate scenario rows

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
