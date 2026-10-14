## Why

`review_metrics promotions min=3` shows design-stage omission findings are now
this repo's biggest repeated OpenSpec churn: the same missing-coverage review
notes appear 52 times. See the saved excerpt at
`openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/review-metrics-design-omissions.txt`.
The repeated misses are concrete: designs omit which scenario a decision
covers, omit negative controls or continued-warning boundaries, and name saved
artifacts without saying what exact output proves success.

Tigerstyle already ships `openspec/templates/design.md`, but the template only
asks for free-form decisions plus coarse proof-plan fields, and
`openspec/config.yaml` only tells authors to follow that template. Design-stage
review therefore keeps catching the same missing structure after authors have
already written the file.

## What Changes

- Tighten `openspec/templates/design.md` so each decision names exact
  scenario/requirement coverage and visible boundary cases.
- Tighten the design-stage rule in `openspec/config.yaml` so prompts require
  full scenario coverage, negative controls / continued-warning boundaries, and
  exact proof acceptance signals.
- Save deterministic extracts and a design-gate transcript proving the hardened
  contract is visible before implementation starts.

## Non-Goals

- No implementation of lint, runner, or consumer-facing product changes.
- No new proposal-stage or tasks-stage governance beyond preserving the existing
  proposal/delta-spec template rules already in `openspec/config.yaml`.
- No retroactive rewrite of archived design docs.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `openspec-stage-authoring`: design-stage authoring and gate guidance now
  front-load scenario coverage, boundary cases, and exact proof acceptance
  signals.

## Impact

- **Files / artifacts**: `openspec/templates/design.md`, `openspec/config.yaml`, `docs/contributor-guide.md` (existing alignment retained; no edit planned), `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/specs/openspec-stage-authoring/spec.md`, `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/design.md`, `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/tasks.md`, and saved proof under `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/`.
- **Ordering / dependencies**: this change bootstraps from the pre-hardening design template already in repo, then updates the delta spec and design-template contract, then updates the template and config rule, then saves deterministic extracts, then runs the design-stage gate transcript.
- **Verification / evidence**: save the scoping excerpt from `review_metrics`, one extract for the full decision-block label inventory plus template heading order, one extract for the proof-plan acceptance-signal field, one contributor-guide template-link extract, one config-rule extract that captures the preserved proposal/delta-spec rules plus the hardened design rule, and one `openspec_gate stage=design` transcript.
