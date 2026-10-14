# Harden OpenSpec Delta-Spec Template

## Why

Recent tigerstyle OpenSpec work hardened proposal, design, and task authoring,
but change-local delta specs still start from scratch. Review metrics still show
repeated proposal/spec omissions: missing explicit `The system MUST ...`
requirement sentences, missing `#### Scenario:` blocks, path-only artifact
claims, and confusion between active-change delta specs and main specs under
`openspec/specs/`.

Tigerstyle has no repo-local `openspec/templates/spec.md`, no contributor-guide
link for delta-spec authors, and no repo-local proposal-stage guidance that
points authors at one copyable delta-spec shape before implementation.

## What Changes

- Add a copyable delta-spec template at `openspec/templates/spec.md`.
- Make that template spell out the exact delta headings, normative requirement
  prompt, scenario prompt, and repo-specific guardrails for active-change specs.
- Update repo-local OpenSpec guidance so maintainers can find the delta-spec
  template from `docs/contributor-guide.md` and `openspec/config.yaml`.
- Save deterministic proof that the template shape and guidance lines match the
  staged authoring contract.

## Non-Goals

- No generic upstream OpenSpec schema work.
- No rewrite of archived delta specs.
- No contributor-guide or config expansion for `openspec/templates/tasks.md`; task-template governance stays under `openspec-task-governance`.
- No new implementation outside the repo-local template and authoring guidance.

## Capabilities

### New Capabilities

- `openspec-delta-spec-authoring`: tigerstyle ships a copyable change-local
  delta-spec template with repo-specific guardrails and guidance entry points.

### Modified Capabilities

- `openspec-stage-authoring`: proposal-stage guidance also points authors at the
  repo-local delta-spec template before design/tasks work begins.

## Impact

- **Files / artifacts**: `openspec/templates/spec.md`, `docs/contributor-guide.md`,
  `openspec/config.yaml`, change-local OpenSpec artifacts under
  `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/`, and saved proof under
  `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/`.
- **Ordering / dependencies**: define the delta-spec template contract first,
  then wire contributor/config guidance to that path, then save deterministic
  extracts proving the exact template and guidance lines.
- **Verification / evidence**: save one deterministic extract for the template
  headings/prompts/guardrails, one crosswalk extract for contributor/config
  guidance, and one proposal-stage gate transcript for the finished OpenSpec
  artifacts.
