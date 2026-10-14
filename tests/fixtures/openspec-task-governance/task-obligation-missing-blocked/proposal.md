## Why

Task obligations can be omitted when a broad verification task hides scenario-specific behavior.

## What Changes

- Add task verification obligations.

## Non-Goals

- Do not change runtime lint behavior.

## Capabilities

### Modified Capabilities

- `openspec-task-governance`: task verification obligations.

## Impact

- **Files / artifacts**: `openspec/templates/tasks.md`, `openspec/config.yaml`, and fixture tasks.
- **Concrete scope**: task-obligation fixtures only.
- **Downstream validation boundaries**: no Aspen or object corpus evidence changes.
- **Ordering / dependencies**: fixtures depend on template/config guidance.
- **Verification / evidence**: fixture runner must print expected blocked/pass outputs.
