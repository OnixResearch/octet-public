## Why

Repeated spec omission checks need a passing fixture where proposal promises are explicitly encoded in the delta spec.

## What Changes

- Add a spec authoring prompt.
- The implementation must preserve the non-goal boundary that historical archived examples are not normalized.
- Verification must save an artifact showing that archive-local references are left untouched.

## Capabilities

### Modified Capabilities
- `openspec-stage-authoring`: adds spec-stage prompts.

## Impact

- **Files**: `openspec/templates/spec.md`.
- **APIs**: none.
- **Dependencies**: none.
- **Testing**: a saved evidence artifact must show archive-local references remain untouched.
- **Non-goals**: historical archived examples are not normalized.
