# Harden OpenSpec Proposal And Design Templates

## Why

`review_metrics promotions last=100` shows proposal/design authoring is now the
highest repeated OpenSpec churn in this repo: proposal omission/prompt findings
appear 8 times, design omission/guidance 4 times, and design omission/prompt 3
times. Recent stage-gate retries kept asking for the same missing structure:
exact repo-relative artifact paths, ordering/dependency notes, and explicit
proof/evidence planning.

Tigerstyle already ships `openspec/templates/tasks.md`, but it has no repo-
local `openspec/templates/proposal.md` or `openspec/templates/design.md`.
Authors still invent those shapes change by change, so the same omissions recur
before implementation starts.

## What Changes

- Add copyable proposal and design templates under `openspec/templates/`.
- Make the proposal template ask for exact repo-relative files/artifacts,
  ordering/dependency notes, and planned verification/evidence surfaces.
- Make the design template ask for an ordered proof plan that binds each saved
  artifact to a repo-relative path, command/helper, prerequisites, and cleanup
  expectations.
- Update repo-local OpenSpec authoring guidance in `docs/contributor-guide.md`
  and `openspec/config.yaml` so maintainers see the same template contract
  before implementation.

## Non-Goals

- No generic upstream OpenSpec schema work outside this repo.
- No new task-governance changes beyond the existing `openspec/templates/tasks.md`
  contract.
- No retroactive rewrite of archived proposal/design files.

## Capabilities

### New Capabilities

- `openspec-stage-authoring`: tigerstyle ships copyable proposal/design
  templates and repo-local guidance that front-load exact artifact paths,
  sequencing, and evidence planning before implementation.

## Impact

- **Files / artifacts**: `openspec/templates/proposal.md`,
  `openspec/templates/design.md`, `docs/contributor-guide.md`,
  `openspec/config.yaml`, change-local specs/design/tasks, and saved proof under
  `openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/`.
- **Ordering / dependencies**: land the two templates first, then point
  contributor/config guidance at them, then save deterministic extracts proving
  the exact headings and prompt lines match the spec.
- **Verification / evidence**: save one exact extract for the proposal
  template, one for the design template, and one crosswalk extract showing both
  template paths appear in contributor/config guidance.
