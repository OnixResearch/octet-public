## Why

Review metrics still show repeated `omission|spec|prompt` findings after the design-obligation hardening. The common failure is that a proposal or design promise names a required behavior, boundary, affected artifact, or verification expectation, but the delta spec omits an explicit requirement/scenario for it.

## What Changes

- **Delta-spec template**: add an explicit obligation cross-check prompt before review.
- **OpenSpec config**: require spec-stage reviewers/authors to map proposal/design promises into delta-spec requirements and scenarios.
- **Fixtures/evidence**: add one failing and one passing spec-governance fixture that prove omitted obligations are caught and explicit obligation coverage passes; saved evidence means new evidence under this change's evidence directory and its eventual archive copy, not edits to historical archives.

## Capabilities

### Modified Capabilities
- `openspec-stage-authoring`: spec-stage authoring prevents proposal/design obligations from disappearing before design/tasks.

## Impact

- **Files**: `openspec/templates/spec.md`, `openspec/config.yaml`, `openspec/specs/openspec-stage-authoring/spec.md`, `tests/fixtures/openspec-spec-governance/**`, archived evidence.
- **APIs**: none.
- **Dependencies**: none.
- **Testing**: run template/config/fixture helper checks, failing/passing proposal-gate fixtures, `openspec validate --all --strict --json`, and `git diff --check`; save only new evidence under this change/archive evidence path.
- **Non-goals**: no rewrite of the external gate extension, no broad cleanup of historical OpenSpec archives, and no updates to old archived evidence.
