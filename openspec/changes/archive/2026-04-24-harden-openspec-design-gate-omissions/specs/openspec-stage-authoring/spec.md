## MODIFIED Requirements

### Requirement: Design Template Captures Ordered Proof Planning

The system MUST ship a copyable OpenSpec design template at
`openspec/templates/design.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings.

This exact-five-`##`-headings constraint applies to the shipped template file
`openspec/templates/design.md`, not to authored change-local `design.md` files
copied from it:

- `## Context`
- `## Goals / Non-Goals`
- `## Decisions`
- `## Deterministic Proof Plan`
- `## Risks / Trade-offs`

Inside `## Decisions`, each copied decision block MUST keep these exact prompt
labels:

- `**Choice:** <what was decided>`
- `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
- `**Rationale:** <why this choice wins>`
- `**Alternative considered:** <what was rejected and why>`
- `**Implementation:** <how this will be carried out>`
- `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`

When a delta requirement has one or more named scenarios, `**Covers:**` MUST
list the exact scenario headings, not only the requirement heading. A bare
requirement heading is only valid for a zero-scenario requirement.

Inside `## Deterministic Proof Plan`, the template MUST keep these exact prompt
labels:

- `- scenario or requirement:`
- `- artifact path:`
- `- command / helper:`
- `- exact expected output / acceptance signal:`
- `- ordering / prerequisites:`
- `- cleanup / postcondition:`

#### Scenario: Maintainer maps design decisions to exact delta-spec coverage

- GIVEN a maintainer copies `openspec/templates/design.md`
- WHEN they draft a design for a change with multiple scenario headings or a zero-scenario requirement
- THEN `openspec/templates/design.md` still shows the exact five `## ` headings in order plus the full decision-block label inventory:
  - `**Choice:** <what was decided>`
  - `**Covers:** <exact delta-spec scenario headings or requirement headings this decision satisfies>`
  - `**Rationale:** <why this choice wins>`
  - `**Alternative considered:** <what was rejected and why>`
  - `**Implementation:** <how this will be carried out>`
  - `**Boundary cases:** <negative controls, continued-warning cases, or intentional gaps reviewers must see>`
- AND they can name the exact scenario headings or requirement heading each decision satisfies
- AND a reviewer can tell when a normative case still lacks design coverage

#### Scenario: Maintainer names negative controls and proof acceptance signals

- GIVEN a change contract depends on non-trigger boundaries or continued-warning cases
- WHEN the maintainer fills the decision block and deterministic proof plan
- THEN the design names those boundary cases explicitly
- AND each planned proof artifact names the exact output or acceptance signal it must prove

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST point proposal-stage, delta-spec-stage, and design-stage
authors at the repo-local OpenSpec template files before implementation.

`docs/contributor-guide.md` MUST contain these exact standalone markdown-link
lines:

- `[OpenSpec proposal template](../openspec/templates/proposal.md)`
- `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
- `[OpenSpec design template](../openspec/templates/design.md)`

`openspec/config.yaml` MUST continue to contain one proposal rule that names
`openspec/templates/proposal.md` and one proposal rule that names
`openspec/templates/spec.md` for delta-spec authoring.

`openspec/config.yaml` MUST also contain one design rule that names
`openspec/templates/design.md`, requires every added or modified scenario
heading or zero-scenario requirement to appear in design coverage, and requires
named boundary cases plus exact expected output or acceptance signals when saved
proof artifacts matter.

#### Scenario: Contributor guide points authors at the repo-local stage templates

- GIVEN repo-local OpenSpec guidance reads `docs/contributor-guide.md`
- WHEN a maintainer looks up the stage templates before implementation
- THEN `docs/contributor-guide.md` still contains these exact standalone markdown-link lines:
  - `[OpenSpec proposal template](../openspec/templates/proposal.md)`
  - `[OpenSpec delta-spec template](../openspec/templates/spec.md)`
  - `[OpenSpec design template](../openspec/templates/design.md)`

#### Scenario: Proposal-stage and delta-spec guidance keep pointing at repo-local templates

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal-stage prompts are prepared for a new change
- THEN one proposal rule names `openspec/templates/proposal.md`
- AND one proposal rule names `openspec/templates/spec.md` for delta-spec authoring

#### Scenario: Design-stage guidance names coverage and boundary expectations

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN design-stage prompts are prepared for a new change
- THEN one design rule names `openspec/templates/design.md`
- AND it tells authors to cover every added or modified scenario heading or zero-scenario requirement
- AND it tells authors to name boundary cases and exact proof acceptance signals when they matter
