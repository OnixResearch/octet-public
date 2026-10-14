## Why

This fixture exists to prove the tasks-stage gate emits the exact blocking
finding for a bundled implementation task.

## What Changes

- Add one bad `I<n>` task that updates code and saves an artifact in the same
  checkbox
- Keep the rest of the fixture minimal so the mixed-claim finding is the only
  intended blocker

## Non-Goals

- no scenario-coverage-row failures
- no malformed `H<n>` checkpoint fields

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for bundled
  implementation-task blocking

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
