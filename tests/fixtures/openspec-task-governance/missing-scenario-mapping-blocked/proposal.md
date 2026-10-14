## Why

This fixture exists to prove the tasks-stage gate emits the exact blocking
finding when a delta-spec scenario has no matching `## Verification Coverage`
row.

## What Changes

- keep two spec scenarios so one row can be present while the target row is
  missing
- omit the coverage row for the target scenario only

## Non-Goals

- no mixed `I<n>` task markers
- no malformed `H<n>` checkpoint fields

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for missing scenario
  mapping

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
