## Why

This fixture exists to prove the tasks-stage gate accepts a human/oracle
checkpoint that includes all four replay fields.

## What Changes

- add one `H1` task with `command=`, `input=`, `asserts=`, and `evidence=`
- keep the fixture otherwise minimal so the checkpoint shape is the only thing
  under review

## Non-Goals

- no mixed `I<n>` task markers
- no scenario-coverage omissions

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: passing fixture for well-formed
  human/oracle checkpoints

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
