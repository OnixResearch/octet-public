## Phase 1: Implementation

- [x] I1 Add `openspec/templates/spec.md` with the exact heading inventory and prompt lines promised by `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/specs/openspec-stage-authoring/spec.md`.
- [x] I2 Add the repo-specific guardrail comments to `openspec/templates/spec.md` exactly as promised by `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/specs/openspec-stage-authoring/spec.md`.
- [x] I3 Update `docs/contributor-guide.md` and `openspec/config.yaml` so proposal-stage and contributor guidance point at `openspec/templates/spec.md` alongside the existing proposal/design template paths.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Maintainer drafts a delta spec without inventing headings" with a Python extract over `openspec/templates/spec.md`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/spec-template-fields.txt`.
- [x] V2 Verify scenario "Maintainer sees repo-specific delta-spec guardrails in the template" with a Python extract over `openspec/templates/spec.md`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/spec-template-fields.txt`.
- [x] V3 Verify scenario "Contributor guide exposes the delta-spec template" with a Python extract over `docs/contributor-guide.md`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/openspec-stage-authoring-spec-crosswalk.txt`.
- [x] V4 Verify scenario "Proposal-stage guidance names the delta-spec template" with a Python extract over `openspec/config.yaml` plus `docs/contributor-guide.md`; save output at `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/openspec-stage-authoring-spec-crosswalk.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`openspec_gate stage=proposal change=harden-openspec-delta-spec-template`; input=`final proposal.md plus openspec-stage-authoring delta spec`; asserts=`proposal-stage review accepts the delta-spec template scope and guidance contract without missing spec-authoring structure findings`; evidence=`openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/proposal-gate.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Maintainer drafts a delta spec without inventing headings` -> `V1` -> `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/spec-template-fields.txt`
- `Scenario: Maintainer sees repo-specific delta-spec guardrails in the template` -> `V2` -> `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/spec-template-fields.txt`
- `Scenario: Contributor guide exposes the delta-spec template` -> `V3` -> `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/openspec-stage-authoring-spec-crosswalk.txt`
- `Scenario: Proposal-stage guidance names the delta-spec template` -> `V4` -> `openspec/changes/archive/2026-04-18-harden-openspec-delta-spec-template/evidence/openspec-stage-authoring-spec-crosswalk.txt`
