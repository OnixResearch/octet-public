## Why

This fixture exists to prove the tasks-stage gate blocks a human/oracle
checkpoint that omits one required replay field.

## What Changes

- add one `H1` task with `command=`, `input=`, and `asserts=` but no
  `evidence=` field
- keep the fixture otherwise minimal so the missing field is the only intended
  blocker

## Non-Goals

- no mixed `I<n>` task markers
- no scenario-coverage omissions

## Capabilities

### New Capabilities
- `openspec-task-governance-fixture`: negative fixture for malformed
  human/oracle checkpoints

## Impact

- **Files**: fixture-only OpenSpec artifacts under `tests/fixtures/openspec-task-governance/`
- **APIs**: none
- **Dependencies**: none
- **Testing**: copy fixture into `openspec/changes/` and run `openspec_gate stage=tasks`
