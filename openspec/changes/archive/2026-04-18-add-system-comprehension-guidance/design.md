# Design: Add Tiger Style System Comprehension Guidance

## Context

Tiger Style has strong opinions about explicit boundaries and reviewable
structure, but the public docs still assume readers can reconstruct the system
shape on their own. That is not enough for onboarding or architecture review.
The missing public surface is a guide that teaches maintainers to present a
system in a reader-friendly order, plus a copyable template for system
walkthroughs.

## Goals / Non-Goals

**Goals:**
- Publish one stable public guide for system comprehension review.
- Publish one copyable walkthrough template.
- Make both artifacts discoverable from the README and contributor docs.
- Keep the content focused on architecture legibility and guided explanation,
  not automatic scoring.

**Non-Goals:**
- Automatic architecture quality scoring.
- Diagram generation or graph export.
- New lints for crate topology or module naming.
- Replacing contributor traceability docs with a generic onboarding manual.

## Decisions

### 1. Use a guide and a template, not an architecture scoring tool

**Choice:** Add `docs/system-comprehension.md` and
`templates/system-walkthrough.md` as doc-first review surfaces.

**Rationale:** The gap is public guidance about how to explain systems, not a
missing machine score. A guide and template improve review and onboarding
without pretending legibility can be reduced to one numeric threshold.

**Alternative:** Start with automatic architecture scoring. Rejected because the
repo does not yet have a stable public architecture doctrine to score against.

### 2. Keep the guide ordered around reader comprehension

**Choice:** The guide keeps these exact second-level headings, each exactly once
and in the listed order, with no extra second-level headings:
- `## Reader, boundary, and system slice`
- `## Structure before function`
- `## Functional walkthrough after structure`
- `## Deep dives, weak spots, and ownership`
- `## Top-level docs, crate roots, and dependency maps`
- `## Reviewer prompts for 2-minute legibility`

Section ownership stays explicit:
- `## Reader, boundary, and system slice` owns intended audience, scope, and
  the system slice under discussion.
- `## Structure before function` owns major components, boundaries, and the
  rule that readers need a map before dynamic behavior.
- `## Functional walkthrough after structure` owns the primary end-to-end flow.
- `## Deep dives, weak spots, and ownership` owns selected depth, known weak
  spots, and who owns unclear areas.
- `## Top-level docs, crate roots, and dependency maps` owns README, `lib.rs`,
  module tree, and dependency-surface guidance.
- `## Reviewer prompts for 2-minute legibility` owns whiteboard-style review
  prompts and the rule that a reader should be able to sketch the major system
  slice quickly.

Under each heading, the guide includes at least one explicit reviewer prompt or
checklist question.

**Rationale:** This change is about explanation order. Exact headings make the
ordered doctrine reviewable and keep deterministic proof strong.

**Alternative:** Publish a looser essay about onboarding. Rejected because it
would weaken traceability and make drift harder to detect.

### 3. Keep the template fill-in only and map it to the guide

**Choice:** The template keeps these exact second-level headings, each exactly
once and in the listed order, with no extra second-level headings:
- `## Intended reader and system slice`
- `## Major components and boundaries`
- `## Key relationships and dependency edges`
- `## Primary end-to-end flow`
- `## Deep dives and weak spots`
- `## Open questions and follow-ups`

Every template section stays a fill-in field or prompt rather than free-form
narrative prose.

**Rationale:** The guide explains how to think. The template gives maintainers a
copyable shape for design notes, onboarding notes, or review artifacts.

**Alternative:** Put example snippets inside the guide only. Rejected because
maintainers still need a reusable artifact shape.

### 4. Link the guide and template from public entry points only

**Choice:** Update `README.md` to add these exact Markdown links:
- `[System comprehension guide](docs/system-comprehension.md)`
- `[System walkthrough template](templates/system-walkthrough.md)`

Update `docs/contributor-guide.md` to add these exact Markdown links:
- `[System comprehension guide](system-comprehension.md)`
- `[System walkthrough template](../templates/system-walkthrough.md)`

This change does not add a contributor-doc link to a canonical main
architecture spec because no such synced main spec exists yet.

**Rationale:** Public entry points should surface the guide and template without
creating broken links to a future main spec path.

**Alternative:** Link to the change-local delta spec from contributor docs.
Rejected because contributor docs should not point at active change-local deltas
as if they were canonical public contracts.

### 5. Require a saved human review record for legibility quality

**Choice:** Save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`.
That file uses these exact labeled lines:
- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/system-comprehension.md, templates/system-walkthrough.md`
- `question 1: <pass|fail> - guide teaches readers to explain structure before function before depth`
- `question 2: <pass|fail> - guide covers reader, boundary, system slice, one primary end-to-end flow, and selected deep dives with ownership gaps`
- `question 3: <pass|fail> - guide makes top-level docs, crate roots, and dependency maps part of comprehension review`
- `question 4: <pass|fail> - every required guide heading includes at least one explicit reviewer prompt or checklist question`
- `question 5: <pass|fail> - template covers every required section and each section stays a fill-in prompt rather than free-form prose`
- `verdict: <pass|fail>`

For acceptance, questions 1 through 5 must all be `pass` and `verdict` must be
`pass`.

**Rationale:** Exact headings and links do not prove the guide is actually
usable for architecture legibility review.

### 6. Save deterministic proof artifacts with exact output contracts

**Choice:** Deterministic verification saves these exact artifacts under
`openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/`:
- `system-comprehension-guide-headings.txt` prints the full detected list of
  guide second-level headings, one per line, after validation confirms the list
  is an exact no-extra match.
- `system-walkthrough-template-headings.txt` prints the full detected list of
  template second-level headings, one per line, after validation confirms the
  list is an exact no-extra match.
- `readme-system-comprehension-links.txt` prints these exact lines in order:
  - `exists:docs/system-comprehension.md`
  - `exists:templates/system-walkthrough.md`
  - `[System comprehension guide](docs/system-comprehension.md)`
  - `[System walkthrough template](templates/system-walkthrough.md)`
- `contributor-system-comprehension-links.txt` prints these exact lines in
  order:
  - `exists:docs/system-comprehension.md`
  - `exists:templates/system-walkthrough.md`
  - `[System comprehension guide](system-comprehension.md)`
  - `[System walkthrough template](../templates/system-walkthrough.md)`
- `human-system-comprehension-review-labels.txt` prints these exact lines in
  order:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/system-comprehension.md, templates/system-walkthrough.md:1`
  - `question 1: pass - guide teaches readers to explain structure before function before depth:1`
  - `question 2: pass - guide covers reader, boundary, system slice, one primary end-to-end flow, and selected deep dives with ownership gaps:1`
  - `question 3: pass - guide makes top-level docs, crate roots, and dependency maps part of comprehension review:1`
  - `question 4: pass - every required guide heading includes at least one explicit reviewer prompt or checklist question:1`
  - `question 5: pass - template covers every required section and each section stays a fill-in prompt rather than free-form prose:1`
  - `verdict: pass:1`

**Rationale:** Exact artifacts make review independent of fuzzy summaries.

## Risks / Trade-offs

**[Reader-order guidance may feel prescriptive]** -> Some maintainers may prefer
free-form architecture notes. Mitigation: keep the guide about reviewable order,
not style-policing every sentence.

**[No canonical main architecture spec yet]** -> Contributors may expect a main
spec link immediately. Mitigation: keep docs linked only to guide/template until
a canonical spec exists through sync/archive.

**[Scope creep into diagram tooling]** -> Reviewers may ask for graph
visualization in the same change. Mitigation: keep tooling and scoring
explicitly out of scope.
