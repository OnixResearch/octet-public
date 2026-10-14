# Architecture Specification

## Purpose

Tiger Style architecture-legibility requirements for public system-comprehension guidance and reusable system walkthrough templates.

## Requirements
### Requirement: Public System Comprehension Guide

The project MUST ship a public system-comprehension guide at
`docs/system-comprehension.md` that teaches maintainers to explain system shape
in reader-first order: structure first, then function, then selected depth.

That guide MUST cover:

- intended reader, boundary, and system slice
- the rule that readers need major components and boundaries before dynamic
  behavior
- one primary end-to-end functional walkthrough after structure is established
- selected deep dives, weak spots, and ownership gaps
- top-level docs, crate roots, and dependency maps as comprehension surfaces
- reviewer prompts for a quick subsystem map or whiteboard-style legibility test

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

That guide MUST keep these exact second-level headings:

- `## Reader, boundary, and system slice`
- `## Structure before function`
- `## Functional walkthrough after structure`
- `## Deep dives, weak spots, and ownership`
- `## Top-level docs, crate roots, and dependency maps`
- `## Reviewer prompts for 2-minute legibility`

#### Scenario: Guide teaches structure before function before depth

- GIVEN a maintainer opens `docs/system-comprehension.md`
- WHEN they read the guide
- THEN they are taught to explain structure before function before selected
  depth

### Requirement: Copyable System Walkthrough Template

The project MUST ship a copyable walkthrough template at
`templates/system-walkthrough.md`.

That template MUST include fill-in fields for:

- intended reader and system slice
- major components and boundaries
- key relationships and dependency edges
- the primary end-to-end flow
- selected deep dives and weak spots
- open questions and follow-ups

Every template section MUST stay a fill-in field or reviewer prompt rather than
free-form prose.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Intended reader and system slice`
- `## Major components and boundaries`
- `## Key relationships and dependency edges`
- `## Primary end-to-end flow`
- `## Deep dives and weak spots`
- `## Open questions and follow-ups`

#### Scenario: Template captures a guided system walkthrough

- GIVEN a maintainer needs to capture a system walkthrough
- WHEN they copy `templates/system-walkthrough.md`
- THEN they can record boundaries, flow, weak spots, and follow-ups without
  inventing a format first

### Requirement: Human Review Record For System Comprehension Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`
for the published guide and template.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/system-comprehension.md, templates/system-walkthrough.md`
- `question 1: <pass|fail> - guide teaches readers to explain structure before function before depth`
- `question 2: <pass|fail> - guide covers reader, boundary, system slice, one primary end-to-end flow, and selected deep dives with ownership gaps`
- `question 3: <pass|fail> - guide makes top-level docs, crate roots, and dependency maps part of comprehension review`
- `question 4: <pass|fail> - every required guide heading includes at least one explicit reviewer prompt or checklist question`
- `question 5: <pass|fail> - template covers every required section and each section stays a fill-in prompt rather than free-form prose`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 5` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the guide or template is corrected.

#### Scenario: Reviewer signs off system-comprehension guidance quality

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/archive/2026-04-18-add-system-comprehension-guidance/evidence/human-system-comprehension-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 5` are all `pass`
- AND `verdict` is `pass`

