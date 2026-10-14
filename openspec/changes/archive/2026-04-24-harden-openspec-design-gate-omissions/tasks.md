## Phase 1: Implementation

- [x] I1 Update `openspec/templates/design.md` so each copied decision block keeps the exact `**Covers:**` and `**Boundary cases:**` prompt lines promised by `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/specs/openspec-stage-authoring/spec.md`.
- [x] I2 Update `openspec/config.yaml` so the design-stage rule names `openspec/templates/design.md`, requires coverage for every added or modified scenario heading or zero-scenario requirement, and requires named boundary cases plus exact proof acceptance signals.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Maintainer maps design decisions to exact delta-spec coverage" with a Python extract over `openspec/templates/design.md` that proves the exact `## ` heading order plus the full decision-block label inventory (`**Choice:**`, `**Covers:**`, `**Rationale:**`, `**Alternative considered:**`, `**Implementation:**`, `**Boundary cases:**`); save output at `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-coverage-fields.txt`.
- [x] V2 Verify scenario "Maintainer names negative controls and proof acceptance signals" with a Python extract over `openspec/templates/design.md`; save output at `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-proof-fields.txt`.
- [x] V3 Verify scenario "Contributor guide points authors at the repo-local stage templates" with a Python extract over `docs/contributor-guide.md`; save output at `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/contributor-guide-template-links.txt`.
- [x] V4 Verify scenarios "Proposal-stage and delta-spec guidance keep pointing at repo-local templates" and "Design-stage guidance names coverage and boundary expectations" with a Python extract over `openspec/config.yaml`; save output at `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-stage-guidance.txt`.
- [x] V5 Verify the proposal scoping excerpt from `review_metrics promotions min=3`; save output at `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/review-metrics-design-omissions.txt`.

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`openspec_gate stage=design change=harden-openspec-design-gate-omissions`; input=`final proposal.md plus openspec-stage-authoring delta spec and design.md`; asserts=`design-stage review accepts the hardened coverage, boundary-case, and acceptance-signal contract without repeated design-omission findings`; evidence=`openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-gate.txt`

## Change Status
- Status: complete
- Open tasks: none
Use `Open tasks: none` only when every governed `I<n>`, `V<n>`, and `H<n>` task checkbox is checked.
## Verification Coverage

- `Scenario: Maintainer maps design decisions to exact delta-spec coverage` -> `V1` -> `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-coverage-fields.txt`
- `Scenario: Maintainer names negative controls and proof acceptance signals` -> `V2` -> `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-proof-fields.txt`
- `Scenario: Contributor guide points authors at the repo-local stage templates` -> `V3` -> `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/contributor-guide-template-links.txt`
- `Scenario: Proposal-stage and delta-spec guidance keep pointing at repo-local templates` -> `V4` -> `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-stage-guidance.txt`
- `Scenario: Design-stage guidance names coverage and boundary expectations` -> `V4` -> `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-stage-guidance.txt`
