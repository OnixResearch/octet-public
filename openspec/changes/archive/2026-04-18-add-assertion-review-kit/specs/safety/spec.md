## ADDED Requirements

### Requirement: Public Tiger Style Assertion Review Guide

The project MUST ship a public assertion review guide at
`docs/assertion-review.md` that translates Tiger Style assertion philosophy
into a repeatable review checklist.

That guide MUST cover:

- why these checklist sections matter instead of presenting them as slogans
- assertions on arguments, return values, and invariants
- pair assertions across boundaries
- positive and negative space
- implication assertions and split checks
- compile-time assertions
- invalid-path testing
- explicit fail-fast framing for programmer errors
- explicit guidance that assertion philosophy is not only about raw density

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

Under each of those exact second-level headings, the guide MUST include at
least one explicit reviewer prompt or checklist question.

That guide MUST keep these exact second-level headings:

- `## Assertion targets: arguments, returns, invariants`
- `## Pair assertions across boundaries`
- `## Positive and negative space`
- `## Implication assertions and split checks`
- `## Compile-time assertions`
- `## Invalid-path testing`
- `## Reviewer prompts and failure handling`

Section ownership stays explicit:

- `## Assertion targets: arguments, returns, invariants` owns function
  contract checks.
- `## Pair assertions across boundaries` owns caller/callee, write/read, and
  encode/decode pairing.
- `## Positive and negative space` owns valid/invalid boundary reasoning.
- `## Implication assertions and split checks` owns implication-style checks
  and split compound assertions.
- `## Compile-time assertions` owns constant relations and design-time checks.
- `## Invalid-path testing` owns invalid-path and edge-case test planning.
- `## Reviewer prompts and failure handling` owns why the checklist matters,
  fail-fast framing for programmer errors, and the rule that assertion
  philosophy is not only about raw density.

#### Scenario: Guide keeps required headings and order

- GIVEN the guide has been drafted
- WHEN deterministic verification reads `docs/assertion-review.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Reviewing a boundary with caller and callee checks

- GIVEN a maintainer is reviewing a function, serializer, or storage boundary
- WHEN they follow `docs/assertion-review.md`
- THEN the guide prompts assertions on inputs, outputs, and invariants
- AND it prompts paired checks across the relevant boundary
- AND it keeps fail-fast framing visible for programmer errors

### Requirement: Copyable Assertion Checklist Template

The project MUST ship a copyable assertion checklist template at
`templates/assertion-checklist.md`.

That template MUST include fill-in fields for:

- the function, boundary, or invariant under review
- assertions on inputs, outputs, and invariants
- paired assertion sites
- positive and negative space cases
- implications and split assertions
- compile-time assertions
- invalid-path tests
- open risks and follow-ups

Every template section MUST stay a fill-in field or reviewer prompt rather
than free-form prose.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the template.

That template MUST keep these exact second-level headings:

- `## Function, boundary, or invariant under review`
- `## Assertions on inputs, outputs, and invariants`
- `## Paired assertion sites`
- `## Positive and negative space cases`
- `## Implications and split assertions`
- `## Compile-time assertions`
- `## Invalid-path tests`
- `## Open risks and follow-ups`

#### Scenario: Template keeps required headings and order

- GIVEN the template has been drafted
- WHEN deterministic verification reads `templates/assertion-checklist.md`
- THEN the required second-level headings appear exactly once and in the
  required order

#### Scenario: Planning invalid-path tests

- GIVEN a maintainer needs to review or plan invalid-path coverage
- WHEN they copy `templates/assertion-checklist.md`
- THEN they can record paired checks, negative-space cases, and invalid-path
  tests without inventing a format first

### Requirement: Saved Deterministic Proof Artifacts For Assertion Kit

The implementation MUST save deterministic verification artifacts at these
paths:

- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`
- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`
- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`

Those artifacts MUST capture these exact saved outputs:

- `assertion-guide-headings.txt` prints the full list of detected guide
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `assertion-template-headings.txt` prints the full list of detected template
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `readme-assertion-links.txt` prints `exists:docs/assertion-review.md`,
  `exists:templates/assertion-checklist.md`, then the two exact README link
  lines.
- `contributor-assertion-links.txt` prints `exists:docs/assertion-review.md`,
  `exists:templates/assertion-checklist.md`,
  `exists:openspec/specs/safety/spec.md`, then the three exact contributor-doc
  link lines.
- `human-assertion-review-labels.txt` prints these exact lines in order after
  exact-once validation succeeds, and it is generated only after
  `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
  already exists:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md:1`
  - `question 1: pass - guide explains why the checklist matters and includes reviewer prompts under every required heading:1`
  - `question 2: pass - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings:1`
  - `question 3: pass - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose:1`
  - `question 4: pass - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density:1`
  - `verdict: pass:1`

#### Scenario: Saved guide heading proof is reviewable

- GIVEN guide heading verification has passed
- WHEN a maintainer opens `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-guide-headings.txt`
- THEN the file exists
- AND it prints the full detected guide heading list in the required order

#### Scenario: Saved template heading proof is reviewable

- GIVEN template heading verification has passed
- WHEN a maintainer opens `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/assertion-template-headings.txt`
- THEN the file exists
- AND it prints the full detected template heading list in the required order

#### Scenario: Saved README link proof is reviewable

- GIVEN README link verification has passed
- WHEN a maintainer opens `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/readme-assertion-links.txt`
- THEN the file exists
- AND it prints the required `exists:` lines plus the two exact README link lines

#### Scenario: Saved contributor link proof is reviewable

- GIVEN contributor-guide link verification has passed
- WHEN a maintainer opens `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/contributor-assertion-links.txt`
- THEN the file exists
- AND it prints the required `exists:` lines plus the three exact contributor-guide link lines

#### Scenario: Saved review-label proof is reviewable

- GIVEN human review label verification has passed
- WHEN a maintainer opens `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review-labels.txt`
- THEN the file exists
- AND it prints the exact-once label counts for the required `pass` review record lines

### Requirement: Human Review Record For Assertion Kit

The implementation MUST save one human/oracle review record at
`openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
for the published guide and template. Each required label below MUST appear
exactly once in that file.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/assertion-review.md, templates/assertion-checklist.md`
- `question 1: <pass|fail> - guide explains why the checklist matters and includes reviewer prompts under every required heading`
- `question 2: <pass|fail> - guide covers every required assertion topic from Public Tiger Style Assertion Review Guide under the designated headings`
- `question 3: <pass|fail> - template covers every required field, and every section stays a fill-in field or prompt rather than free-form prose`
- `question 4: <pass|fail> - overall guide guidance says assertions fail fast on programmer errors, pair across boundaries, require invalid-path testing, and are not only about raw density`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 4` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the docs are corrected.

#### Scenario: Reviewer signs off assertion coverage

- GIVEN the guide and template drafts are complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  paths, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/archive/2026-04-18-add-assertion-review-kit/evidence/human-assertion-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 4` are all `pass`
- AND `verdict` is `pass`
