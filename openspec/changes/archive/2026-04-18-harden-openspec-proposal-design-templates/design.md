# Design: Harden OpenSpec Proposal And Design Templates

## Context

Tigerstyle already has a repo-local `openspec/templates/tasks.md`, and recent
OpenSpec work proves that a copyable template plus repo-local guidance can cut
review churn. Proposal and design authoring still lack that structure. Recent
`review_metrics promotions` data shows the next repeated failure classes are:

- proposal omission/prompt = 8
- design omission/guidance = 4
- design omission/prompt = 3

The repeated examples are consistent: proposals omit exact repo-relative
artifact paths or ordering notes, and designs name proof ideas without binding
saved artifacts to paths, helper commands, or cleanup expectations. This change
keeps scope on repo-local authoring aids and guidance before implementation.

## Goals / Non-Goals

**Goals:**
- give maintainers a copyable proposal shape before they draft a change
- front-load exact artifact paths, ordering, and evidence planning
- make design docs name an ordered proof plan instead of vague verification prose
- point repo-local OpenSpec guidance at the same template contract

**Non-Goals:**
- generic upstream OpenSpec changes
- new task-stage governance rules
- archive-wide rewrites of historical proposal/design files

## Decisions

### 1. Proposal template owns scope, artifact-path, and ordering prompts

**Choice:** Add `openspec/templates/proposal.md` with this exact second-level
heading inventory, in order, with no additional second-level headings:

- `## Why`
- `## What Changes`
- `## Non-Goals`
- `## Capabilities`
- `## Impact`

The template keeps exact prompt lines for the proposal hotspots that recent
review keeps requesting:

- `- **Files / artifacts**: <exact repo-relative paths touched by this change>`
- `- **Ordering / dependencies**: <what must land first or stay in lockstep>`
- `- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>`

It also keeps both capability subsections:

- `### New Capabilities`
- `### Modified Capabilities`

**Rationale:** Proposal churn is mostly missing structure, not missing effort.
One copyable shape makes authors name the exact surfaces and ordering before
spec/design/tasks work fans out.

**Alternative considered:** Keep proposal guidance only in prose comments inside
`openspec/config.yaml`. Rejected because repo-local review history already shows
that free-form proposals drift when no copyable file shape exists.

### 2. Design template owns ordered proof planning

**Choice:** Add `openspec/templates/design.md` with this exact second-level
heading inventory, in order, with no additional second-level headings:

- `## Context`
- `## Goals / Non-Goals`
- `## Decisions`
- `## Deterministic Proof Plan`
- `## Risks / Trade-offs`

Inside `## Deterministic Proof Plan`, each planned proof item keeps these exact
prompt labels:

- `- scenario or requirement:`
- `- artifact path:`
- `- command / helper:`
- `- ordering / prerequisites:`
- `- cleanup / postcondition:`

**Rationale:** The repeated design omissions are about missing save paths and
missing sequencing, not about whether verification exists at all.

**Alternative considered:** Leave proof planning in ad hoc decision prose.
Rejected because that keeps artifact paths and ordering implicit until tasks or
review catch them.

### 3. Contributor/config guidance points directly at the templates

**Choice:** Update `docs/contributor-guide.md` and `openspec/config.yaml` so
both proposal-stage and design-stage authoring point directly at the new
template files.

The contributor guide keeps these exact standalone links:

- `[OpenSpec proposal template](../openspec/templates/proposal.md)`
- `[OpenSpec design template](../openspec/templates/design.md)`

`openspec/config.yaml` keeps one proposal rule that names
`openspec/templates/proposal.md` and one design rule that names
`openspec/templates/design.md`.

**Rationale:** Templates do not help if authors cannot find them from the repo's
existing contributor and gate-guidance entry points.

**Alternative considered:** Add template files only. Rejected because the repo's
current gate/guidance surface would still leave proposal/design structure
undiscoverable unless someone already knew the paths.

### 4. Proof uses deterministic extracts, not narrative screenshots

**Choice:** Save three deterministic proof artifacts under
`openspec/changes/archive/2026-04-18-harden-openspec-proposal-design-templates/evidence/`.

Expected outputs:

- `proposal-template-fields.txt` contains the complete extracted `## ` heading
  inventory after no-extra validation, followed by these exact prompt lines:
  - `## Why`
  - `## What Changes`
  - `## Non-Goals`
  - `## Capabilities`
  - `## Impact`
  - `### New Capabilities`
  - `### Modified Capabilities`
  - `- **Files / artifacts**: <exact repo-relative paths touched by this change>`
  - `- **Ordering / dependencies**: <what must land first or stay in lockstep>`
  - `- **Verification / evidence**: <saved artifacts or helper/gate runs planned for review>`
- `design-template-fields.txt` contains the complete extracted `## ` heading
  inventory after no-extra validation, followed by these exact prompt lines:
  - `## Context`
  - `## Goals / Non-Goals`
  - `## Decisions`
  - `## Deterministic Proof Plan`
  - `## Risks / Trade-offs`
  - `- scenario or requirement:`
  - `- artifact path:`
  - `- command / helper:`
  - `- ordering / prerequisites:`
  - `- cleanup / postcondition:`
- `openspec-stage-authoring-crosswalk.txt` contains these exact lines:
  - `[OpenSpec proposal template](../openspec/templates/proposal.md)`
  - `[OpenSpec design template](../openspec/templates/design.md)`
  - `proposal rule -> openspec/templates/proposal.md`
  - `design rule -> openspec/templates/design.md`

**Rationale:** The change promises exact template/guidance shape, so exact text
extracts are the most reviewable proof.

## Risks / Trade-offs

**[More template ceremony]** -> Proposal/design files gain more placeholders.
Mitigation: keep the heading sets short and focused on the fields review keeps
missing.

**[Template drift from guidance]** -> Files may diverge from contributor/config
instructions. Mitigation: save one crosswalk artifact that proves both guidance
entry points reference the same template paths.

**[Overfitting to current review pain]** -> Later changes may need extra fields.
Mitigation: keep the templates minimal and exact, then extend them in a follow-
on change only if repeated review evidence justifies it.
