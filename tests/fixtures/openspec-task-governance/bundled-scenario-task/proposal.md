## Why

This fixture exists to prove the tasks-stage gate blocks one `V<n>` task that
claims two scenarios at once.

## What Changes

- add two spec scenarios
- use one bundled `V1` task that names both scenarios
- keep coverage rows present so the bundled V-task is the only intended blocker

## Non-Goals

- no mixed `I<n>` task markers
- no missing coverage rows
- no malformed `H<n>` checkpoint fields

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for bundled scenario
  verification tasks

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
