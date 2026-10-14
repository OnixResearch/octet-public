## ADDED Requirements

### Requirement: Proposal Template Captures Scope, Artifacts, And Ordering

The system MUST ship a copyable OpenSpec proposal template at
`openspec/templates/proposal.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## Why`
- `## What Changes`
- `## Non-Goals`
- `## Capabilities`
- `## Impact`

That template MUST also keep these exact prompt lines:

- `### New Capabilities`
- `### Modified Capabilities`
- `- **Files / artifacts**: <exact repo-relative paths touched by this change>`
- `- **Ordering / dependencies**: <what must land first or stay in lockstep>`
- `- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>`

#### Scenario: Maintainer names exact proposal surfaces before drafting specs

- GIVEN a maintainer copies `openspec/templates/proposal.md`
- WHEN they draft a new change proposal
- THEN they can name exact repo-relative files or artifacts touched by the change
- AND they can record ordering or dependency notes before implementation starts
- AND they can name the planned saved evidence or helper/gate runs for review

### Requirement: Design Template Captures Ordered Proof Planning

The system MUST ship a copyable OpenSpec design template at
`openspec/templates/design.md`.

That template MUST keep these exact second-level headings in this order, with no
additional second-level headings:

- `## Context`
- `## Goals / Non-Goals`
- `## Decisions`
- `## Deterministic Proof Plan`
- `## Risks / Trade-offs`

Inside `## Deterministic Proof Plan`, the template MUST keep these exact prompt
labels:

- `- scenario or requirement:`
- `- artifact path:`
- `- command / helper:`
- `- ordering / prerequisites:`
- `- cleanup / postcondition:`

#### Scenario: Maintainer binds each proof artifact to path and order

- GIVEN a maintainer copies `openspec/templates/design.md`
- WHEN they plan proof for a new change
- THEN they can bind each scenario or requirement to a saved artifact path
- AND they can name the command or helper that produces that artifact
- AND they can record prerequisites, sequencing, and cleanup expectations

### Requirement: Repo Guidance Points Authors At Stage Templates

The system MUST point proposal-stage and design-stage authors at the repo-local
OpenSpec template files before implementation.

`docs/contributor-guide.md` MUST contain these exact standalone markdown-link
lines:

- `[OpenSpec proposal template](../openspec/templates/proposal.md)`
- `[OpenSpec design template](../openspec/templates/design.md)`

`openspec/config.yaml` MUST contain one proposal rule that names
`openspec/templates/proposal.md` and one design rule that names
`openspec/templates/design.md`.

#### Scenario: Contributor guide exposes both OpenSpec stage templates

- GIVEN a maintainer opens `docs/contributor-guide.md`
- WHEN they look for repo-local OpenSpec authoring guidance
- THEN they can find the standalone proposal-template link line above
- AND they can find the standalone design-template link line above

#### Scenario: Config rules surface both templates to stage guidance

- GIVEN repo-local OpenSpec stage guidance reads `openspec/config.yaml`
- WHEN proposal-stage or design-stage prompts are prepared
- THEN proposal guidance names `openspec/templates/proposal.md`
- AND design guidance names `openspec/templates/design.md`
