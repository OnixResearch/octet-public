## ADDED Requirements

### Requirement: Public Tiger Style Bounds And Pacing Review Guide

The project MUST ship a public bounds-and-pacing review guide at
`docs/bounds-and-pacing-review.md` that translates Tiger Style bounded
allocation and external-event pacing rules into a repeatable review checklist.

That guide MUST cover:

- why these checklist sections matter instead of presenting them as slogans
- allocation lifetime and ownership
- startup sizing and post-init bounds
- queues, buffers, and backpressure
- buffering external events before processing
- running the system at its own pace instead of reacting directly to each
  external event
- batch and work budgeting
- overload and failure behavior

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

That guide MUST keep these exact second-level headings:

- `## Allocation lifetime and ownership`
- `## Startup sizing and post-init bounds`
- `## Queues, buffers, and backpressure`
- `## External-event pacing`
- `## Batch and work budgeting`
- `## Overload and failure behavior`
- `## Reviewer prompts and escape hatches`

Section ownership stays explicit:

- `## Allocation lifetime and ownership` owns allocation lifetime and
  ownership.
- `## Startup sizing and post-init bounds` owns startup sizing and bounded
  post-init behavior.
- `## Queues, buffers, and backpressure` owns explicit bounds and
  backpressure choices.
- `## External-event pacing` owns buffering external events before processing
  and running the system at its own pace instead of reacting directly to each
  event.
- `## Batch and work budgeting` owns per-batch and per-tick work budgeting.
- `## Overload and failure behavior` owns overload handling and failure mode
  choices.
- `## Reviewer prompts and escape hatches` owns why the checklist matters,
  audited exceptions, and review cadence.

#### Scenario: Guide keeps required headings and order

- GIVEN the guide has been drafted
- WHEN deterministic verification reads `docs/bounds-and-pacing-review.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Reviewing queue or event-source ownership

- GIVEN a maintainer is reviewing a queue, buffer, or external event source
- WHEN they follow `docs/bounds-and-pacing-review.md`
- THEN the guide prompts explicit bounds, batching, and pacing ownership
- AND it keeps buffer-before-process guidance visible

### Requirement: Copyable Bounds And Pacing Template

The project MUST ship a copyable bounds-and-pacing checklist template at
`templates/bounds-and-pacing-checklist.md`.

That template MUST include fill-in fields for:

- the component or boundary under review
- the allocation plan
- startup sizing and post-init limits
- queue and buffer bounds
- backpressure choices
- external-event intake and pacing
- batch and work budget
- overload and failure behavior
- open risks and follow-ups

Every template section MUST stay a fill-in field or prompt rather than free-
form prose.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Component or boundary`
- `## Allocation plan`
- `## Startup sizing and post-init limits`
- `## Queue and buffer bounds`
- `## External-event intake and pacing`
- `## Batch and work budget`
- `## Overload and failure behavior`
- `## Open risks and follow-ups`

Template section ownership stays explicit:

- `## Component or boundary` owns the component or boundary under review.
- `## Allocation plan` owns allocation lifetime, ownership, and who may grow
  or retain resources.
- `## Startup sizing and post-init limits` owns startup sizing and bounded
  post-init behavior.
- `## Queue and buffer bounds` owns explicit queue and buffer bounds plus
  backpressure choices.
- `## External-event intake and pacing` owns buffering external events before
  processing and running at the system's own pace.
- `## Batch and work budget` owns per-batch and per-tick work budgeting.
- `## Overload and failure behavior` owns overload handling and failure mode
  choices.
- `## Open risks and follow-ups` owns residual risks and deferred follow-up
  work.

#### Scenario: Template keeps required headings and order

- GIVEN the template has been drafted
- WHEN deterministic verification reads
  `templates/bounds-and-pacing-checklist.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Planning allocation and pacing limits

- GIVEN a maintainer needs to plan queue bounds, backpressure, or startup sizing
- WHEN they copy `templates/bounds-and-pacing-checklist.md`
- THEN they can record allocation plans, backpressure choices, pacing choices,
  and overload behavior without inventing a format first

### Requirement: Saved Deterministic Proof Artifacts For Bounds And Pacing Kit

The implementation MUST save deterministic verification artifacts at these
paths:

- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-guide-headings.txt`
- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/bounds-template-headings.txt`
- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/readme-bounds-links.txt`
- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/contributor-bounds-links.txt`
- `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review-labels.txt`

Those artifacts MUST capture these exact saved outputs:

- `bounds-guide-headings.txt` prints the full list of detected guide
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `bounds-template-headings.txt` prints the full list of detected template
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `readme-bounds-links.txt` prints `exists:docs/bounds-and-pacing-review.md`,
  `exists:templates/bounds-and-pacing-checklist.md`, then the two exact README
  link lines.
- `contributor-bounds-links.txt` prints
  `exists:docs/bounds-and-pacing-review.md`,
  `exists:templates/bounds-and-pacing-checklist.md`,
  `exists:openspec/specs/safety/spec.md`, then the three exact contributor-doc
  link lines.
- `human-bounds-review-labels.txt` prints these exact lines in order after
  exact-once validation succeeds:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/bounds-and-pacing-review.md, templates/bounds-and-pacing-checklist.md:1`
  - `question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading:1`
  - `question 2: pass - guide covers every required bounds-and-pacing topic from Public Tiger Style Bounds And Pacing Review Guide under the designated headings:1`
  - `question 3: pass - template covers every required field, including backpressure choices, and every section stays a fill-in field or prompt rather than free-form prose:1`
  - `question 4: pass - the External-event pacing guidance says systems buffer external events and run at their own pace instead of reacting directly to every event:1`
  - `verdict: pass:1`

#### Scenario: Saved guide heading artifact keeps exact heading inventory

- GIVEN deterministic verification checks `docs/bounds-and-pacing-review.md`
- WHEN it saves `bounds-guide-headings.txt`
- THEN the artifact records the full verified heading inventory with no extras

#### Scenario: Saved template heading artifact keeps exact heading inventory

- GIVEN deterministic verification checks
  `templates/bounds-and-pacing-checklist.md`
- WHEN it saves `bounds-template-headings.txt`
- THEN the artifact records the full verified heading inventory with no extras

#### Scenario: Saved README link artifact keeps existence and exact link lines

- GIVEN deterministic verification checks `README.md`
- WHEN it saves `readme-bounds-links.txt`
- THEN the artifact records the required file existence lines and exact README
  link lines

#### Scenario: Saved contributor link artifact keeps existence and exact link lines

- GIVEN deterministic verification checks `docs/contributor-guide.md`
- WHEN it saves `contributor-bounds-links.txt`
- THEN the artifact records the required existence lines and exact contributor
  link lines

#### Scenario: Saved review-label artifact keeps exact label counts

- GIVEN deterministic verification checks
  `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
- WHEN it saves `human-bounds-review-labels.txt`
- THEN the artifact records the exact required label-count lines in order

### Requirement: Human Review Record For Bounds And Pacing Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
for the published guide and template. Each required label below MUST appear
exactly once in that file.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable, and each required label MUST appear
exactly once:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/bounds-and-pacing-review.md, templates/bounds-and-pacing-checklist.md`
- `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
- `question 2: <pass|fail> - guide covers every required bounds-and-pacing topic from Public Tiger Style Bounds And Pacing Review Guide under the designated headings`
- `question 3: <pass|fail> - template covers every required field, including backpressure choices, and every section stays a fill-in field or prompt rather than free-form prose`
- `question 4: <pass|fail> - the External-event pacing guidance says systems buffer external events and run at their own pace instead of reacting directly to every event`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 4` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the docs are corrected.

#### Scenario: Reviewer signs off bounds and pacing coverage

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/archive/2026-04-18-add-bounds-and-pacing-review-kit/evidence/human-bounds-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 4` are all `pass`
- AND `verdict` is `pass`
