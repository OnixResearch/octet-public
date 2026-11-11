## Context

Tigerstyle already hardened proposal, delta-spec, and task authoring shape, but
`review_metrics promotions min=3` still shows one design-stage omission class
far above the rest: 52 repeated findings where designs fail to say which
scenario a decision covers, fail to name negative controls or continued-warning
boundaries, or fail to state what exact output makes a saved artifact pass.

The current `openspec/templates/design.md` preserves section order and a coarse
proof-plan skeleton, but it does not force decision-to-scenario traceability or
exact acceptance-signal planning. `openspec/config.yaml` currently tells authors
to follow the design template, yet it does not say that every added/modified
scenario heading or zero-scenario requirement must be covered explicitly.

## Goals / Non-Goals

**Goals:**
- make each design decision name the exact scenario or requirement headings it covers
- make boundary cases and continued-warning expectations explicit before review
- make saved proof artifacts state an exact expected output or acceptance signal
- harden design-stage guidance in `openspec/config.yaml` so the gate prompt asks for the same contract the template encodes

**Non-Goals:**
- changing proposal-stage or tasks-stage governance in this change
- retrofitting old archived designs to the new prompt shape
- promising immediate review-metric improvement before future changes use the new contract

## Decisions

### 1. Decision blocks carry explicit coverage and boundary prompts

**Choice:** extend each design-template decision block with exact `**Covers:**`
and `**Boundary cases:**` prompts.

**Covers:** `Scenario: Maintainer maps design decisions to exact delta-spec coverage`

**Rationale:** the highest-repeat design omissions are missing scenario
coverage and missing non-trigger boundaries. Putting those prompts inside every
decision block makes the reviewer-visible gap obvious before implementation.

**Alternative considered:** keep free-form decision prose and rely on reviewers
to ask for the missing mapping. Rejected because that is the churn this change
is trying to remove.

**Implementation:** update `openspec/templates/design.md` and the
`openspec-stage-authoring` delta spec so every copied decision block includes:

- `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
- `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`

When a delta requirement has one or more scenarios, `**Covers:**` names the
exact scenario headings. A bare requirement heading is only valid for a
zero-scenario requirement.

**Boundary cases:** zero-scenario requirements may name the requirement heading
instead of a scenario heading, but multi-scenario requirements may not collapse
coverage back to the parent requirement heading.

### 2. Deterministic proof items carry exact acceptance signals

**Choice:** add one exact proof-plan prompt line,
`- exact expected output / acceptance signal:`, alongside the existing path,
command, ordering, and cleanup fields.

**Covers:** `Scenario: Maintainer names negative controls and proof acceptance signals`

**Rationale:** repeated design findings show that path-only artifact planning is
not enough; review needs the exact line set, schema, or pass condition that the
artifact must prove.

**Alternative considered:** rely on the existing `artifact path` and
`cleanup / postcondition` lines. Rejected because neither field says what exact
output makes the artifact acceptable.

**Implementation:** update `openspec/templates/design.md` and the delta spec so
`## Deterministic Proof Plan` keeps the new prompt line in addition to the
existing required labels. Use one checked-in helper,
`python3 openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/update-proof-artifacts.py`,
to regenerate the deterministic extracts exactly as review expects.

**Boundary cases:** path-only artifact planning is still insufficient even when
an evidence path is stable; the replay contract must include the exact helper
path plus the exact expected output lines.

### 3. Design-stage guidance names full coverage obligations

**Choice:** harden the `openspec/config.yaml` design rule so it names
`openspec/templates/design.md`, requires every added/modified scenario heading
or zero-scenario requirement to appear in design coverage, and requires named
boundary cases plus exact acceptance signals when saved proof artifacts matter.

**Covers:** `Scenario: Contributor guide points authors at the repo-local stage templates`; `Scenario: Proposal-stage and delta-spec guidance keep pointing at repo-local templates`; `Scenario: Design-stage guidance names coverage and boundary expectations`

**Rationale:** template changes alone do not help if stage guidance still asks
for only a generic design. The repo-local gate prompt must ask for the same
coverage and artifact contract.

**Alternative considered:** link the template path only. Rejected because the
existing design rule already does that and still leaves repeated omissions.

**Implementation:** replace the current minimal design rule text in
`openspec/config.yaml` with one rule that keeps the template path and spells out
the new coverage and boundary expectations. The exact-five-`##`-headings rule
applies to the shipped template file `openspec/templates/design.md`, not to
every authored change-local `design.md` copied from it. `docs/contributor-guide.md`
keeps its existing template links unchanged in this rollout; alignment is
retained, not re-authored, and saved proof will verify those unchanged exact
link lines through the checked-in helper.

**Boundary cases:** this change preserves the existing proposal and delta-spec
stage guidance lines; it only strengthens design-stage guidance plus the
unchanged contributor-guide template-link proof.

## Deterministic Proof Plan

- scenario or requirement:
  - `Scenario: Maintainer maps design decisions to exact delta-spec coverage`
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-coverage-fields.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/update-proof-artifacts.py`
- exact expected output / acceptance signal:
  - the saved extract is the complete second-level-heading inventory plus full decision-block label inventory for `openspec/templates/design.md`, and it prints exactly these lines, in order, with no additional `## ` headings omitted or allowed:
    - `## Context`
    - `## Goals / Non-Goals`
    - `## Decisions`
    - `## Deterministic Proof Plan`
    - `## Risks / Trade-offs`
    - `**Choice:** <what was decided>`
    - `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
    - `**Rationale:** <why this choice wins>`
    - `**Alternative considered:** <what was rejected and why>`
    - `**Implementation:** <how this will be carried out>`
    - `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`
- ordering / prerequisites:
  - update `openspec/templates/design.md` first so the extract matches final text
- cleanup / postcondition:
  - no temp files remain outside the saved artifact

- scenario or requirement:
  - `Scenario: Maintainer names negative controls and proof acceptance signals`
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-template-proof-fields.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/update-proof-artifacts.py`
- exact expected output / acceptance signal:
  - the saved extract prints these exact lines, in order:
    - `## Deterministic Proof Plan`
    - `- scenario or requirement:`
    - `- artifact path:`
    - `- command / helper:`
    - `- exact expected output / acceptance signal:`
    - `- ordering / prerequisites:`
    - `- cleanup / postcondition:`
- ordering / prerequisites:
  - update the proof-plan prompt block before saving the extract
- cleanup / postcondition:
  - the artifact reflects the final template text only

- scenario or requirement:
  - proposal scoping excerpt from `review_metrics promotions min=3`
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/review-metrics-design-omissions.txt`
- command / helper:
  - `review_metrics action=promotions min=3`
- exact expected output / acceptance signal:
  - the saved extract prints these exact leading lines, in order:
    - `$ review_metrics promotions min=3`
    - `- count=52 [class=omission] [scope=design] [route=prompt]`
    - `  task: Strengthen OpenSpec design gate/template to block repeated omission findings.`
- ordering / prerequisites:
  - capture the scoping excerpt before finalizing proposal/design wording so the motivating repetition count is reviewable
- cleanup / postcondition:
  - the artifact remains a stable saved excerpt of the motivating design-omission cluster

- scenario or requirement:
  - `Scenario: Contributor guide points authors at the repo-local stage templates`
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/contributor-guide-template-links.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/update-proof-artifacts.py`
- exact expected output / acceptance signal:
  - the saved extract prints these exact standalone markdown-link lines, in order:
    - `[OpenSpec proposal template](../openspec/templates/proposal.md)`
    - `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
    - `[OpenSpec design template](../openspec/templates/design.md)`
- ordering / prerequisites:
  - none; `docs/contributor-guide.md` already carries the canonical links this change retains
- cleanup / postcondition:
  - the artifact contains only the three exact markdown-link lines needed for review

- scenario or requirement:
  - `Scenario: Proposal-stage and delta-spec guidance keep pointing at repo-local templates`
  - `Scenario: Design-stage guidance names coverage and boundary expectations`
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-stage-guidance.txt`
- command / helper:
  - `python3 openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/update-proof-artifacts.py`
- exact expected output / acceptance signal:
  - the saved extract prints these exact `openspec/config.yaml` rule lines, in order:
    - `    - Follow \`openspec/templates/proposal.md\`; name exact repo-relative files/artifacts, ordering/dependencies, and planned saved evidence or helper/gate runs`
    - `    - Follow \`openspec/templates/spec.md\` for delta-spec authoring; keep change-local specs in delta form, start each added or modified requirement with \`The system MUST ...\`, and give every added or modified requirement at least one \`#### Scenario:\` block`
    - `    - Follow \`openspec/templates/design.md\`; every added or modified scenario heading or zero-scenario requirement must appear in a decision's \`**Covers:**\` field; name boundary cases for non-trigger or continued-warning conditions; every saved proof artifact must state its exact expected output or acceptance signal`
- ordering / prerequisites:
  - update `openspec/config.yaml` after the template wording is final so the saved extract matches the final proposal/spec/design rule trio
- cleanup / postcondition:
  - the artifact contains only the three exact config-rule lines needed for review

- scenario or requirement:
  - human/oracle checkpoint for the full change
- artifact path:
  - `openspec/changes/archive/2026-04-24-harden-openspec-design-gate-omissions/evidence/design-gate.txt`
- command / helper:
  - `openspec_gate stage=design change=harden-openspec-design-gate-omissions`
- exact expected output / acceptance signal:
  - the transcript reports an overall passing design-stage gate result for this change and does not report any design-stage blocker findings
- ordering / prerequisites:
  - run after the final proposal, delta spec, design, and tasks are in place
- cleanup / postcondition:
  - the saved transcript references the active change path and needs no post-run cleanup

## Risks / Trade-offs

- more template ceremony -> keep the new prompts to the exact repeated omission classes instead of adding broad prose sections
- over-constraining simple changes -> let authors list one requirement heading in `**Covers:**` when a change has no scenario split
- drift between template and config guidance -> verify both with saved extracts and a design-stage gate transcript
