## ADDED Requirements

### Requirement: Public Safety-Critical Claims Guide

The project MUST ship a public safety-critical claims guide at
`docs/safety-critical.md` that explains what Tiger Style can claim, what it
cannot claim, and how maintainers should write safety-sensitive docs and review
summaries without implying unsupported guarantees.

That guide MUST cover:

- supported Tiger Style claims such as linted policy, reviewable guidance, and
  explicit evidence expectations
- unsupported claims such as certification, regulator approval, or a complete
  safety case inferred only from lint cleanliness, tests, or review completion
- the difference among lints, tests, formal proof, assurance evidence, and
  certification
- the need for named evidence and human review on high-consequence claims
- concrete preferred/avoid wording for docs, rollout notes, and review
  summaries

Under each of those exact second-level headings, the guide MUST include at
least one explicit maintainer prompt or wording rule.

Those required second-level headings MUST appear exactly once, in the listed
order, and no additional second-level headings may appear in the guide.

That guide MUST keep these exact second-level headings:

- `## What Tiger Style can claim`
- `## What Tiger Style cannot claim`
- `## Lints, tests, proof, assurance, and certification`
- `## Evidence and human review`
- `## Responsible wording in docs and reviews`

#### Scenario: Reader sees what Tiger Style can and cannot claim

- GIVEN a maintainer or consumer opens `docs/safety-critical.md`
- WHEN they read the guide
- THEN they can see both the supported Tiger Style claim surface and the
  unsupported claims that the repo must not imply

### Requirement: Saved Deterministic Proof Artifacts For Safety-Critical Claims Guide

The implementation MUST save deterministic verification artifacts at these
paths:

- `openspec/changes/clarify-safety-critical-claims/evidence/safety-critical-guide-headings.txt`
- `openspec/changes/clarify-safety-critical-claims/evidence/readme-safety-critical-links.txt`
- `openspec/changes/clarify-safety-critical-claims/evidence/contributor-safety-critical-links.txt`
- `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review-labels.txt`

Those artifacts MUST capture these exact saved outputs:

- `safety-critical-guide-headings.txt` prints the full list of detected guide
  second-level headings, one per line, after verification confirms that list
  exactly matches the required headings in order with no extras.
- `readme-safety-critical-links.txt` prints `exists:docs/safety-critical.md`,
  then the exact README link line.
- `contributor-safety-critical-links.txt` prints
  `exists:docs/safety-critical.md`, `exists:openspec/specs/safety/spec.md`,
  then the two exact contributor-doc link lines.
- `human-safety-critical-review-labels.txt` prints these exact lines in order
  after exact-once validation succeeds:
  - `reviewer role: Tiger Style maintainer:1`
  - `reviewed artifact paths: docs/safety-critical.md:1`
  - `question 1: pass - guide distinguishes clearly between supported Tiger Style claims and unsupported certification or assurance claims:1`
  - `question 2: pass - guide explains the difference among lints, tests, formal proof, assurance evidence, and certification without overstating any one surface:1`
  - `question 3: pass - guide says high-consequence claims need named evidence and human review beyond green automation:1`
  - `question 4: pass - every required section includes at least one explicit maintainer prompt or wording rule:1`
  - `question 5: pass - guide gives explicit wording guidance for docs, reviews, and rollout notes so maintainers do not imply unsupported safety-critical guarantees:1`
  - `verdict: pass:1`

#### Scenario: Saved guide heading artifact keeps exact heading inventory

- GIVEN deterministic verification checks `docs/safety-critical.md`
- WHEN it saves `safety-critical-guide-headings.txt`
- THEN the artifact records the full verified heading inventory with no extras

#### Scenario: Saved README link artifact keeps existence and exact link line

- GIVEN deterministic verification checks `README.md`
- WHEN it saves `readme-safety-critical-links.txt`
- THEN the artifact records the required file existence line and exact README
  link line

#### Scenario: Saved contributor link artifact keeps existence and exact link lines

- GIVEN deterministic verification checks `docs/contributor-guide.md`
- WHEN it saves `contributor-safety-critical-links.txt`
- THEN the artifact records the required existence lines and exact contributor
  link lines

#### Scenario: Saved review-label artifact keeps exact label counts

- GIVEN deterministic verification checks
  `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`
- WHEN it saves `human-safety-critical-review-labels.txt`
- THEN the artifact records the exact required label-count lines in order

### Requirement: Human Review Record For Safety-Critical Claims Guide

The implementation MUST save one human/oracle review record at
`openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`
for the published guide.

That review record MUST include these exact labeled lines with `pass` or
`fail` answer tokens where applicable:

- `reviewer role: Tiger Style maintainer`
- `reviewed artifact paths: docs/safety-critical.md`
- `question 1: <pass|fail> - guide distinguishes clearly between supported Tiger Style claims and unsupported certification or assurance claims`
- `question 2: <pass|fail> - guide explains the difference among lints, tests, formal proof, assurance evidence, and certification without overstating any one surface`
- `question 3: <pass|fail> - guide says high-consequence claims need named evidence and human review beyond green automation`
- `question 4: <pass|fail> - every required section includes at least one explicit maintainer prompt or wording rule`
- `question 5: <pass|fail> - guide gives explicit wording guidance for docs, reviews, and rollout notes so maintainers do not imply unsupported safety-critical guarantees`
- `verdict: <pass|fail>`

For change acceptance, `question 1` through `question 5` MUST all be `pass`
and `verdict` MUST be `pass`. Any failed question MUST force `verdict: fail`
and block completion until the guide is corrected.

#### Scenario: Reviewer signs off claims language

- GIVEN the guide draft is complete
- WHEN a Tiger Style maintainer performs the human/oracle review
- THEN the saved evidence records the reviewer role, the reviewed artifact
  path, the required questions, and a final verdict

#### Scenario: Saved review record keeps required labels

- GIVEN the human/oracle review has been completed
- WHEN deterministic verification reads
  `openspec/changes/clarify-safety-critical-claims/evidence/human-safety-critical-review.txt`
- THEN the file contains the exact labeled lines required above
- AND `question 1` through `question 5` are all `pass`
- AND `verdict` is `pass`
